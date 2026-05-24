use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use crossbeam::{
    channel::Sender,
    deque::{Injector, Worker},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Clone)]
pub struct ScanTask {
    pub path: PathBuf,
    pub parent_id: LocalId,
    pub worker_id: u8,
    pub ancestors: Vec<(u64, u64)>,
}

pub enum ScanEvent {
    DirDiscovered {
        parent_worker_id: u8,
        child_worker_id: u8,
        local_parent_id: LocalId,
        local_child_id: LocalId,
        name: String,
    },
    FileDiscovered {
        parent_worker_id: u8,
        local_parent_id: LocalId,
        name: String,
        size: u64,
        is_symlink: bool,
    },
}

#[derive(Clone)]
pub struct TraversalStats {
    pub files_scanned: Arc<AtomicUsize>,
    pub dirs_scanned: Arc<AtomicUsize>,
    pub bytes_scanned: Arc<AtomicUsize>,
}

impl TraversalStats {
    pub fn reset(&self) {
        self.files_scanned.store(0, Ordering::SeqCst);
        self.dirs_scanned.store(0, Ordering::SeqCst);
        self.bytes_scanned.store(0, Ordering::SeqCst);
    }
}

pub struct TraversalEngine {
    num_threads: usize,
    stats: TraversalStats,
}

impl Default for TraversalEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TraversalEngine {
    #[must_use]
    pub fn new() -> Self {
        let num_threads = thread::available_parallelism().map_or(4, std::num::NonZero::get);
        Self {
            num_threads,
            stats: TraversalStats {
                files_scanned: Arc::new(AtomicUsize::new(0)),
                dirs_scanned: Arc::new(AtomicUsize::new(0)),
                bytes_scanned: Arc::new(AtomicUsize::new(0)),
            },
        }
    }

    #[must_use]
    pub const fn stats(&self) -> &TraversalStats {
        &self.stats
    }

    pub fn start_traversal(
        &self,
        root_path: PathBuf,
        event_tx: Sender<Vec<ScanEvent>>,
    ) -> Result<thread::JoinHandle<()>, crate::EdirstatError> {
        let num_threads = self.num_threads;
        let stats = self.stats.clone();

        let handle = thread::spawn(move || {
            // Setup global injector for starting and overflow tasks
            let injector = Arc::new(Injector::new());

            // Build initial scan task
            let root_id = (0, 0); // Placeholder for root
            let root_metadata = fs::metadata(&root_path);
            let root_file_id = root_metadata.as_ref().map_or(root_id, get_file_id);

            let initial_task = ScanTask {
                path: root_path.clone(),
                parent_id: LocalId(0),
                worker_id: 0,
                ancestors: vec![root_file_id],
            };
            injector.push(initial_task);

            // Create local worker queues and stealers
            let mut workers = Vec::with_capacity(num_threads);
            let mut stealers = Vec::with_capacity(num_threads);
            for _ in 0..num_threads {
                let w = Worker::new_fifo();
                let s = w.stealer();
                workers.push(w);
                stealers.push(s);
            }

            let stealers = Arc::new(stealers);
            let busy_workers = Arc::new(AtomicUsize::new(0));
            let done = Arc::new(AtomicBool::new(false));

            let mut thread_handles = Vec::with_capacity(num_threads);

            for worker_idx in 0..num_threads {
                let local_worker = workers.remove(0);
                let stealers = stealers.clone();
                let injector = injector.clone();
                let busy_workers = busy_workers.clone();
                let done = done.clone();
                let event_tx = event_tx.clone();

                let stats = stats.clone();

                thread_handles.push(thread::spawn(move || {
                    let mut local_id_counter = 1u32; // Root is 0, workers start generating local child IDs
                    let mut event_buffer = Vec::with_capacity(1024);
                    let worker_id_u8 = worker_idx as u8;

                    // Helper to push and flush events
                    let mut emit_event =
                        |event: ScanEvent, force_flush: bool, tx: &Sender<Vec<ScanEvent>>| {
                            event_buffer.push(event);
                            if event_buffer.len() >= 1024
                                || (force_flush && !event_buffer.is_empty())
                            {
                                let batch =
                                    std::mem::replace(&mut event_buffer, Vec::with_capacity(1024));
                                let _ = tx.send(batch);
                            }
                        };

                    loop {
                        // Find a task
                        let task_opt = local_worker.pop().or_else(|| {
                            // Try stealing from the global injector
                            let mut steal_res = injector.steal();
                            while steal_res.is_retry() {
                                steal_res = injector.steal();
                            }
                            if let crossbeam::deque::Steal::Success(t) = steal_res {
                                return Some(t);
                            }

                            // Work stealing: try stealing from other workers
                            for i in 0..stealers.len() {
                                if i == worker_idx {
                                    continue;
                                }
                                let mut steal_res = stealers[i].steal();
                                while steal_res.is_retry() {
                                    steal_res = stealers[i].steal();
                                }
                                if let crossbeam::deque::Steal::Success(t) = steal_res {
                                    return Some(t);
                                }
                            }
                            None
                        });

                        if let Some(task) = task_opt {
                            // Increment active busy counter
                            busy_workers.fetch_add(1, Ordering::SeqCst);

                            // Process the directory scan task
                            scan_directory(
                                &task,
                                worker_id_u8,
                                &mut local_id_counter,
                                &mut emit_event,
                                &event_tx,
                                &local_worker,
                                &stats,
                            );

                            // Decrement active busy counter
                            busy_workers.fetch_sub(1, Ordering::SeqCst);
                        } else {
                            // No tasks available. Check termination condition.
                            // If all queues are empty and busy_workers is 0, we're done!
                            if busy_workers.load(Ordering::SeqCst) == 0 && injector.is_empty() {
                                done.store(true, Ordering::SeqCst);
                            }

                            if done.load(Ordering::SeqCst) {
                                break;
                            }

                            // Wait briefly to prevent spinning
                            thread::sleep(Duration::from_micros(200));
                        }
                    }

                    // Flush final events remaining in buffer
                    if !event_buffer.is_empty() {
                        let _ = event_tx.send(event_buffer);
                    }
                }));
            }

            // Wait for all worker threads to finish
            for handle in thread_handles {
                let _ = handle.join();
            }
        });

        Ok(handle)
    }
}

