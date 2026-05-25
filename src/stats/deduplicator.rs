use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub const HASH_BLOCK_SIZE: usize = 4096; // 4KB hashing block size
pub const MULTI_RANGE_SPREAD_SIZE: u64 = 100 * 1024 * 1024; // 100MB spread size for multi-range checks

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub size: u64,
    pub nodes: Vec<u32>,
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
    let path_lower = path.to_lowercase();

    if ignore_system {
        // System directory and file patterns
        if path_lower.contains("system volume information")
            || path_lower.contains("$recycle.bin")
            || path_lower.contains("windows/system32")
            || path_lower.contains("/etc/")
            || path_lower.contains("/var/")
            || path_lower.contains("/usr/")
            || path_lower.contains("/proc/")
            || path_lower.contains("/sys/")
            || path_lower.contains("/dev/")
            || path_lower.contains("swapfile")
            || path_lower.contains("pagefile.sys")
        {
            return true;
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
    expected_modified: i64,
    expected_created: i64,
) -> Option<[u8; 32]> {
    // 1. Verify metadata timestamps before reading
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_modified
    {
        return None; // modified since snapshot
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_created
    {
        return None; // created since snapshot
    }

    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(start_offset)).ok()?;

    let mut buffer = vec![0u8; len];
    let n = file.read(&mut buffer).ok()?;
    buffer.truncate(n);

    let hash = blake3::hash(&buffer);
    Some(hash.into())
}

/// Multi-range hashing spread across large files (every 100MB)
fn calculate_multi_range_hash(
    path: &str,
    file_size: u64,
    expected_modified: i64,
    expected_created: i64,
) -> Option<[u8; 32]> {
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_modified
    {
        return None;
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_created
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
    expected_modified: i64,
    expected_created: i64,
) -> Option<[u8; 32]> {
    let metadata = std::fs::metadata(path).ok()?;

    if let Ok(modified_time) = metadata.modified()
        && let Ok(duration) = modified_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_modified
    {
        return None;
    }

    if let Ok(created_time) = metadata.created()
        && let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH)
        && duration.as_secs() as i64 != expected_created
    {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunk buffer

    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Some(hasher.finalize().into())
}

pub type HashFn = dyn Fn(&str, u64, i64, i64) -> Option<[u8; 32]>;

/// Main execution routine of the background deduplication runner
#[allow(clippy::needless_pass_by_value)]
pub fn run_deduplication(
    snapshot: Arc<crate::arena::FileArenaSnapshot>,
    progress: atomic_progress::Progress,
    results: Arc<parking_lot::RwLock<Vec<DuplicateGroup>>>,
    cancel: Arc<AtomicBool>,
    config: DeduplicatorConfig,
) {
    let start_time = Instant::now();
    let is_cancelled = || cancel.load(Ordering::SeqCst);

    if is_cancelled() {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    }

    // Set state to SizeGrouping (Phase 1)
    progress.set_name("Phase 1/7: Grouping all scanned files by size...");
    progress.set_total(0); // Indeterminate spinner initially
    progress.set_pos(0);

    let mut expected_timestamps = HashMap::new();
    let mut size_groups: HashMap<u64, Vec<u32>> = HashMap::new();

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

        let path = snapshot.get_full_path(idx as u32);
        if is_excluded_path(&path, config.ignore_system, config.ignore_hidden) {
            continue;
        }

        // Periodically show filenames in the loader thread
        if idx % 1000 == 0 {
            progress.set_item(path.clone());
        }

        expected_timestamps.insert(
            idx as u32,
            (node.modified_timestamp, node.created_timestamp),
        );

        size_groups.entry(size).or_default().push(idx as u32);
    }

    if is_cancelled() {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    }

    // Filter size groups to those with duplicates (size >= 2)
    let mut current_groups: Vec<DuplicateGroup> = size_groups
        .into_iter()
        .filter(|(_, nodes)| nodes.len() >= 2)
        .map(|(size, nodes)| DuplicateGroup { size, nodes })
        .collect();

    // Sort descending by size to present largest first
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));

    (*results.write()).clone_from(&current_groups);

    // Generic helper to process candidate groups during each hashing phase
    let run_hashing_phase = |current_groups: Vec<DuplicateGroup>,
                             hash_fn: &HashFn,
                             phase_name: &str|
     -> Option<Vec<DuplicateGroup>> {
        let total_groups = current_groups.len();
        progress.set_name(phase_name);
        progress.set_total(total_groups as u64);
        progress.set_pos(0);

        let mut next_groups = Vec::new();

        for (grp_idx, group) in current_groups.into_iter().enumerate() {
            if is_cancelled() {
                return None;
            }

            progress.set_pos(grp_idx as u64);

            let mut hash_subgroups: HashMap<[u8; 32], Vec<u32>> = HashMap::new();
            let mut failed_or_skipped = Vec::new();

            for &node_idx in &group.nodes {
                let path = snapshot.get_full_path(node_idx);
                progress.set_item(path.clone()); // Show currently hashed file in real-time

                let &(expected_mod, expected_cre) =
                    expected_timestamps.get(&node_idx).unwrap_or(&(0, 0));

                if let Some(hash) = hash_fn(&path, group.size, expected_mod, expected_cre) {
                    hash_subgroups.entry(hash).or_default().push(node_idx);
                } else {
                    // Check if file is still there and unchanged on disk
                    if let Ok(meta) = std::fs::metadata(&path)
                        && meta.len() == group.size
                    {
                        failed_or_skipped.push(node_idx);
                    }
                }
            }

            // Subgroups with duplicates
            for (_, nodes) in hash_subgroups {
                if nodes.len() >= 2 {
                    next_groups.push(DuplicateGroup {
                        size: group.size,
                        nodes,
                    });
                }
            }

            // Too small files that were skipped are grouped back together
            if failed_or_skipped.len() >= 2 {
                next_groups.push(DuplicateGroup {
                    size: group.size,
                    nodes: failed_or_skipped,
                });
            }
        }

        Some(next_groups)
    };

    // --- Phase 2: Prefix Hashing ---
    let prefix_hash_fn = |path: &str, _size: u64, expected_mod: i64, expected_cre: i64| {
        calculate_hash_at_range(path, 0, HASH_BLOCK_SIZE, expected_mod, expected_cre)
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &prefix_hash_fn,
        "Phase 2/7: Hashing file prefixes (first 4KB)...",
    ) else {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    (*results.write()).clone_from(&current_groups);

    // --- Phase 3: Midpoint Hashing ---
    let midpoint_hash_fn = |path: &str, size: u64, expected_mod: i64, expected_cre: i64| {
        if size <= (HASH_BLOCK_SIZE * 2) as u64 {
            return None;
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
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &midpoint_hash_fn,
        "Phase 3/7: Hashing file midpoints...",
    ) else {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    (*results.write()).clone_from(&current_groups);

    // --- Phase 4: Suffix Hashing ---
    let suffix_hash_fn = |path: &str, size: u64, expected_mod: i64, expected_cre: i64| {
        if size <= HASH_BLOCK_SIZE as u64 {
            return None;
        }
        let start_offset = size - HASH_BLOCK_SIZE as u64;
        calculate_hash_at_range(
            path,
            start_offset,
            HASH_BLOCK_SIZE,
            expected_mod,
            expected_cre,
        )
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &suffix_hash_fn,
        "Phase 4/7: Hashing file suffixes...",
    ) else {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    (*results.write()).clone_from(&current_groups);

    // --- Phase 5: Multi-Range Hashing ---
    let multi_range_hash_fn = |path: &str, size: u64, expected_mod: i64, expected_cre: i64| {
        if size < MULTI_RANGE_SPREAD_SIZE {
            return None;
        }
        calculate_multi_range_hash(path, size, expected_mod, expected_cre)
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &multi_range_hash_fn,
        "Phase 5/7: Multi-range hashing large files...",
    ) else {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    (*results.write()).clone_from(&current_groups);

    // --- Phase 6: Full Hashing ---
    let full_hash_fn = |path: &str, _size: u64, expected_mod: i64, expected_cre: i64| {
        calculate_full_hash(path, expected_mod, expected_cre)
    };
    let Some(groups) = run_hashing_phase(
        current_groups,
        &full_hash_fn,
        "Phase 6/7: Full BLAKE3 hashing of remaining candidates...",
    ) else {
        progress.set_error(Some("Scan was cancelled."));
        progress.finish();
        return;
    };
    current_groups = groups;
    current_groups.sort_by_key(|g| std::cmp::Reverse(g.size));
    (*results.write()).clone_from(&current_groups);

    // --- Phase 7: Validation ---
    let total_groups = current_groups.len();
    progress.set_name("Phase 7/7: Final timestamp validation...");
    progress.set_total(total_groups as u64);
    progress.set_pos(0);

    let mut final_groups = Vec::new();

    for (grp_idx, group) in current_groups.into_iter().enumerate() {
        if is_cancelled() {
            progress.set_error(Some("Scan was cancelled."));
            progress.finish();
            return;
        }

        progress.set_pos(grp_idx as u64);

        let mut validated_nodes = Vec::new();

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
                        .map_or(true, |duration| duration.as_secs() as i64 == expected_mod)
                });
                let created_ok = meta.created().map_or(true, |cre_time| {
                    cre_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(true, |duration| duration.as_secs() as i64 == expected_cre)
                });

                if modified_ok && created_ok {
                    validated_nodes.push(node_idx);
                }
            }
        }

        if validated_nodes.len() >= 2 {
            final_groups.push(DuplicateGroup {
                size: group.size,
                nodes: validated_nodes,
            });
        }
    }

    final_groups.sort_by_key(|g| std::cmp::Reverse(g.size));

    let final_groups_count = final_groups.len();
    let duration = start_time.elapsed();

    let reclaimable: u64 = final_groups
        .iter()
        .map(|g| g.size * (g.nodes.len() as u64 - 1))
        .sum();

    let space_str = prettier_bytes::ByteFormatter::new()
        .format(reclaimable)
        .to_string();

    *results.write() = final_groups;

    progress.set_name(format!(
        "Finished in {duration:.2?}! Found {final_groups_count} duplicate groups. Potential reclaimable space: {space_str}"
    ));
    progress.finish();
}
