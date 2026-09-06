use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use compact_str::CompactString;
use crossbeam::channel::Receiver;

pub use edirstat_core::state::SharedState;

use super::traversal::{LocalId, ScanEvent};
use crate::arena::{
    FileArenaSnapshot, FileNode, NO_INDEX, NodeStorage, StringPool, precompute_dir_counts,
};

pub struct Coordinator {
    /// Lock-free channel to receive events
    event_rx: Receiver<Vec<ScanEvent>>,
    /// Shared state wrapper for swapping snapshots
    shared_state: Arc<SharedState>,
}

impl Coordinator {
    pub const fn new(event_rx: Receiver<Vec<ScanEvent>>, shared_state: Arc<SharedState>) -> Self {
        Self {
            event_rx,
            shared_state,
        }
    }

    pub fn run_coordinator_loop(&mut self, root_path_str: &str) {
        self.shared_state.is_scanning.store(true, Ordering::SeqCst);

        let mut arena = Vec::with_capacity(1024 * 1024); // Pre-allocate space for ~1M nodes
        let mut string_pool = StringPool::new();

        // Local extension tracking in the background thread
        let mut ext_map: std::collections::HashMap<CompactString, (u64, u32), ahash::RandomState> =
            std::collections::HashMap::with_hasher(ahash::RandomState::new());

        // Local to Global ID mapping: outer index is worker_id, inner is local_id.0
        let mut id_map: Vec<Vec<u32>> = Vec::new();

        // Track the last child inserted for each parent global index to ensure O(1) appends
        let mut last_child_map: Vec<u32> = Vec::new();

        // Register root directory node (Global ID 0)
        let root_name_id = string_pool.get_or_insert(root_path_str.as_bytes());
        let root_node = FileNode::new(root_name_id, None, true, false, 0, 0);
        arena.push(root_node);
        last_child_map.push(NO_INDEX);

        // Map root node: LocalId(0) for worker 0 is global index 0
        register_id(&mut id_map, 0, LocalId(0), 0);

        let mut last_publish = Instant::now();
        let mut publish_interval = Duration::from_millis(100);
        let mut dirty = false;

        while let Ok(batch) = self.event_rx.recv() {
            for event in batch {
                match event {
                    ScanEvent::DirDiscovered {
                        parent_worker_id,
                        child_worker_id,
                        local_parent_id,
                        local_child_id,
                        name,
                        modified_timestamp,
                        created_timestamp,
                        no_permission,
                    } => {
                        // Resolve parent global index using the parent's creator worker ID
                        if let Some(parent_global_id) =
                            resolve_id(&id_map, parent_worker_id, local_parent_id)
                        {
                            let name_id = string_pool.get_or_insert(name.as_bytes());
                            let child_global_id = arena.len() as u32;

                            // Create the directory node with initial timestamps
                            let mut dir_node = FileNode::new(
                                name_id,
                                Some(parent_global_id),
                                true,
                                false,
                                modified_timestamp,
                                created_timestamp,
                            );
                            if no_permission {
                                dir_node.flags |= FileNode::FLAG_NO_PERMISSION;
                            }
                            arena.push(dir_node);
                            last_child_map.push(NO_INDEX);

                            // Map worker's local child ID to our global index using the child's creator worker ID
                            register_id(
                                &mut id_map,
                                child_worker_id,
                                local_child_id,
                                child_global_id,
                            );

                            // Connect child to sibling chain in O(1) using last_child_map
                            connect_child(
                                &mut arena,
                                &mut last_child_map,
                                parent_global_id,
                                child_global_id,
                            );

                            dirty = true;
                        }
                    }
                    ScanEvent::FileDiscovered {
                        parent_worker_id,
                        local_parent_id,
                        name,
                        size,
                        is_symlink,
                        is_dataless,
                        is_special,
                        modified_timestamp,
                        created_timestamp,
                        no_permission,
                    } => {
                        if name.is_empty() && size == 0 {
                            // Directory completion signal
                            continue;
                        }

                        // Resolve parent global index using the parent's creator worker ID
                        if let Some(parent_global_id) =
                            resolve_id(&id_map, parent_worker_id, local_parent_id)
                        {
                            let name_id = string_pool.get_or_insert(name.as_bytes());
                            let file_global_id = arena.len() as u32;

                            // Create file node (parent pointer is set)
                            let mut file_node = FileNode::new(
                                name_id,
                                Some(parent_global_id),
                                false,
                                is_symlink,
                                modified_timestamp,
                                created_timestamp,
                            );
                            file_node.size = size;
                            if no_permission {
                                file_node.flags |= FileNode::FLAG_NO_PERMISSION;
                            }
                            if is_dataless {
                                file_node.flags |= FileNode::FLAG_DATALESS;
                            }
                            if is_special {
                                file_node.flags |= FileNode::FLAG_SPECIAL;
                            }
                            arena.push(file_node);
                            last_child_map.push(NO_INDEX);

                            // Connect child to sibling chain in O(1)
                            connect_child(
                                &mut arena,
                                &mut last_child_map,
                                parent_global_id,
                                file_global_id,
                            );

                            // O(1) Background Live Extension Tracking
                            let ext_slice = crate::arena::get_ext_slice(&name);
                            crate::arena::with_lowercase_ext(ext_slice, |ext_lowercased| {
                                let ext = CompactString::new(ext_lowercased);
                                let entry = ext_map.entry(ext).or_insert((0, 0));
                                entry.0 += size;
                                entry.1 += 1;
                            });

                            dirty = true;
                        }
                    }
                    ScanEvent::PermissionDenied {
                        worker_id,
                        local_id,
                    } => {
                        if let Some(global_id) = resolve_id(&id_map, worker_id, local_id) {
                            arena[global_id as usize].flags |= FileNode::FLAG_NO_PERMISSION;
                            dirty = true;
                        }
                    }
                }
            }

            // Publish snapshot if dirty and scaled interval elapsed
            if dirty && last_publish.elapsed() >= publish_interval {
                let current_size = arena.len();

                // Dynamically scale updates to prevent main thread rendering stutter
                publish_interval = if current_size > 500_000 {
                    Duration::from_millis(1000)
                } else if current_size > 100_000 {
                    Duration::from_millis(500)
                } else {
                    Duration::from_millis(100)
                };

                // Propagate sizes bottom-up before publishing
                let mut published_arena = arena.clone();
                propagate_all_sizes_bottom_up(&mut published_arena);

                let dir_counts = Arc::new(precompute_dir_counts(&published_arena));
                let snapshot = FileArenaSnapshot {
                    nodes: Arc::new(NodeStorage::Owned(published_arena)),
                    string_pool: Arc::new(string_pool.clone()),
                    dir_counts,
                };
                self.shared_state.store_snapshot(snapshot);

                // Publish background sorted statistics
                let mut stats_vec: Vec<(CompactString, u64, u32)> = ext_map
                    .iter()
                    .map(|(ext, &(total_size, file_count))| (ext.clone(), total_size, file_count))
                    .collect();
                stats_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
                self.shared_state.extension_stats.store(Arc::new(stats_vec));

                last_publish = Instant::now();
                dirty = false;
            }
        }

        let cancelled = self.shared_state.scan_cancel.load(Ordering::SeqCst);
        if cancelled {
            // Throw away all data. Store an empty snapshot!
            let empty_snapshot = FileArenaSnapshot {
                nodes: Arc::new(NodeStorage::Owned(Vec::new())),
                string_pool: Arc::new(StringPool::new()),
                dir_counts: Arc::new(Vec::new()),
            };
            self.shared_state.store_snapshot(empty_snapshot);
            self.shared_state
                .extension_stats
                .store(Arc::new(Vec::new()));
            self.shared_state.scan_stats.reset();
        } else {
            // Final size propagation and metrics compilation upon loop exit
            propagate_all_sizes_bottom_up(&mut arena);

            let dir_counts = Arc::new(precompute_dir_counts(&arena));
            let snapshot = FileArenaSnapshot {
                nodes: Arc::new(NodeStorage::Owned(arena)),
                string_pool: Arc::new(string_pool),
                dir_counts,
            };
            self.shared_state.store_snapshot(snapshot);

            let mut stats_vec: Vec<(CompactString, u64, u32)> = ext_map
                .into_iter()
                .map(|(ext, (total_size, file_count))| (ext, total_size, file_count))
                .collect();
            stats_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
            self.shared_state.extension_stats.store(Arc::new(stats_vec));
        }

        self.shared_state.is_scanning.store(false, Ordering::SeqCst);
    }
}

