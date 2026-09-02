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

use compact_str::CompactString;
use crossbeam::{
    channel::Sender,
    deque::{Injector, Worker},
};

pub use edirstat_core::{file_id::get_file_id, state::TraversalStats};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Clone)]
pub struct ScanTask {
    pub path: PathBuf,
    pub parent_id: LocalId,
    pub worker_id: u8,
    pub ancestors: smallvec::SmallVec<[(u64, u64); 16]>,
    /// The device/volume identifier to restrict traversal within.
    pub expected_device_id: Option<u64>,
}

pub enum ScanEvent {
    DirDiscovered {
        parent_worker_id: u8,
        child_worker_id: u8,
        local_parent_id: LocalId,
        local_child_id: LocalId,
        name: CompactString,
        modified_timestamp: u32,
        created_timestamp: u32,
        no_permission: bool,
    },
    FileDiscovered {
        parent_worker_id: u8,
        local_parent_id: LocalId,
        name: CompactString,
        size: u64,
        is_symlink: bool,
        modified_timestamp: u32,
        created_timestamp: u32,
        no_permission: bool,
    },
    PermissionDenied {
        worker_id: u8,
        local_id: LocalId,
    },
}

pub struct TraversalEngine {
    num_threads: usize,
    stats: TraversalStats,
}

impl Default for TraversalEngine {
    fn default() -> Self {
        Self::new(TraversalStats::default())
    }
}

impl TraversalEngine {
    #[must_use]
    pub fn new(stats: TraversalStats) -> Self {
        let num_threads = thread::available_parallelism().map_or(4, std::num::NonZero::get);
        Self { num_threads, stats }
    }

    #[must_use]
    pub const fn stats(&self) -> &TraversalStats {
        &self.stats
    }

    #[must_use]
    pub const fn num_threads(&self) -> usize {
        self.num_threads
    }

