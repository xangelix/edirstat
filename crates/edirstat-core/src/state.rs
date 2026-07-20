use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use arc_swap::ArcSwap;
use compact_str::CompactString;

use crate::arena::{FileArenaSnapshot, NodeStorage, StringPool};

/// Live counters describing scan progress, shared between the traversal engine
/// and any frontends displaying them.
#[derive(Debug, Clone, Default)]
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

#[derive(Debug)]
pub struct SharedState {
    /// Atomic pointer to the latest immutable snapshot of the tree
    pub current_snapshot: ArcSwap<FileArenaSnapshot>,
    /// Indicates whether the scanner is actively running
    pub is_scanning: Arc<AtomicBool>,
    /// Background-computed live extension statistics (ext, `total_size`, `file_count`)
    pub extension_stats: ArcSwap<Vec<(CompactString, u64, u32)>>,
    /// Live scan progress counters (files/dirs/bytes scanned)
    pub scan_stats: TraversalStats,
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedState {
    #[must_use]
    pub fn new() -> Self {
        let initial_snapshot = FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(Vec::new())),
            string_pool: Arc::new(StringPool::new()),
            dir_counts: Arc::new(Vec::new()),
        };
        Self {
            current_snapshot: ArcSwap::new(Arc::new(initial_snapshot)),
            is_scanning: Arc::new(AtomicBool::new(false)),
            extension_stats: ArcSwap::new(Arc::new(Vec::new())),
            scan_stats: TraversalStats::default(),
        }
    }

    /// Publish a new immutable snapshot atomically.
    pub fn store_snapshot(&self, snapshot: FileArenaSnapshot) {
        self.current_snapshot.store(Arc::new(snapshot));
    }
}