/// Accumulates file sizes, counts, and latest timestamps from leaf nodes up
/// to the root in a single, cache-friendly, reverse O(N) sweep.
fn propagate_all_sizes_bottom_up(arena: &mut [FileNode]) {
    for idx in (0..arena.len()).rev() {
        let parent = arena[idx].parent;
        if parent != NO_INDEX {
            let parent_idx = parent as usize;
            let size = arena[idx].size;
            let file_count = if arena[idx].is_directory() {
                arena[idx].file_count
            } else {
                1
            };
            let modified = arena[idx].modified_timestamp;
            let created = arena[idx].created_timestamp;

            // Update parent directly in contiguous slice memory
            let parent_node = &mut arena[parent_idx];
            parent_node.size += size;
            parent_node.file_count += file_count;
            if modified > parent_node.modified_timestamp {
                parent_node.modified_timestamp = modified;
            }
            if created > parent_node.created_timestamp {
                parent_node.created_timestamp = created;
            }
        }
    }
}

#[inline]
fn register_id(id_map: &mut Vec<Vec<u32>>, worker_id: u8, local_id: LocalId, global_id: u32) {
    let w_idx = worker_id as usize;
    if w_idx >= id_map.len() {
        id_map.resize(w_idx + 1, Vec::new());
    }
    let l_idx = local_id.0 as usize;
    if l_idx >= id_map[w_idx].len() {
        let needed_len = l_idx + 1;
        let new_len = needed_len.next_power_of_two().max(128);
        id_map[w_idx].resize(new_len, NO_INDEX);
    }
    id_map[w_idx][l_idx] = global_id;
}