    pub fn start_traversal(
        &self,
        root_path: PathBuf,
        same_filesystem: bool,
        scan_cancel: Arc<AtomicBool>,
        event_tx: Sender<Vec<ScanEvent>>,
    ) -> Result<thread::JoinHandle<()>, crate::EdirstatError> {
        let num_threads = self.num_threads;
        let stats = self.stats.clone();

        let handle = thread::spawn(move || {
            // Run MFT parser directly if target is a file named "$MFT" (case-insensitive)
            let is_mft_file = root_path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("$mft"));

            if is_mft_file {
                match super::mft::try_scan_mft(&root_path, &scan_cancel.clone(), &event_tx, &stats)
                {
                    Ok(()) => return,
                    Err(_) => {
                        stats.reset();
                    }
                }
            }

            // Attempt raw MFT parsing on Windows only if partition is explicitly detected as NTFS
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            {
                if super::mft::is_ntfs(&root_path) {
                    match super::mft::try_scan_mft(&root_path, &scan_cancel, &event_tx, &stats) {
                        Ok(()) => {
                            // Raw scan was executed successfully, end thread execution
                            return;
                        }
                        Err(_) => {
                            // Bypassed or failed raw access; fallback continues to parallel walker
                            stats.reset();
                        }
                    }
                }
            }

            // Setup global injector for starting and overflow tasks
            let injector = Arc::new(Injector::new());

            // Build initial scan task
            let root_id = (0, 0); // Placeholder for root
            let root_metadata = fs::metadata(&root_path);
            let root_file_id = root_metadata.as_ref().map_or(root_id, get_file_id);
            let expected_device_id = if same_filesystem {
                root_metadata.as_ref().map(get_device_id).ok()
            } else {
                None
            };

            let initial_task = ScanTask {
                path: root_path.clone(),
                parent_id: LocalId(0),
                worker_id: 0,
                ancestors: smallvec::smallvec![root_file_id],
                expected_device_id,
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
            // In-flight task count: tasks queued anywhere plus tasks currently
            // being scanned. Incremented when a task is queued (the root task
            // here, subdirectories inside `scan_directory`), decremented only
            // after a task's scan completes. New tasks are created exclusively
            // by in-progress scans, so a zero count means no work remains
            // anywhere and no more can ever appear. Counting busy workers
            // instead would miss tasks sitting unpicked in local queues.
            let in_flight_tasks = Arc::new(AtomicUsize::new(1)); // the root task
            let done = Arc::new(AtomicBool::new(false));

            let mut thread_handles = Vec::with_capacity(num_threads);

            for worker_idx in 0..num_threads {
                let local_worker = workers.remove(0);
                let stealers = stealers.clone();
                let injector = injector.clone();
                let in_flight_tasks = in_flight_tasks.clone();
                let done = done.clone();
                let event_tx = event_tx.clone();
                let scan_cancel = scan_cancel.clone();

                let stats = stats.clone();

                thread_handles.push(thread::spawn(move || {
                    let mut ctx = WorkerContext::new(
                        worker_idx as u8,
                        &event_tx,
                        &local_worker,
                        &stats,
                        &scan_cancel,
                        &in_flight_tasks,
                    );

                    loop {
                        if ctx.scan_cancel.load(Ordering::Relaxed) {
                            done.store(true, Ordering::SeqCst);
                        }
                        if done.load(Ordering::SeqCst) {
                            break;
                        }

                        // Find a task
                        let task_opt = ctx.local_worker.pop().or_else(|| {
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
                            // Process the directory scan task; its in-flight
                            // count is released only once the scan completes.
                            scan_directory(&task, &mut ctx);
                            in_flight_tasks.fetch_sub(1, Ordering::SeqCst);
                        } else {
                            // No tasks available. Check termination condition.
                            // A zero in-flight count is stable: no task is
                            // queued anywhere, no scan is in progress, and only
                            // in-progress scans can create new tasks.
                            if in_flight_tasks.load(Ordering::SeqCst) == 0 {
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
                    ctx.flush();
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

struct WorkerContext<'a> {
    worker_id: u8,
    local_id_counter: u32,
    event_buffer: Vec<ScanEvent>,
    event_tx: &'a Sender<Vec<ScanEvent>>,
    local_worker: &'a Worker<ScanTask>,
    stats: &'a TraversalStats,
    scan_cancel: &'a Arc<AtomicBool>,
    in_flight_tasks: &'a AtomicUsize,
}

impl<'a> WorkerContext<'a> {
    fn new(
        worker_id: u8,
        event_tx: &'a Sender<Vec<ScanEvent>>,
        local_worker: &'a Worker<ScanTask>,
        stats: &'a TraversalStats,
        scan_cancel: &'a Arc<AtomicBool>,
        in_flight_tasks: &'a AtomicUsize,
    ) -> Self {
        Self {
            worker_id,
            local_id_counter: 1, // Root is 0, workers start generating local child IDs
            event_buffer: Vec::with_capacity(1024),
            event_tx,
            local_worker,
            stats,
            scan_cancel,
            in_flight_tasks,
        }
    }

    fn emit_event(&mut self, event: ScanEvent, force_flush: bool) {
        self.event_buffer.push(event);
        if self.event_buffer.len() >= 1024 || (force_flush && !self.event_buffer.is_empty()) {
            let batch = std::mem::replace(&mut self.event_buffer, Vec::with_capacity(1024));
            let _ = self.event_tx.send(batch);
        }
    }

    fn flush(&mut self) {
        if !self.event_buffer.is_empty() {
            let batch = std::mem::replace(&mut self.event_buffer, Vec::with_capacity(1024));
            let _ = self.event_tx.send(batch);
        }
    }

    const fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.local_id_counter);
        self.local_id_counter += 1;
        id
    }
}

fn scan_directory(task: &ScanTask, ctx: &mut WorkerContext<'_>) {
    if ctx.scan_cancel.load(Ordering::Relaxed) {
        return;
    }
    let dir_path = &task.path;
    let parent_local_id = task.parent_id;

    // Try reading directory entries
    let Ok(entries) = fs::read_dir(dir_path) else {
        if let Err(e) = fs::read_dir(dir_path)
            && e.kind() == std::io::ErrorKind::PermissionDenied
        {
            ctx.emit_event(
                ScanEvent::PermissionDenied {
                    worker_id: task.worker_id,
                    local_id: parent_local_id,
                },
                true,
            );
        }
        return;
    };

    ctx.stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

    for (entry_idx, entry_res) in entries.enumerate() {
        if entry_idx % 256 == 0 && ctx.scan_cancel.load(Ordering::Relaxed) {
            break;
        }
        let Ok(entry) = entry_res else { continue };

        let Some(meta) = crate::arena::EntryMetadata::from_dir_entry(&entry) else {
            continue;
        };

        // Check if directory
        if meta.is_dir {
            // If we are scanning the system root, skip locations that contain
            // virtual files, network mounts, or sandboxed/containerized filesystems.
            if task.path == std::path::Path::new("/") {
                let name_str = meta.name.as_str();
                match name_str {
                    "proc" | "sys" | "dev" | "run" | "tmp" | "mnt" | "media" => continue,
                    _ => {}
                }
            }

            // Mount Point / Device boundary safety protection check
            if let Some(expected_dev) = task.expected_device_id
                && meta.file_id != (0, 0)
                && meta.file_id.0 != expected_dev
            {
                // Do not descend into subdirectories across filesystem boundaries (e.g. /sys or /proc)
                continue;
            }

            // Cycle Detection
            if meta.file_id != (0, 0) && task.ancestors.contains(&meta.file_id) {
                continue;
            }

            // Assign new local ID
            let child_local_id = ctx.next_local_id();

            // Emit directory discovery event immediately (force flush) to prevent work-stealing races
            ctx.emit_event(
                ScanEvent::DirDiscovered {
                    parent_worker_id: task.worker_id,
                    child_worker_id: ctx.worker_id,
                    local_parent_id: parent_local_id,
                    local_child_id: child_local_id,
                    name: meta.name,
                    modified_timestamp: meta.modified_timestamp,
                    created_timestamp: meta.created_timestamp,
                    no_permission: meta.no_permission,
                },
                true,
            );

            // Create a new task and push to local queue
            let mut new_ancestors = task.ancestors.clone();
            if meta.file_id != (0, 0) {
                new_ancestors.push(meta.file_id);
            }

            let new_task = ScanTask {
                path: entry.path(),
                parent_id: child_local_id,
                worker_id: ctx.worker_id,
                ancestors: new_ancestors,
                expected_device_id: task.expected_device_id,
            };
            ctx.local_worker.push(new_task);
            // Keep the in-flight count nonzero for as long as queued work
            // exists; this scan's own count is released after it returns, so
            // the increment always precedes that decrement.
            ctx.in_flight_tasks.fetch_add(1, Ordering::SeqCst);
        } else {
            // It's a file
            ctx.stats.files_scanned.fetch_add(1, Ordering::Relaxed);
            ctx.stats
                .bytes_scanned
                .fetch_add(meta.len as usize, Ordering::Relaxed);

            ctx.emit_event(
                ScanEvent::FileDiscovered {
                    parent_worker_id: task.worker_id,
                    local_parent_id: parent_local_id,
                    name: meta.name,
                    size: meta.len,
                    is_symlink: meta.is_symlink,
                    modified_timestamp: meta.modified_timestamp,
                    created_timestamp: meta.created_timestamp,
                    no_permission: meta.no_permission,
                },
                false,
            );
        }
    }

    // Force flush events after completing a directory scan to keep coordinator updated
    ctx.emit_event(
        ScanEvent::FileDiscovered {
            parent_worker_id: task.worker_id,
            local_parent_id: parent_local_id,
            name: CompactString::default(),
            size: 0,
            is_symlink: false,
            modified_timestamp: 0,
            created_timestamp: 0,
            no_permission: false,
        },
        true,
    );
}

#[cfg(unix)]
fn get_device_id(meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt as _;

    meta.dev()
}

#[cfg(windows)]
fn get_device_id(meta: &fs::Metadata) -> u64 {
    use std::os::windows::fs::MetadataExt as _;

    meta.volume_serial_number().unwrap_or(0) as u64
}

#[cfg(not(any(unix, windows)))]
fn get_device_id(_meta: &fs::Metadata) -> u64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{FileArenaSnapshot, NO_EXTENSION};
    use crate::coordinator::{Coordinator, SharedState};

    fn node_index(snapshot: &FileArenaSnapshot, name: &str) -> Option<usize> {
        snapshot
            .nodes
            .iter()
            .position(|n| snapshot.string_pool.get(n.name_id) == Some(name))
    }

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
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch traversal
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;

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

    #[test]
    #[cfg(unix)]
    fn test_traversal_permission_denied() -> Result<(), crate::EdirstatError> {
        use std::os::unix::fs::PermissionsExt as _;

        // If running as root skip this test.
        let is_root = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .is_some_and(|uid| uid == 0);

        if is_root {
            return Ok(());
        }

        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_perm");
        let subdir = temp_dir.join("noperm_subdir");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean old
        std::fs::create_dir_all(&subdir)?;

        // Set the subdirectory to no permissions
        let mut perms = std::fs::metadata(&subdir)?.permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&subdir, perms)?;

        // Initialize state
        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch traversal
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;

        // Run coordinator
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());

        // Wait for traversal thread to finish
        let _ = handle.join();

        // Restore permissions so we can clean up
        let mut restore_perms = std::fs::metadata(&subdir)?.permissions();
        restore_perms.set_mode(0o755);
        let _ = std::fs::set_permissions(&subdir, restore_perms);
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Verify that the restricted subdirectory node exists and has FLAG_NO_PERMISSION
        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());

        let mut found_noperm = false;
        for node in snapshot.nodes.iter() {
            let name = snapshot.string_pool.get(node.name_id).unwrap_or("");
            if name == "noperm_subdir" {
                assert!(node.has_no_permission());
                found_noperm = true;
            }
        }
        assert!(
            found_noperm,
            "Subdirectory with restricted permissions should be present in the snapshot with FLAG_NO_PERMISSION flag set"
        );

        Ok(())
    }

    #[test]
    fn test_traversal_same_filesystem() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_same_fs");
        let subdir = temp_dir.join("subdir");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&subdir)?;

        let file1_path = subdir.join("file1.txt");
        std::fs::write(&file1_path, vec![0u8; 50])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch traversal with same_filesystem = true
        let handle =
            engine.start_traversal(temp_dir.clone(), true, shared_state.scan_cancel.clone(), tx)?;

        let mut coordinator = Coordinator::new(rx, shared_state);
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());

        let _ = handle.join();

        let stats = engine.stats();
        assert_eq!(stats.files_scanned.load(Ordering::SeqCst), 1);
        assert_eq!(stats.dirs_scanned.load(Ordering::SeqCst), 2); // temp_dir and subdir
        assert_eq!(stats.bytes_scanned.load(Ordering::SeqCst), 50);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_empty_directory() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_empty");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let stats = engine.stats();
        assert_eq!(stats.dirs_scanned.load(Ordering::SeqCst), 1);
        assert_eq!(stats.files_scanned.load(Ordering::SeqCst), 0);
        assert_eq!(stats.bytes_scanned.load(Ordering::SeqCst), 0);

        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 1);
        let root = &snapshot.nodes[0];
        assert!(root.is_directory());
        assert_eq!(root.size, 0);
        assert_eq!(root.file_count, 0);
        assert!(shared_state.extension_stats.load().is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_nested_size_propagation() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_nested_sizes");
        let subdir = temp_dir.join("sub");
        let deepdir = subdir.join("deep");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&deepdir)?;

        std::fs::write(temp_dir.join("a.txt"), vec![0u8; 100])?;
        std::fs::write(subdir.join("b.txt"), vec![0u8; 200])?;
        std::fs::write(deepdir.join("c.txt"), vec![0u8; 50])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 6);
        // Sizes propagate bottom-up into every ancestor directory
        assert_eq!(snapshot.nodes[0].size, 350);
        assert_eq!(
            node_index(&snapshot, "sub").map(|i| snapshot.nodes[i].size),
            Some(250)
        );
        assert_eq!(
            node_index(&snapshot, "deep").map(|i| snapshot.nodes[i].size),
            Some(50)
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_same_filesystem_false_allows_subdirectories()
    -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_same_fs_false");
        let subdir = temp_dir.join("subvol_mock");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&subdir)?;

        std::fs::write(subdir.join("file.txt"), vec![0u8; 123])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch traversal with same_filesystem = false (should traverse all subdirectories)
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;

        let mut coordinator = Coordinator::new(rx, shared_state);
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let stats = engine.stats();
        assert_eq!(stats.files_scanned.load(Ordering::SeqCst), 1);
        assert_eq!(stats.dirs_scanned.load(Ordering::SeqCst), 2);
        assert_eq!(stats.bytes_scanned.load(Ordering::SeqCst), 123);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_file_count_and_dir_counts() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_counts");
        let subdir = temp_dir.join("sub");
        let deepdir = subdir.join("deep");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&deepdir)?;

