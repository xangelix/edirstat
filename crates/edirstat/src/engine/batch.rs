//! What scan producers send the coordinator: whole directory listings, already in the arena
//! slots their producer reserved.
//!
//! A producer (a traversal worker, the MFT scan) reads a directory to the end, reserves one slot
//! per entry from a counter every producer shares ([`Slots`]), and sends the entries linked: each
//! one's `parent` is its directory's slot, its `next_sibling` the next slot of the listing. A
//! directory's own slot was reserved before anyone could read it, so children always sit after
//! their parent, and a listing can be placed whenever it arrives, in any order, with no
//! hand-off between producers. The coordinator copies nodes into place, links each directory to
//! its first child, and interns names: a batch carries each of its names once, with its hash.

use std::{
    collections::HashMap,
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
};

use compact_str::CompactString;
use crossbeam::channel::Sender;
use hashbrown::HashTable;

use crate::arena::{FileNode, NO_INDEX, StringId};
use edirstat_core::state::TraversalStats;

/// One directory's entries: the batch's next `len` nodes, for slots `first_slot..first_slot +
/// len`, children of the directory in slot `parent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listing {
    pub parent: u32,
    pub first_slot: u32,
    pub len: u32,
}

/// Listings sent to the coordinator in one message.
///
/// Every node's `parent` and `next_sibling` are arena slots; its `name_id` is the index of its
/// name in this batch, which the coordinator replaces with the interned one.
#[derive(Debug, Default)]
pub struct ScanBatch {
    pub listings: Vec<Listing>,
    pub nodes: Vec<FileNode>,
    /// The batch's names, each once, back to back.
    pub names: String,
    /// Where each name ends in `names`.
    pub name_ends: Vec<u32>,
    /// Each name's [`name_hash`].
    pub name_hashes: Vec<u64>,
    /// Slots of directories that could not be read for lack of permission.
    pub denied: Vec<u32>,
    /// Total size and number of the batch's files, by lowercase extension.
    pub extensions: Vec<(CompactString, u64, u32)>,
}

impl ScanBatch {
    /// The batch's names with their hashes, in order.
    pub fn names(&self) -> impl Iterator<Item = (&str, u64)> {
        let mut start = 0;
        self.name_ends
            .iter()
            .zip(&self.name_hashes)
            .map(move |(&end, &hash)| {
                let name = self.names.get(start..end as usize).unwrap_or_default();
                start = end as usize;
                (name, hash)
            })
    }
}

/// The hash a name is interned under, the same for every producer and the coordinator.
pub fn name_hash(name: &str) -> u64 {
    static HASHER: LazyLock<ahash::RandomState> = LazyLock::new(ahash::RandomState::new);
    HASHER.hash_one(name)
}

/// The arena slots of one scan. Slot 0 is the scan root.
///
/// 64 bits wide so that it never wraps: a wrapped slot could come before its parent.
#[derive(Debug)]
pub struct Slots(AtomicU64);

impl Default for Slots {
    fn default() -> Self {
        Self(AtomicU64::new(1))
    }
}

impl Slots {
    /// Reserves `len` consecutive slots, returning the first; `None` once the arena is full
    /// (`NO_INDEX` is never a slot).
    fn reserve(&self, len: u32) -> Option<u32> {
        let first = self.0.fetch_add(u64::from(len), Ordering::Relaxed);
        (first + u64::from(len) < u64::from(NO_INDEX)).then_some(first as u32)
    }
}

/// Nodes per batch sent.
const BATCH_NODES: usize = 8192;

/// Builds batches one listing at a time, with the progress and extension totals of what it
/// built since its last flush.
#[derive(Debug)]
pub struct BatchBuilder {
    batch: ScanBatch,
    /// The batch's names, by hash: a name met again is not sent again.
    batch_names: HashTable<u32>,
    /// First node of the listing being built.
    listing_start: usize,
    extensions: HashMap<CompactString, (u64, u32), ahash::RandomState>,
    files: usize,
    dirs: usize,
    bytes: usize,
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self {
            batch: Self::empty_batch(),
            batch_names: HashTable::with_capacity(BATCH_NODES),
            listing_start: 0,
            extensions: HashMap::default(),
            files: 0,
            dirs: 0,
            bytes: 0,
        }
    }
}

impl BatchBuilder {
    fn empty_batch() -> ScanBatch {
        ScanBatch {
            nodes: Vec::with_capacity(BATCH_NODES),
            name_ends: Vec::with_capacity(BATCH_NODES),
            name_hashes: Vec::with_capacity(BATCH_NODES),
            names: String::with_capacity(BATCH_NODES * 32),
            ..ScanBatch::default()
        }
    }