fn scan_directory<F>(
    task: &ScanTask,
    worker_id: u8,
    local_id_counter: &mut u32,
    emit_event: &mut F,
    event_tx: &Sender<Vec<ScanEvent>>,
    local_worker: &Worker<ScanTask>,
    stats: &TraversalStats,
) where
    F: FnMut(ScanEvent, bool, &Sender<Vec<ScanEvent>>),
{
    let dir_path = &task.path;
    let parent_local_id = task.parent_id;

    // Try reading directory entries
    let Ok(entries) = fs::read_dir(dir_path) else {
        return;
    };

    stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

    for entry_res in entries {
        let Ok(entry) = entry_res else { continue };

        let path = entry.path();

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        let is_symlink = metadata.is_symlink();

        // Check if directory
        if metadata.is_dir() {
            let file_id = get_file_id(&metadata);

            // Cycle Detection
            if task.ancestors.contains(&file_id) {
                continue;
            }

            // Assign new local ID
            let child_local_id = LocalId(*local_id_counter);
            *local_id_counter += 1;

            // Emit directory discovery event immediately (force flush) to prevent work-stealing races
            emit_event(
                ScanEvent::DirDiscovered {
                    parent_worker_id: task.worker_id,
                    child_worker_id: worker_id,
                    local_parent_id: parent_local_id,
                    local_child_id: child_local_id,
                    name,
                },
                true,
                event_tx,
            );

            // Create a new task and push to local queue
            let mut new_ancestors = task.ancestors.clone();
            new_ancestors.push(file_id);

            let new_task = ScanTask {
                path,
                parent_id: child_local_id,
                worker_id,
                ancestors: new_ancestors,
            };
            local_worker.push(new_task);
        } else {
            // It's a file
            let size = metadata.len();
            stats.files_scanned.fetch_add(1, Ordering::Relaxed);
            stats
                .bytes_scanned
                .fetch_add(size as usize, Ordering::Relaxed);

            emit_event(
                ScanEvent::FileDiscovered {
                    parent_worker_id: task.worker_id,
                    local_parent_id: parent_local_id,
                    name,
                    size,
                    is_symlink,
                },
                false,
                event_tx,
            );
        }
    }

    // Force flush events after completing a directory scan to keep coordinator updated
    emit_event(
        ScanEvent::FileDiscovered {
            parent_worker_id: task.worker_id,
            local_parent_id: parent_local_id,
            name: String::new(),
            size: 0,
            is_symlink: false,
        },
        true,
        event_tx,
    );
}

#[cfg(unix)]
#[must_use]
pub fn get_file_id(meta: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (meta.dev(), meta.ino())
}

#[cfg(windows)]
pub fn get_file_id(meta: &fs::Metadata) -> (u64, u64) {
    use std::os::windows::fs::MetadataExt;
    (
        meta.volume_serial_number().unwrap_or(0) as u64,
        meta.file_index().unwrap_or(0),
    )
}

#[cfg(not(any(unix, windows)))]
pub fn get_file_id(_meta: &fs::Metadata) -> (u64, u64) {
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinator::{Coordinator, SharedState};

    #[test]
    fn test_traversal_and_coordinator() -> Result<(), crate::EdirstatError> {
        // Create a temporary directory structure in target/
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal");
        let subdir = temp_dir.join("subdir");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean old
        std::fs::create_dir_all(&subdir)?;

        // Write files
        let file1_path = subdir.join("file1.txt");
        let file2_path = temp_dir.join("file2.txt");
        std::fs::write(&file1_path, vec![0u8; 100])?;
        std::fs::write(&file2_path, vec![0u8; 200])?;

        // Initialize state
        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new();
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch traversal
        let handle = engine.start_traversal(temp_dir.clone(), tx)?;

        // Run coordinator in this thread (blocks until tx is dropped and all events processed)
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());

        // Wait for traversal thread to finish
        let _ = handle.join();

        // Verify stats
        let stats = engine.stats();
        assert_eq!(stats.files_scanned.load(Ordering::SeqCst), 2);
        assert_eq!(stats.dirs_scanned.load(Ordering::SeqCst), 2); // temp_dir and subdir
        assert_eq!(stats.bytes_scanned.load(Ordering::SeqCst), 300);

        // Verify snapshot tree structure
        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());

        // Root node
        let root = &snapshot.nodes[0];
        assert!(root.is_directory());
        assert_eq!(root.size, 300);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