        std::fs::write(temp_dir.join("a.txt"), vec![0u8; 100])?;
        std::fs::write(subdir.join("b.txt"), vec![0u8; 200])?;
        std::fs::write(deepdir.join("c.txt"), vec![0u8; 50])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 6);
        // file_count counts files recursively, directories excluded
        assert_eq!(snapshot.nodes[0].file_count, 3);
        assert_eq!(
            node_index(&snapshot, "sub").map(|i| snapshot.nodes[i].file_count),
            Some(2)
        );
        assert_eq!(
            node_index(&snapshot, "deep").map(|i| snapshot.nodes[i].file_count),
            Some(1)
        );
        // dir_counts counts subdirectories recursively
        assert_eq!(snapshot.dir_counts[0], 2);
        assert_eq!(
            node_index(&snapshot, "sub").map(|i| snapshot.dir_counts[i]),
            Some(1)
        );
        assert_eq!(
            node_index(&snapshot, "deep").map(|i| snapshot.dir_counts[i]),
            Some(0)
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_traversal_symlinks_not_followed() -> Result<(), crate::EdirstatError> {
        let base_dir = std::env::current_dir()?.join("target");
        let temp_dir = base_dir.join("test_traversal_symlinks");
        let outside_target = base_dir.join("test_traversal_symlinks_target");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::remove_dir_all(&outside_target);
        std::fs::create_dir_all(&temp_dir)?;
        std::fs::create_dir_all(&outside_target)?;

        // A directory outside the scanned tree, only reachable through a symlink
        std::fs::write(outside_target.join("secret.txt"), vec![0u8; 40])?;
        std::fs::write(temp_dir.join("real_file.txt"), vec![0u8; 10])?;
        std::os::unix::fs::symlink(&outside_target, temp_dir.join("dir_link"))?;
        std::os::unix::fs::symlink(temp_dir.join("real_file.txt"), temp_dir.join("file_link"))?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        // root + real_file.txt + dir_link + file_link; nothing through the links
        assert_eq!(snapshot.nodes.len(), 4);
        assert!(node_index(&snapshot, "dir_link").is_some_and(|i| {
            let node = &snapshot.nodes[i];
            node.is_symlink() && !node.is_directory() && node.first_child_opt().is_none()
        }));
        assert!(node_index(&snapshot, "file_link").is_some_and(|i| {
            let node = &snapshot.nodes[i];
            node.is_symlink() && !node.is_directory()
        }));
        assert!(node_index(&snapshot, "real_file.txt").is_some());
        // The symlinked directory's contents must not appear in the snapshot
        assert!(node_index(&snapshot, "secret.txt").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::remove_dir_all(&outside_target);
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_traversal_symlink_cycle_terminates() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_symlink_cycle");
        let loop_dir = temp_dir.join("loop");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&loop_dir)?;

        std::fs::write(loop_dir.join("f.txt"), vec![0u8; 10])?;
        // Symlink back to the scan root: a traversal cycle if it were followed
        std::os::unix::fs::symlink(&temp_dir, loop_dir.join("up"))?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        // Completing at all proves termination; the tree stays minimal
        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 4);
        assert!(node_index(&snapshot, "loop").is_some_and(|i| snapshot.nodes[i].is_directory()));
        assert!(node_index(&snapshot, "up").is_some_and(|i| snapshot.nodes[i].is_symlink()));

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_hidden_files_included() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_hidden");
        let hidden_dir = temp_dir.join(".hiddendir");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&hidden_dir)?;

        std::fs::write(temp_dir.join(".hidden"), vec![0u8; 60])?;
        std::fs::write(hidden_dir.join("inner.txt"), vec![0u8; 40])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        // The engine has no hidden-file filtering: everything contributes size
        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes[0].size, 100);
        assert!(node_index(&snapshot, ".hidden").is_some());
        assert!(node_index(&snapshot, ".hiddendir").is_some());
        assert!(node_index(&snapshot, "inner.txt").is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_extension_stats_aggregation() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_ext_stats");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        std::fs::write(temp_dir.join("a.txt"), vec![0u8; 10])?;
        std::fs::write(temp_dir.join("b.txt"), vec![0u8; 20])?;
        std::fs::write(temp_dir.join("c.rs"), vec![0u8; 5])?;
        std::fs::write(temp_dir.join("d.rs"), vec![0u8; 7])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        // (ext, total_size, file_count) sorted descending by total size
        let stats = shared_state.extension_stats.load();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].0.as_str(), "txt");
        assert_eq!(stats[0].1, 30);
        assert_eq!(stats[0].2, 2);
        assert_eq!(stats[1].0.as_str(), "rs");
        assert_eq!(stats[1].1, 12);
        assert_eq!(stats[1].2, 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_no_extension_bucket() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_no_extension");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        // No dot, trailing dot, and leading dot only all count as no-extension
        std::fs::write(temp_dir.join("Makefile"), vec![0u8; 100])?;
        std::fs::write(temp_dir.join("LICENSE"), vec![0u8; 50])?;
        std::fs::write(temp_dir.join("foo."), vec![0u8; 10])?;
        std::fs::write(temp_dir.join(".gitignore"), vec![0u8; 5])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let stats = shared_state.extension_stats.load();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0.as_str(), NO_EXTENSION);
        assert_eq!(stats[0].1, 165);
        assert_eq!(stats[0].2, 4);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_unicode_names() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_unicode");
        let unicode_dir = temp_dir.join("日本語");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&unicode_dir)?;

        std::fs::write(temp_dir.join("héllo wörld-✓.txt"), vec![0u8; 10])?;
        std::fs::write(unicode_dir.join("内部.txt"), vec![0u8; 20])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes[0].size, 30);
        assert!(node_index(&snapshot, "héllo wörld-✓.txt").is_some());
        assert!(node_index(&snapshot, "日本語").is_some());
        assert!(node_index(&snapshot, "内部.txt").is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_deep_nesting_terminates() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_deep_nesting");
        let mut deepest = temp_dir.clone();
        for depth in 0..64 {
            deepest = deepest.join(format!("d{depth}"));
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&deepest)?;
        std::fs::write(deepest.join("bottom.txt"), vec![0u8; 25])?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new(shared_state.scan_stats.clone());
        let (tx, rx) = crossbeam::channel::unbounded();
        let handle = engine.start_traversal(
            temp_dir.clone(),
            false,
            shared_state.scan_cancel.clone(),
            tx,
        )?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let stats = engine.stats();
        assert_eq!(stats.files_scanned.load(Ordering::SeqCst), 1);
        assert_eq!(stats.dirs_scanned.load(Ordering::SeqCst), 65); // root + 64 nested dirs

        let snapshot = shared_state.current_snapshot.load();
        assert_eq!(snapshot.nodes[0].file_count, 1);
        assert_eq!(snapshot.nodes[0].size, 25);
        assert_eq!(snapshot.dir_counts[0], 64);

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_deep_nesting_multithreaded_termination() -> Result<(), crate::EdirstatError> {
        // Regression test for a work-stealing termination race: an idle worker
        // could observe a "no work left" state in the window between a
        // subdirectory task being queued to a local worker queue and that task
        // being picked up, terminating the scan early. A serial 64-deep chain
        // forces one task handoff per level — the maximal exposure case.
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_deep_nesting_stress");
        let mut deepest = temp_dir.clone();
        for depth in 0..64 {
            deepest = deepest.join(format!("d{depth}"));
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&deepest)?;
        std::fs::write(deepest.join("bottom.txt"), vec![0u8; 25])?;

        for iteration in 0..25 {
            let shared_state = Arc::new(SharedState::new());
            let engine = TraversalEngine {
                num_threads: 32,
                stats: shared_state.scan_stats.clone(),
            };
            let (tx, rx) = crossbeam::channel::unbounded();
            let handle = engine.start_traversal(
                temp_dir.clone(),
                false,
                shared_state.scan_cancel.clone(),
                tx,
            )?;
            let mut coordinator = Coordinator::new(rx, shared_state.clone());
            coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
            let _ = handle.join();

            let stats = engine.stats();
            assert_eq!(
                stats.dirs_scanned.load(Ordering::SeqCst),
                65,
                "iteration {iteration}: scan truncated early (termination race)"
            );
            assert_eq!(
                stats.files_scanned.load(Ordering::SeqCst),
                1,
                "iteration {iteration}: bottom file missed"
            );
            let snapshot = shared_state.current_snapshot.load();
            assert_eq!(snapshot.nodes.len(), 66, "iteration {iteration}");
            assert_eq!(snapshot.nodes[0].size, 25, "iteration {iteration}");
            assert_eq!(snapshot.nodes[0].file_count, 1, "iteration {iteration}");
            assert_eq!(snapshot.dir_counts[0], 64, "iteration {iteration}");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_traversal_wide_tree_multithreaded_termination() -> Result<(), crate::EdirstatError> {
        // Same termination-race coverage with work spread across every queue:
        // 32 independent 8-deep chains so tasks constantly migrate between the
        // injector and local worker queues while other workers sit idle.
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_wide_stress");
        let _ = std::fs::remove_dir_all(&temp_dir);
        for chain in 0..32 {
            let mut dir = temp_dir.join(format!("chain{chain}"));
            for depth in 0..8 {
                dir = dir.join(format!("d{depth}"));
            }
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("leaf.txt"), vec![0u8; 7])?;
        }

        for iteration in 0..10 {
            let shared_state = Arc::new(SharedState::new());
            let engine = TraversalEngine {
                num_threads: 32,
                stats: shared_state.scan_stats.clone(),
            };
            let (tx, rx) = crossbeam::channel::unbounded();
            let handle = engine.start_traversal(
                temp_dir.clone(),
                false,
                shared_state.scan_cancel.clone(),
                tx,
            )?;
            let mut coordinator = Coordinator::new(rx, shared_state.clone());
            coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
            let _ = handle.join();

            let stats = engine.stats();
            // root + 32 chain dirs + 32*8 nested dirs
            assert_eq!(
                stats.dirs_scanned.load(Ordering::SeqCst),
                289,
                "iteration {iteration}: scan truncated early (termination race)"
            );
            assert_eq!(
                stats.files_scanned.load(Ordering::SeqCst),
                32,
                "iteration {iteration}: leaf files missed"
            );
            let snapshot = shared_state.current_snapshot.load();
            assert_eq!(snapshot.nodes[0].size, 32 * 7, "iteration {iteration}");
            assert_eq!(snapshot.nodes[0].file_count, 32, "iteration {iteration}");
            assert_eq!(snapshot.dir_counts[0], 288, "iteration {iteration}");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