    /// Entries in the listing being built.
    #[must_use]
    pub const fn listing_len(&self) -> usize {
        self.batch.nodes.len() - self.listing_start
    }

    /// Whether the batch is full enough to send.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.batch.nodes.len() >= BATCH_NODES
    }

    /// Counts directories read.
    pub const fn add_dirs(&mut self, dirs: usize) {
        self.dirs += dirs;
    }

    /// The batch's index of `name`, added unless the batch has it.
    fn name_index(&mut self, name: &str) -> u32 {
        let hash = name_hash(name);
        let ScanBatch {
            names,
            name_ends,
            name_hashes,
            ..
        } = &mut self.batch;
        let found = self.batch_names.find(hash, |&index| {
            let end = name_ends[index as usize] as usize;
            let start = index
                .checked_sub(1)
                .map_or(0, |prev| name_ends[prev as usize] as usize);
            names.get(start..end) == Some(name)
        });
        if let Some(&index) = found {
            return index;
        }
        let index = name_ends.len() as u32;
        names.push_str(name);
        name_ends.push(names.len() as u32);
        name_hashes.push(hash);
        self.batch_names
            .insert_unique(hash, index, |&index| name_hashes[index as usize]);
        index
    }

    /// Adds an entry to the listing being built; its name, parent and sibling are filled in
    /// here and by [`Self::finish_listing`].
    pub fn push(&mut self, name: &str, mut node: FileNode) {
        node.name_id = StringId(self.name_index(name));
        if !node.is_directory() {
            self.files += 1;
            self.bytes = self
                .bytes
                .saturating_add(usize::try_from(node.size).unwrap_or(usize::MAX));
            let ext = crate::arena::get_ext_slice(name);
            crate::arena::with_lowercase_ext(ext, |ext| {
                if let Some(total) = self.extensions.get_mut(ext) {
                    total.0 += node.size;
                    total.1 += 1;
                } else {
                    self.extensions
                        .insert(CompactString::new(ext), (node.size, 1));
                }
            });
        }
        self.batch.nodes.push(node);
    }

    /// Closes the listing being built as the content of the directory in slot `parent`,
    /// reserving its slots: returns the first, or `None` for an empty listing (or a full
    /// arena, when the listing is dropped).
    pub fn finish_listing(&mut self, parent: u32, slots: &Slots) -> Option<u32> {
        let len = self.listing_len();
        if len == 0 {
            return None;
        }
        let Some(first_slot) = u32::try_from(len).ok().and_then(|len| slots.reserve(len)) else {
            self.discard_listing();
            return None;
        };
        let last = self.batch.nodes.len() - 1;
        for (index, node) in self.batch.nodes[self.listing_start..]
            .iter_mut()
            .enumerate()
        {
            node.parent = parent;
            node.next_sibling = if self.listing_start + index == last {
                NO_INDEX
            } else {
                first_slot + index as u32 + 1
            };
        }
        self.batch.listings.push(Listing {
            parent,
            first_slot,
            len: len as u32,
        });
        self.listing_start = self.batch.nodes.len();
        Some(first_slot)
    }

    /// Drops the listing being built (an interrupted directory). Its names stay in the batch,
    /// unused.
    pub fn discard_listing(&mut self) {
        for node in &self.batch.nodes[self.listing_start..] {
            if !node.is_directory() {
                self.files -= 1;
                self.bytes = self
                    .bytes
                    .saturating_sub(usize::try_from(node.size).unwrap_or(usize::MAX));
            }
        }
        self.batch.nodes.truncate(self.listing_start);
    }

    /// Marks the directory in `slot` as unreadable for lack of permission.
    pub fn deny(&mut self, slot: u32) {
        self.batch.denied.push(slot);
    }

    /// Sends what was built (every listing closed so far) and adds its progress to `stats`.
    pub fn flush(&mut self, tx: &Sender<ScanBatch>, stats: &TraversalStats) {
        debug_assert_eq!(self.listing_len(), 0, "a batch is flushed between listings");
        if self.files + self.dirs > 0 {
            stats.files_scanned.fetch_add(self.files, Ordering::Relaxed);
            stats.dirs_scanned.fetch_add(self.dirs, Ordering::Relaxed);
            stats.bytes_scanned.fetch_add(self.bytes, Ordering::Relaxed);
            (self.files, self.dirs, self.bytes) = (0, 0, 0);
        }
        if self.batch.listings.is_empty() && self.batch.denied.is_empty() {
            return;
        }
        let mut batch = std::mem::replace(&mut self.batch, Self::empty_batch());
        batch.extensions = self
            .extensions
            .drain()
            .map(|(ext, (size, files))| (ext, size, files))
            .collect();
        self.batch_names.clear();
        self.listing_start = 0;
        let _ = tx.send(batch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(size: u64) -> FileNode {
        let mut node = FileNode::new(StringId(0), None, false, false, 0, 0);
        node.size = size;
        node
    }

    fn flushed(builder: &mut BatchBuilder, stats: &TraversalStats) -> ScanBatch {
        let (tx, rx) = crossbeam::channel::unbounded();
        builder.flush(&tx, stats);
        rx.try_recv().unwrap_or_default()
    }

    #[test]
    fn test_listing_links_siblings_and_parent() {
        let slots = Slots::default();
        let mut builder = BatchBuilder::default();
        builder.push("a.txt", file(1));
        builder.push("b.TXT", file(2));
        builder.push("c", FileNode::new(StringId(0), None, true, false, 0, 0));
        assert_eq!(builder.finish_listing(0, &slots), Some(1));

        let stats = TraversalStats::default();
        builder.add_dirs(1);
        let batch = flushed(&mut builder, &stats);

        assert_eq!(
            batch.listings,
            [Listing {
                parent: 0,
                first_slot: 1,
                len: 3
            }]
        );
        let links: Vec<_> = batch
            .nodes
            .iter()
            .map(|n| (n.parent, n.next_sibling, n.name_id.0))
            .collect();
        assert_eq!(links, [(0, 2, 0), (0, 3, 1), (0, NO_INDEX, 2)]);
        let names: Vec<_> = batch.names().map(|(name, _)| name).collect();
        assert_eq!(names, ["a.txt", "b.TXT", "c"]);
        assert_eq!(batch.extensions, [(CompactString::new("txt"), 3, 2)]);
        assert_eq!(stats.files_scanned.load(Ordering::Relaxed), 2);
        assert_eq!(stats.dirs_scanned.load(Ordering::Relaxed), 1);
        assert_eq!(stats.bytes_scanned.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_batch_sends_each_name_once() {
        let slots = Slots::default();
        let mut builder = BatchBuilder::default();
        builder.push("index.js", file(1));
        builder.push("README.md", file(1));
        assert_eq!(builder.finish_listing(0, &slots), Some(1));
        builder.push("README.md", file(1));
        builder.push("index.js", file(1));
        assert_eq!(builder.finish_listing(1, &slots), Some(3));
        let batch = flushed(&mut builder, &TraversalStats::default());

        let ids: Vec<_> = batch.nodes.iter().map(|n| n.name_id.0).collect();
        assert_eq!(ids, [0, 1, 1, 0]);
        assert_eq!(batch.names, "index.jsREADME.md");

        // A new batch starts afresh.
        builder.push("README.md", file(1));
        assert_eq!(builder.finish_listing(0, &slots), Some(5));
        let batch = flushed(&mut builder, &TraversalStats::default());
        assert_eq!(batch.names, "README.md");
        assert_eq!(batch.nodes[0].name_id.0, 0);
    }

    #[test]
    fn test_empty_and_discarded_listings_reserve_nothing() {
        let slots = Slots::default();
        let mut builder = BatchBuilder::default();
        assert_eq!(builder.finish_listing(0, &slots), None);
        builder.push("x", file(5));
        builder.discard_listing();
        assert_eq!(builder.listing_len(), 0);
        builder.push("y", file(7));
        assert_eq!(builder.finish_listing(0, &slots), Some(1));

        let stats = TraversalStats::default();
        let batch = flushed(&mut builder, &stats);
        assert_eq!(batch.nodes.len(), 1);
        let names: Vec<_> = batch.names().map(|(name, _)| name).collect();
        assert_eq!(names[batch.nodes[0].name_id.0 as usize], "y");
        assert_eq!(stats.bytes_scanned.load(Ordering::Relaxed), 7);
    }

    #[test]
    fn test_progress_without_listings_is_counted() {
        let mut builder = BatchBuilder::default();
        builder.add_dirs(3);
        let stats = TraversalStats::default();
        let (tx, rx) = crossbeam::channel::unbounded();
        builder.flush(&tx, &stats);
        assert!(rx.try_recv().is_err(), "nothing to send");
        assert_eq!(stats.dirs_scanned.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_slots_never_reach_no_index() {
        let slots = Slots(AtomicU64::new(u64::from(NO_INDEX) - 2));
        assert_eq!(slots.reserve(1), Some(NO_INDEX - 2));
        assert_eq!(slots.reserve(1), None);
    }

    #[test]
    fn test_name_hash_is_stable_within_a_process() {
        assert_eq!(name_hash("Cargo.toml"), name_hash("Cargo.toml"));
        assert_ne!(name_hash("Cargo.toml"), name_hash("cargo.toml"));
    }
}