#[inline]
fn resolve_id(id_map: &[Vec<u32>], worker_id: u8, local_id: LocalId) -> Option<u32> {
    let w_idx = worker_id as usize;
    if w_idx < id_map.len() {
        let l_idx = local_id.0 as usize;
        if l_idx < id_map[w_idx].len() {
            let gid = id_map[w_idx][l_idx];
            if gid != NO_INDEX {
                return Some(gid);
            }
        }
    }
    None
}

#[inline]
const fn connect_child(
    arena: &mut [FileNode],
    last_child_map: &mut [u32],
    parent_global_id: u32,
    child_global_id: u32,
) {
    let p_idx = parent_global_id as usize;
    let last_child = last_child_map[p_idx];

    if last_child == NO_INDEX {
        // This is the first child of the parent
        arena[p_idx].first_child = child_global_id;
    } else {
        // We have a previous child, attach as its next sibling
        arena[last_child as usize].next_sibling = child_global_id;
    }

    // Update the last child pointer for this parent
    last_child_map[p_idx] = child_global_id;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagate_all_sizes_bottom_up() {
        let mut pool = StringPool::new();
        let root_name = pool.get_or_insert(b"/root");
        let f1_name = pool.get_or_insert(b"f1");
        let d1_name = pool.get_or_insert(b"d1");
        let f2_name = pool.get_or_insert(b"f2");

        // root (dir) <- f1 (file, 100B), d1 (dir) <- f2 (file, 50B)
        let mut arena = vec![
            FileNode::new(root_name, None, true, false, 0, 0),
            FileNode::new(f1_name, Some(0), false, false, 1000, 500),
            FileNode::new(d1_name, Some(0), true, false, 0, 0),
            FileNode::new(f2_name, Some(2), false, false, 2000, 700),
        ];
        arena[1].size = 100;
        arena[3].size = 50;

        propagate_all_sizes_bottom_up(&mut arena);

        // Sizes and file counts accumulate from the leaves to the root
        assert_eq!(arena[2].size, 50);
        assert_eq!(arena[2].file_count, 1);
        assert_eq!(arena[0].size, 150);
        assert_eq!(arena[0].file_count, 2);
        // Parents take the maximum of their children's timestamps
        assert_eq!(arena[2].modified_timestamp, 2000);
        assert_eq!(arena[2].created_timestamp, 700);
        assert_eq!(arena[0].modified_timestamp, 2000);
        assert_eq!(arena[0].created_timestamp, 700);
    }

    #[test]
    fn test_register_and_resolve_id() {
        let mut id_map: Vec<Vec<u32>> = Vec::new();
        register_id(&mut id_map, 0, LocalId(5), 42);

        assert_eq!(resolve_id(&id_map, 0, LocalId(5)), Some(42));
        // Never-registered local id on a known worker
        assert_eq!(resolve_id(&id_map, 0, LocalId(6)), None);
        // Never-registered worker
        assert_eq!(resolve_id(&id_map, 3, LocalId(9)), None);
    }

    #[test]
    fn test_coordinator_drops_events_with_unresolvable_parent() -> Result<(), crate::EdirstatError>
    {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Worker 7 / LocalId 42 was never registered: the event is dropped silently
        tx.send(vec![ScanEvent::FileDiscovered {
            parent_worker_id: 7,
            local_parent_id: LocalId(42),
            name: CompactString::new("x"),
            size: 10,
            is_symlink: false,
            is_dataless: false,
            is_special: false,
            modified_timestamp: 0,
            created_timestamp: 0,
            no_permission: false,
        }])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 1);
        assert!(snapshot.nodes[0].is_directory());
        assert!(shared.extension_stats.load().is_empty());
        Ok(())
    }

    #[test]
    fn test_coordinator_skips_sentinel_flush_event() -> Result<(), crate::EdirstatError> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        // Empty name + zero size is the directory-completion sentinel, not a file
        tx.send(vec![ScanEvent::FileDiscovered {
            parent_worker_id: 0,
            local_parent_id: LocalId(0),
            name: CompactString::default(),
            size: 0,
            is_symlink: false,
            is_dataless: false,
            is_special: false,
            modified_timestamp: 0,
            created_timestamp: 0,
            no_permission: false,
        }])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 1);
        Ok(())
    }

    #[test]
    fn test_coordinator_permission_denied_sets_flag() -> Result<(), crate::EdirstatError> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        tx.send(vec![
            ScanEvent::DirDiscovered {
                parent_worker_id: 0,
                child_worker_id: 0,
                local_parent_id: LocalId(0),
                local_child_id: LocalId(1),
                name: CompactString::new("d"),
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
            ScanEvent::PermissionDenied {
                worker_id: 0,
                local_id: LocalId(1),
            },
        ])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 2);
        assert!(!snapshot.nodes[0].has_no_permission());
        assert_eq!(
            snapshot.string_pool.get(snapshot.nodes[1].name_id),
            Some("d")
        );
        assert!(snapshot.nodes[1].has_no_permission());
        Ok(())
    }

    #[test]
    fn test_coordinator_builds_tree_from_events() -> Result<(), crate::EdirstatError> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        tx.send(vec![
            ScanEvent::DirDiscovered {
                parent_worker_id: 0,
                child_worker_id: 0,
                local_parent_id: LocalId(0),
                local_child_id: LocalId(1),
                name: CompactString::new("d1"),
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(1),
                name: CompactString::new("f1"),
                size: 64,
                is_symlink: false,
                is_dataless: false,
                is_special: false,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(0),
                name: CompactString::new("f2"),
                size: 36,
                is_symlink: false,
                is_dataless: false,
                is_special: false,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
        ])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 4);

        let root = &snapshot.nodes[0];
        assert!(root.is_directory());
        assert_eq!(root.size, 100);
        assert_eq!(root.file_count, 2);

        let d1 = &snapshot.nodes[1];
        assert_eq!(snapshot.string_pool.get(d1.name_id), Some("d1"));
        assert!(d1.is_directory());
        assert_eq!(d1.parent, 0);
        assert_eq!(d1.size, 64);
        assert_eq!(d1.file_count, 1);

        let f1 = &snapshot.nodes[2];
        assert_eq!(snapshot.string_pool.get(f1.name_id), Some("f1"));
        assert!(!f1.is_directory());
        assert_eq!(f1.parent, 1);
        assert_eq!(f1.size, 64);

        let f2 = &snapshot.nodes[3];
        assert_eq!(snapshot.string_pool.get(f2.name_id), Some("f2"));
        assert_eq!(f2.parent, 0);
        assert_eq!(f2.size, 36);

        assert_eq!(snapshot.get_full_path(0), "/root");
        assert_eq!(snapshot.get_full_path(1), "/root/d1");
        assert_eq!(snapshot.get_full_path(2), "/root/d1/f1");
        assert_eq!(snapshot.get_full_path(3), "/root/f2");
        Ok(())
    }

    #[test]
    fn test_coordinator_extension_stats_case_insensitive() -> Result<(), crate::EdirstatError> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        tx.send(vec![
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(0),
                name: CompactString::new("A.TXT"),
                size: 10,
                is_symlink: false,
                is_dataless: false,
                is_special: false,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(0),
                name: CompactString::new("b.txt"),
                size: 20,
                is_symlink: false,
                is_dataless: false,
                is_special: false,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
        ])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let stats = shared.extension_stats.load();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0.as_str(), "txt");
        assert_eq!(stats[0].1, 30);
        assert_eq!(stats[0].2, 2);
        Ok(())
    }

    #[test]
    fn test_coordinator_populates_dataless_and_special_flags() -> Result<(), crate::EdirstatError> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();

        tx.send(vec![
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(0),
                name: CompactString::new("cloud.mp4"),
                size: 1_000_000,
                is_symlink: false,
                is_dataless: true,
                is_special: false,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
            ScanEvent::FileDiscovered {
                parent_worker_id: 0,
                local_parent_id: LocalId(0),
                name: CompactString::new("ipc.sock"),
                size: 0,
                is_symlink: false,
                is_dataless: false,
                is_special: true,
                modified_timestamp: 0,
                created_timestamp: 0,
                no_permission: false,
            },
        ])
        .map_err(std::io::Error::other)?;
        drop(tx);

        let mut coordinator = Coordinator::new(rx, shared.clone());
        coordinator.run_coordinator_loop("/root");

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 3);

        let cloud_node = &snapshot.nodes[1];
        assert_eq!(
            snapshot.string_pool.get(cloud_node.name_id),
            Some("cloud.mp4")
        );
        assert!(cloud_node.is_dataless());
        assert!(!cloud_node.is_special());

        let special_node = &snapshot.nodes[2];
        assert_eq!(
            snapshot.string_pool.get(special_node.name_id),
            Some("ipc.sock")
        );
        assert!(!special_node.is_dataless());
        assert!(special_node.is_special());

        Ok(())
    }
}
