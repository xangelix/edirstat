use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use fluent_zero::t;
use prettier_bytes::ByteFormatter;
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

pub const HASH_BLOCK_SIZE: usize = 4096; // 4KB hashing block size
pub const MULTI_RANGE_SPREAD_SIZE: u64 = 100 * 1024 * 1024; // 100MB spread size for multi-range checks

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashResult {
    Success([u8; 32]),
    Skipped,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub size: u64,
    pub nodes: Vec<u32>,
    pub file_ids: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
pub struct DuplicateRow {
    pub node_idx: u32,
    pub group_idx: usize,
    pub filename: String,
    pub parent_path: String,
    pub size: u64,
    pub size_str: String,
    pub reclaimable_str: String,
    /// Raw Unix timestamp for the created time; formatted at render-time.
    pub created_timestamp: u32,
    /// Raw Unix timestamp for the modified time; formatted at render-time.
    pub modified_timestamp: u32,
    pub is_original: bool,
    pub is_hardlink: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DeduplicationResults {
    pub groups: Vec<DuplicateGroup>,
    pub flat_rows: Vec<DuplicateRow>,
}

impl DeduplicationResults {
    pub fn rebuild_flat_rows(&mut self, snapshot: &crate::arena::FileArenaSnapshot) {
        let current_total_files: usize = self.groups.iter().map(|g| g.nodes.len()).sum();
        let mut flat_rows = Vec::with_capacity(current_total_files);

        for (g_idx, group) in self.groups.iter().enumerate() {
            let mut paired: Vec<(u32, (u64, u64))> = group
                .nodes
                .iter()
                .copied()
                .zip(group.file_ids.iter().copied())
                .collect();
            if paired.len() < group.nodes.len() {
                paired = group
                    .nodes
                    .iter()
                    .copied()
                    .map(|idx| (idx, (0, 0)))
                    .collect();
            }

            paired.sort_by(|a, b| {
                let node_a = &snapshot.nodes[a.0 as usize];
                let node_b = &snapshot.nodes[b.0 as usize];

                let mod_cmp = node_a.modified_timestamp.cmp(&node_b.modified_timestamp);
                if mod_cmp != std::cmp::Ordering::Equal {
                    return mod_cmp;
                }

                let cre_cmp = node_a.created_timestamp.cmp(&node_b.created_timestamp);
                if cre_cmp != std::cmp::Ordering::Equal {
                    return cre_cmp;
                }

                let name_a = snapshot.string_pool.get(node_a.name_id).unwrap_or("");
                let name_b = snapshot.string_pool.get(node_b.name_id).unwrap_or("");
                name_a.len().cmp(&name_b.len())
            });

            let unique_inodes_count = {
                let mut ids: Vec<(u64, u64)> = paired
                    .iter()
                    .map(|p| p.1)
                    .filter(|&id| id != (0, 0))
                    .collect();
                ids.sort_unstable();
                ids.dedup();
                ids.len()
            };
            let total_reclaimable = if unique_inodes_count > 0 {
                group.size * (unique_inodes_count as u64 - 1)
            } else {
                group.size * (paired.len().saturating_sub(1) as u64)
            };

            for (f_idx, &(node_idx, file_id)) in paired.iter().enumerate() {
                let full_path = snapshot.get_full_path(node_idx);
                let path = std::path::Path::new(&full_path);

                let filename = path
                    .file_name()
                    .map_or_else(String::new, |s| s.to_string_lossy().into_owned());

                let parent_path = crate::model::arena::clean_unc_path(
                    &path
                        .parent()
                        .map_or_else(String::new, |s| s.to_string_lossy().into_owned()),
                )
                .into_owned();

                let node = &snapshot.nodes[node_idx as usize];

                let is_original = f_idx == 0;

                let is_hardlink = file_id != (0, 0)
                    && paired
                        .iter()
                        .any(|other| other.0 != node_idx && other.1 == file_id);

                let size_str = ByteFormatter::new().format(group.size).to_string();

                let reclaimable_str = if is_original {
                    ByteFormatter::new().format(total_reclaimable).to_string()
                } else {
                    let individual_reclaimable = if is_hardlink { 0 } else { group.size };
                    ByteFormatter::new()
                        .format(individual_reclaimable)
                        .to_string()
                };

                flat_rows.push(DuplicateRow {
                    node_idx,
                    group_idx: g_idx,
                    filename,
                    parent_path,
                    size: group.size,
                    size_str,
                    reclaimable_str,
                    // Store raw timestamps; the UI formats these at render-time.
                    created_timestamp: node.created_timestamp,
                    modified_timestamp: node.modified_timestamp,
                    is_original,
                    is_hardlink,
                });
            }
        }
        self.flat_rows = flat_rows;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeduplicatorConfig {
    pub min_size: u64,
    pub ignore_system: bool,
    pub ignore_hidden: bool,
}

impl Default for DeduplicatorConfig {
    fn default() -> Self {
        Self {
            min_size: 1024, // 1 KB default
            ignore_system: true,
            ignore_hidden: true,
        }
    }
}

/// Helper function to check if a path is a system file or hidden
#[must_use]
pub fn is_excluded_path(path: &str, ignore_system: bool, ignore_hidden: bool) -> bool {
    if ignore_system {
        // Predefined lowercase segments to avoid runtime conversion allocations
        let system_patterns = [
            "system volume information",
            "$recycle.bin",
            "windows/system32",
            "/etc/",
            "/var/",
            "/usr/",
            "/proc/",
            "/sys/",
            "/dev/",
            "swapfile",
            "pagefile.sys",
        ];

        for pattern in &system_patterns {
            if crate::model::arena::contains_case_insensitive(path, pattern) {
                return true;
            }
        }
    }

    if ignore_hidden {
        // Hidden file/dir check: segments starting with a dot
        for segment in path.split(['/', '\\']) {
            if segment.starts_with('.') && segment != "." && segment != ".." {
                return true;
            }
        }
    }

    false
}

/// Hashing a range of a file
fn calculate_hash_at_range(
    path: &str,
    start_offset: u64,
    len: usize,
    expected_modified: u32,
    expected_created: u32,
) -> Option<[u8; 32]> {
    // 1. Verify metadata timestamps before reading.
    // Timestamps are stored as u32 epoch seconds, so compare the lower 32 bits.
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_modified
    {
        return None; // modified since snapshot
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_created
    {
        return None; // created since snapshot
    }

    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(start_offset)).ok()?;

    let mut buffer = [0u8; HASH_BLOCK_SIZE];
    let read_len = len.min(HASH_BLOCK_SIZE);
    let n = file.read(&mut buffer[..read_len]).ok()?;

    let hash = blake3::hash(&buffer[..n]);
    Some(hash.into())
}

/// Multi-range hashing spread across large files (every 100MB)
fn calculate_multi_range_hash(
    path: &str,
    file_size: u64,
    expected_modified: u32,
    expected_created: u32,
) -> Option<[u8; 32]> {
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_modified
    {
        return None;
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_created
    {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; HASH_BLOCK_SIZE];

    let mut offset = MULTI_RANGE_SPREAD_SIZE;
    while offset + HASH_BLOCK_SIZE as u64 <= file_size {
        file.seek(SeekFrom::Start(offset)).ok()?;
        let n = file.read(&mut buffer).ok()?;
        hasher.update(&buffer[..n]);
        offset += MULTI_RANGE_SPREAD_SIZE;
    }

    Some(hasher.finalize().into())
}

/// Full cryptographic hash of the entire file contents
fn calculate_full_hash(
    path: &str,
    expected_modified: u32,
    expected_created: u32,
) -> Option<[u8; 32]> {
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_modified
    {
        return None;
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as u32 != expected_created
    {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 16384]; // 16KB stack buffer

    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Some(hasher.finalize().into())
}

pub type HashFn = dyn Fn(&str, u64, u32, u32) -> HashResult + Send + Sync;

/// Main execution routine of the background deduplication runner
#[allow(clippy::needless_pass_by_value)]
pub fn run_deduplication(
    snapshot: Arc<crate::arena::FileArenaSnapshot>,
    progress: atomic_progress::Progress,
    results: Arc<parking_lot::RwLock<DeduplicationResults>>,
    cancel: Arc<AtomicBool>,
    config: DeduplicatorConfig,
) {
    let start_time = Instant::now();
    let is_cancelled = || cancel.load(Ordering::SeqCst);
    let update_results = |groups: Vec<DuplicateGroup>| {
        let mut guard = results.write();
        guard.groups = groups;
        guard.rebuild_flat_rows(&snapshot);
    };

    let cancel_and_clear = || {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        *results.write() = DeduplicationResults::default();
    };

    if is_cancelled() {
        cancel_and_clear();
        return;
    }

    // Set state to SizeGrouping (Phase 1)
    progress.set_name(t!("dedup-phase1-size"));
    progress.set_total(0); // Indeterminate spinner initially
    progress.set_pos(0);

    // Initial rapid grouping purely by size
    let mut size_groups: HashMap<u64, Vec<u32>, ahash::RandomState> =
        HashMap::with_capacity_and_hasher(512, ahash::RandomState::new());

    for (idx, node) in snapshot.nodes.iter().enumerate() {
        if is_cancelled() {
            break;
        }

        if node.is_directory() || node.is_symlink() {
            continue;
        }

        let size = node.size;
        if size < config.min_size {
            continue;
        }

        size_groups.entry(size).or_default().push(idx as u32);
    }

    if is_cancelled() {
        cancel_and_clear();
        return;
    }

    // Identify candidate size groups containing duplicates
    let candidate_groups_count = size_groups
        .iter()
        .filter(|(_, nodes)| nodes.len() >= 2)
        .count();
    progress.set_name(t!("dedup-phase1-filter"));
    progress.set_total(candidate_groups_count as u64);
    progress.set_pos(0);

    let mut expected_timestamps = HashMap::with_capacity_and_hasher(
        (snapshot.nodes.len() / 10).max(16), // Heuristic estimate
        ahash::RandomState::new(),
    );
    let mut current_groups = Vec::new();
    let mut progress_counter = 0;

    // Filter candidate size groups through path exclusion checks
    for (size, nodes) in size_groups {
        if is_cancelled() {
            break;
        }

        if nodes.len() < 2 {
            continue;
        }

        progress_counter += 1;
        if progress_counter % 50 == 0 || progress_counter == candidate_groups_count {
            progress.set_pos(progress_counter as u64);
        }

        let mut filtered_nodes = Vec::new();
        for node_idx in nodes {
            let path = snapshot.get_full_path(node_idx);

            // Throttle UI update of processed files
            if node_idx % 100 == 0 {
                progress.set_item(path.clone());
            }

            if is_excluded_path(&path, config.ignore_system, config.ignore_hidden) {
                continue;
            }

            let node = &snapshot.nodes[node_idx as usize];
            expected_timestamps.insert(node_idx, (node.modified_timestamp, node.created_timestamp));
            filtered_nodes.push(node_idx);
        }

        if filtered_nodes.len() >= 2 {
            current_groups.push(DuplicateGroup {
                size,
                nodes: filtered_nodes,
                file_ids: Vec::new(),
            });
        }
    }

    if is_cancelled() {
        cancel_and_clear();
        return;
    }

    // Sort descending by size to process larger files first
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));

    update_results(current_groups.clone());

    // Generic helper to process candidate groups during each hashing phase
    let run_hashing_phase = |current_groups: Vec<DuplicateGroup>,
                             hash_fn: &HashFn,
                             phase_name: &str|
     -> Option<Vec<DuplicateGroup>> {
        let total_groups = current_groups.len();
        progress.set_name(phase_name);
        progress.set_total(total_groups as u64);
        progress.set_pos(0);

        // Thread-safe atomic counter to track completed groups out-of-order
        let completed_groups = std::sync::atomic::AtomicUsize::new(0);

        // Process size blocks concurrently across all CPU cores
        let next_groups_res: Result<Vec<Vec<DuplicateGroup>>, ()> = current_groups
            .into_par_iter()
            .map(|group| {
                if cancel.load(Ordering::Relaxed) {
                    return Err(());
                }

                let mut local_groups = Vec::new();
                let mut hash_subgroups: HashMap<[u8; 32], Vec<u32>, ahash::RandomState> =
                    HashMap::with_capacity_and_hasher(8, ahash::RandomState::new());
                let mut failed_or_skipped = Vec::new();

                for &node_idx in &group.nodes {
                    // Micro-check cancellation inside file iterations for tight responsiveness
                    if cancel.load(Ordering::Relaxed) {
                        return Err(());
                    }

                    let path = snapshot.get_full_path(node_idx);

                    // Throttle progress text updates slightly to avoid atomic cache-line bouncing
                    if node_idx % 10 == 0 {
                        progress.set_item(path.clone()); // Show currently hashed file in real-time
                    }

                    let &(expected_mod, expected_cre) =
                        expected_timestamps.get(&node_idx).unwrap_or(&(0, 0));

                    match hash_fn(&path, group.size, expected_mod, expected_cre) {
                        HashResult::Success(hash) => {
                            hash_subgroups.entry(hash).or_default().push(node_idx);
                        }
                        HashResult::Skipped => {
                            failed_or_skipped.push(node_idx);
                        }
                        HashResult::Error => {
                            // Completely discard target nodes containing structural or reading errors
                        }
                    }
                }

                // Subgroups with duplicates
                for (_, nodes) in hash_subgroups {
                    if nodes.len() >= 2 {
                        local_groups.push(DuplicateGroup {
                            size: group.size,
                            nodes,
                            file_ids: Vec::new(),
                        });
                    }
                }

                // Too small files that were skipped are grouped back together
                if failed_or_skipped.len() >= 2 {
                    local_groups.push(DuplicateGroup {
                        size: group.size,
                        nodes: failed_or_skipped,
                        file_ids: Vec::new(),
                    });
                }

                // Increment progress position safely across worker threads
                let current_progress = completed_groups.fetch_add(1, Ordering::Relaxed);
                if current_progress.is_multiple_of(5) || current_progress + 1 == total_groups {
                    progress.set_pos(current_progress as u64 + 1);
                }

                Ok(local_groups)
            })
            .collect(); // Propagates a short-circuit Err if canceled mid-scan

        // Flatten the thread-isolated vector chunks back down into a single lineage line
        match next_groups_res {
            Ok(chunks) => Some(chunks.into_iter().flatten().collect()),
            Err(()) => None,
        }
    };

    // --- Phase 2: Prefix Hashing ---
    let prefix_hash_fn = |path: &str, _size: u64, expected_mod: u32, expected_cre: u32| {
        calculate_hash_at_range(path, 0, HASH_BLOCK_SIZE, expected_mod, expected_cre)
            .map_or(HashResult::Error, HashResult::Success)
    };
    let Some(groups) =
        run_hashing_phase(current_groups, &prefix_hash_fn, &t!("dedup-phase2-prefix"))
    else {
        cancel_and_clear();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    update_results(current_groups.clone());

    // --- Phase 3: Midpoint Hashing ---
    let midpoint_hash_fn = |path: &str, size: u64, expected_mod: u32, expected_cre: u32| {
        if size <= (HASH_BLOCK_SIZE * 2) as u64 {
            return HashResult::Skipped;
        }
        let mid = size / 2;
        let start_offset = mid.saturating_sub(HASH_BLOCK_SIZE as u64 / 2);
        calculate_hash_at_range(
            path,
            start_offset,
            HASH_BLOCK_SIZE,
            expected_mod,
            expected_cre,
        )
        .map_or(HashResult::Error, HashResult::Success)
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &midpoint_hash_fn,
        &t!("dedup-phase3-midpoint"),
    ) else {
        cancel_and_clear();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    update_results(current_groups.clone());

    // --- Phase 4: Suffix Hashing ---
    let suffix_hash_fn = |path: &str, size: u64, expected_mod: u32, expected_cre: u32| {
        if size <= HASH_BLOCK_SIZE as u64 {
            return HashResult::Skipped;
        }
        let start_offset = size - HASH_BLOCK_SIZE as u64;
        calculate_hash_at_range(
            path,
            start_offset,
            HASH_BLOCK_SIZE,
            expected_mod,
            expected_cre,
        )
        .map_or(HashResult::Error, HashResult::Success)
    };
    let Some(groups) =
        run_hashing_phase(current_groups, &suffix_hash_fn, &t!("dedup-phase4-suffix"))
    else {
        cancel_and_clear();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    update_results(current_groups.clone());

    // --- Phase 5: Multi-Range Hashing ---
    let multi_range_hash_fn = |path: &str, size: u64, expected_mod: u32, expected_cre: u32| {
        if size < MULTI_RANGE_SPREAD_SIZE {
            return HashResult::Skipped;
        }
        calculate_multi_range_hash(path, size, expected_mod, expected_cre)
            .map_or(HashResult::Error, HashResult::Success)
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &multi_range_hash_fn,
        &t!("dedup-phase5-multirange"),
    ) else {
        cancel_and_clear();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    update_results(current_groups.clone());

    // --- Phase 6: Full Hashing ---
    let full_hash_fn = |path: &str, _size: u64, expected_mod: u32, expected_cre: u32| {
        calculate_full_hash(path, expected_mod, expected_cre)
            .map_or(HashResult::Error, HashResult::Success)
    };
    let Some(groups) = run_hashing_phase(current_groups, &full_hash_fn, &t!("dedup-phase6-full"))
    else {
        cancel_and_clear();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    update_results(current_groups.clone());

    // --- Phase 7: Validation ---
    let total_groups = current_groups.len();
    progress.set_name(t!("dedup-phase7-validation"));
    progress.set_total(total_groups as u64);
    progress.set_pos(0);

    let mut final_groups = Vec::new();

    for (grp_idx, group) in current_groups.into_iter().enumerate() {
        if is_cancelled() {
            cancel_and_clear();
            return;
        }

        progress.set_pos(grp_idx as u64);

        let mut validated_nodes = Vec::new();
        let mut validated_file_ids = Vec::new();

        for &node_idx in &group.nodes {
            let path = snapshot.get_full_path(node_idx);
            progress.set_item(path.clone());

            let &(expected_mod, expected_cre) =
                expected_timestamps.get(&node_idx).unwrap_or(&(0, 0));

            if let Some(meta) = std::fs::metadata(&path)
                .ok()
                .filter(|m| m.len() == group.size)
            {
                let modified_ok = meta.modified().map_or(true, |mod_time| {
                    mod_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(true, |duration| duration.as_secs() as u32 == expected_mod)
                });
                let created_ok = meta.created().map_or(true, |cre_time| {
                    cre_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(true, |duration| duration.as_secs() as u32 == expected_cre)
                });

                if modified_ok && created_ok {
                    let file_id = crate::engine::traversal::get_file_id(&meta);
                    validated_nodes.push(node_idx);
                    validated_file_ids.push(file_id);
                }
            }
        }

        if validated_nodes.len() >= 2 {
            final_groups.push(DuplicateGroup {
                size: group.size,
                nodes: validated_nodes,
                file_ids: validated_file_ids,
            });
        }
    }

    final_groups.sort_by_key(|g| std::cmp::Reverse(g.size));

    let final_groups_count = final_groups.len();
    let duration = start_time.elapsed();

    let reclaimable: u64 = final_groups
        .iter()
        .map(|g| {
            let unique_count = {
                let mut ids: Vec<(u64, u64)> = g
                    .file_ids
                    .iter()
                    .copied()
                    .filter(|&id| id != (0, 0))
                    .collect();
                ids.sort_unstable();
                ids.dedup();
                ids.len()
            };
            if unique_count > 0 {
                g.size * (unique_count as u64 - 1)
            } else {
                g.size * (g.nodes.len() as u64 - 1)
            }
        })
        .sum();

    let space_str = prettier_bytes::ByteFormatter::new()
        .format(reclaimable)
        .to_string();

    update_results(final_groups);

    progress.set_name(t!("dedup-phase-finished", {
        "duration" => format!("{duration:.2?}"),
        "count" => final_groups_count,
        "space" => space_str.as_str()
    }));
    progress.finish();
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use parking_lot::RwLock;

    use super::*;
    use crate::{
        arena::{FileArenaSnapshot, FileNode, NodeStorage, StringPool},
        coordinator::{Coordinator, SharedState},
        engine::traversal::TraversalEngine,
    };

    #[test]
    fn test_deduplication_with_hardlink() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_deduplicator");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean old
        std::fs::create_dir_all(&temp_dir)?;

        let file1_path = temp_dir.join("file1.txt");
        let file2_path = temp_dir.join("file2.txt");
        let duplicate_path = temp_dir.join("duplicate.txt");

        let content = b"hello identical content! hello identical content!";
        std::fs::write(&file1_path, content)?;
        // Create a hardlink
        std::fs::hard_link(&file1_path, &file2_path)?;
        // Create a regular duplicate file
        std::fs::write(&duplicate_path, content)?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new();
        let (tx, rx) = crossbeam::channel::unbounded();

        let handle = engine.start_traversal(temp_dir.clone(), false, tx)?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());

        let results = Arc::new(RwLock::new(DeduplicationResults::default()));
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = atomic_progress::Progress::new_spinner("Deduplicator");
        let config = DeduplicatorConfig {
            min_size: 1,
            ignore_system: false,
            ignore_hidden: false,
        };

        run_deduplication(snapshot.clone(), progress, results.clone(), cancel, config);

        let results_guard = results.read();
        let groups = &results_guard.groups;
        assert_eq!(groups.len(), 1);

        let group = &groups[0];

        assert_eq!(group.nodes.len(), 3);
        assert_eq!(group.file_ids.len(), 3);

        // Verify we have 2 unique inodes
        let mut unique_ids = group.file_ids.clone();

        drop(results_guard);
        unique_ids.sort_unstable();
        unique_ids.dedup();
        assert_eq!(unique_ids.len(), 2);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_is_excluded_path_system() {
        assert!(is_excluded_path("/etc/passwd", true, false));
        assert!(is_excluded_path(
            "C:\\System Volume Information\\test",
            true,
            false
        ));
        assert!(is_excluded_path("/var/log/syslog", true, false));
        assert!(!is_excluded_path("/etc/passwd", false, false));
    }

    #[test]
    fn test_is_excluded_path_hidden() {
        assert!(is_excluded_path("/home/tux/.git/config", false, true));
        assert!(is_excluded_path("C:\\foo\\.bar\\test", false, true));
        assert!(!is_excluded_path("/home/tux/.git/config", false, false));
        assert!(!is_excluded_path("/home/tux/normal/path", false, true));
    }

    #[test]
    fn test_is_excluded_path_none() {
        assert!(!is_excluded_path(
            "/home/tux/Documents/test.txt",
            true,
            true
        ));
    }

    #[test]
    fn test_rebuild_flat_rows_empty() {
        let pool = StringPool::new();
        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(vec![])),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };
        let mut results = DeduplicationResults::default();
        results.rebuild_flat_rows(&snapshot);
        assert!(results.flat_rows.is_empty());
    }

    #[test]
    fn test_rebuild_flat_rows_standard() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"/");
        let f1_id = pool.get_or_insert(b"file1.png");
        let f2_id = pool.get_or_insert(b"file2.png");

        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 10, 0),
            FileNode::new(f2_id, Some(0), false, false, 20, 0),
        ];

        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };

        let mut results = DeduplicationResults {
            groups: vec![DuplicateGroup {
                size: 1000,
                nodes: vec![1, 2],
                file_ids: vec![(0, 0), (0, 0)],
            }],
            flat_rows: vec![],
        };

        results.rebuild_flat_rows(&snapshot);
        assert_eq!(results.flat_rows.len(), 2);

        assert_eq!(results.flat_rows[0].node_idx, 1);
        assert!(results.flat_rows[0].is_original);
        assert_eq!(results.flat_rows[0].reclaimable_str, "1000 B");

        assert_eq!(results.flat_rows[1].node_idx, 2);
        assert!(!results.flat_rows[1].is_original);
        assert_eq!(results.flat_rows[1].reclaimable_str, "1000 B");
    }

    #[test]
    fn test_rebuild_flat_rows_hardlinks() {
        let mut pool = StringPool::new();
        let r_id = pool.get_or_insert(b"/");
        let f1_id = pool.get_or_insert(b"file1.png");
        let f2_id = pool.get_or_insert(b"file2.png");

        let nodes = vec![
            FileNode::new(r_id, None, true, false, 0, 0),
            FileNode::new(f1_id, Some(0), false, false, 10, 0),
            FileNode::new(f2_id, Some(0), false, false, 20, 0),
        ];

        let snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(pool),
            dir_counts: Arc::new(vec![]),
        };

        let mut results = DeduplicationResults {
            groups: vec![DuplicateGroup {
                size: 1000,
                nodes: vec![1, 2],
                file_ids: vec![(5, 5), (5, 5)],
            }],
            flat_rows: vec![],
        };

        results.rebuild_flat_rows(&snapshot);
        assert_eq!(results.flat_rows.len(), 2);

        assert_eq!(results.flat_rows[0].node_idx, 1);
        assert!(results.flat_rows[0].is_hardlink);
        assert_eq!(results.flat_rows[0].reclaimable_str, "0 B");

        assert_eq!(results.flat_rows[1].node_idx, 2);
        assert!(results.flat_rows[1].is_hardlink);
        assert_eq!(results.flat_rows[1].reclaimable_str, "0 B");
    }

    #[test]
    fn test_deduplication_cancellation_clears_results() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_deduplicator_cancel");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean old
        std::fs::create_dir_all(&temp_dir)?;

        let file1_path = temp_dir.join("file1.txt");
        let file2_path = temp_dir.join("file2.txt");

        let content = b"some identical content";
        std::fs::write(&file1_path, content)?;
        std::fs::write(&file2_path, content)?;

        let shared_state = Arc::new(SharedState::new());
        let engine = TraversalEngine::new();
        let (tx, rx) = crossbeam::channel::unbounded();

        let handle = engine.start_traversal(temp_dir.clone(), false, tx)?;
        let mut coordinator = Coordinator::new(rx, shared_state.clone());
        coordinator.run_coordinator_loop(&temp_dir.to_string_lossy());
        let _ = handle.join();

        let snapshot = shared_state.current_snapshot.load();
        assert!(!snapshot.nodes.is_empty());

        // Initialize with pre-existing results
        let results = Arc::new(RwLock::new(DeduplicationResults {
            groups: vec![DuplicateGroup {
                size: 10,
                nodes: vec![1, 2],
                file_ids: vec![(0, 0), (0, 0)],
            }],
            flat_rows: vec![],
        }));
        let cancel = Arc::new(AtomicBool::new(true)); // Cancelled immediately
        let progress = atomic_progress::Progress::new_spinner("Deduplicator");
        let config = DeduplicatorConfig {
            min_size: 1,
            ignore_system: false,
            ignore_hidden: false,
        };

        run_deduplication(snapshot.clone(), progress, results.clone(), cancel, config);

        // Results should be cleared
        let results_guard = results.read();
        let groups_is_empty = results_guard.groups.is_empty();
        let flat_rows_is_empty = results_guard.flat_rows.is_empty();
        drop(results_guard);

        assert!(groups_is_empty);
        assert!(flat_rows_is_empty);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
