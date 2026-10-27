use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use compact_str::CompactString;
use crossbeam::channel::Receiver;
use hashbrown::HashTable;

pub use edirstat_core::state::SharedState;

use super::batch::{BatchNames, SHARD_SHIFT, ScanBatch, split_name_id};
use crate::arena::{FileArenaSnapshot, FileNode, NO_INDEX, NodeStorage, StringId, StringPool};

pub struct Coordinator {
    /// Lock-free channel to receive batches
    event_rx: Receiver<ScanBatch>,
    /// Shared state wrapper for swapping snapshots
    shared_state: Arc<SharedState>,
}

impl Coordinator {
    pub const fn new(event_rx: Receiver<ScanBatch>, shared_state: Arc<SharedState>) -> Self {
        Self {
            event_rx,
            shared_state,
        }
    }

    pub fn run_coordinator_loop(&mut self, root_path_str: &str) {
        self.shared_state.is_scanning.store(true, Ordering::SeqCst);

        let mut tree = Tree::new(root_path_str);
        let mut last_publish = Instant::now();
        let mut publish_interval = Duration::from_millis(100);

        while let Ok(batch) = self.event_rx.recv() {
            tree.place(batch);

            // Publish snapshot if dirty and scaled interval elapsed
            if tree.dirty && last_publish.elapsed() >= publish_interval {
                let current_size = tree.arena.len();

                // Dynamically scale updates to prevent main thread rendering stutter
                publish_interval = if current_size > 500_000 {
                    Duration::from_millis(1000)
                } else if current_size > 100_000 {
                    Duration::from_millis(500)
                } else {
                    Duration::from_millis(100)
                };

                self.shared_state.store_snapshot(tree.snapshot());
                self.shared_state
                    .extension_stats
                    .store(Arc::new(tree.extension_stats()));

                last_publish = Instant::now();
                tree.dirty = false;
            }
        }

        let cancelled = self.shared_state.scan_cancel.load(Ordering::SeqCst);
        if cancelled {
            // Throw away all data. Store an empty snapshot!
            let empty_snapshot = FileArenaSnapshot {
                nodes: Arc::new(NodeStorage::Owned(Vec::new())),
                string_pool: Arc::new(StringPool::new()),
                dir_counts: Arc::new(Vec::new()),
                extensions: crate::extensions::ExtensionStore::default(),
            };
            self.shared_state.store_snapshot(empty_snapshot);
            self.shared_state
                .extension_stats
                .store(Arc::new(Vec::new()));
            self.shared_state.scan_stats.reset();
        } else {
            let extension_stats = tree.extension_stats();
            self.shared_state.store_snapshot(tree.finish());
            self.shared_state
                .extension_stats
                .store(Arc::new(extension_stats));
        }

        self.shared_state.is_scanning.store(false, Ordering::SeqCst);
    }
}

/// The tree being built: listings placed in the slots their producers reserved.
///
/// Names are interned off this thread ([`Names`]): a node holds where its name was sent until a
/// snapshot looks up the interned id.
struct Tree {
    arena: Vec<FileNode>,
    names: Names,
    /// Scratch: how many names each shard had before the batch being placed.
    bases: Vec<u32>,
    extensions: std::collections::HashMap<CompactString, (u64, u32), ahash::RandomState>,
    /// Every slot below this one is filled.
    filled: usize,
    dirty: bool,
}

/// A slot reserved but not filled yet. Only the root has no parent, and only placeholders
/// carry anything into a fill: the first child and flags of a directory whose listing, or
/// whose permission failure, came first.
const fn is_placeholder(slot: usize, node: &FileNode) -> bool {
    slot != 0 && node.parent == NO_INDEX
}

impl Tree {
    fn new(root_path_str: &str) -> Self {
        let mut arena = Vec::with_capacity(1024 * 1024); // Pre-allocate space for ~1M nodes
        arena.push(FileNode::new(StringId(ROOT_PLACE), None, true, false, 0, 0));
        Self {
            arena,
            names: Names::start(root_path_str),
            bases: Vec::new(),
            extensions: std::collections::HashMap::default(),
            filled: 1,
            dirty: false,
        }
    }

    fn grow_to(&mut self, len: usize) {
        if self.arena.len() < len {
            self.arena
                .resize(len, FileNode::new(StringId(0), None, false, false, 0, 0));
        }
    }

    /// Places a batch's listings, marks its unreadable directories, and adds its extension
    /// totals.
    fn place(&mut self, batch: ScanBatch) {
        let ScanBatch {
            listings,
            nodes,
            names,
            denied,
            extensions,
        } = batch;
        let mut bases = std::mem::take(&mut self.bases);
        self.names.send(names, &mut bases);

        let mut nodes = nodes.iter();
        for listing in &listings {
            let first = listing.first_slot as usize;
            let end = first + listing.len as usize;
            self.grow_to(end);
            for (index, (slot, node)) in self.arena[first..end]
                .iter_mut()
                .zip(nodes.by_ref())
                .enumerate()
            {
                let pending = *slot;
                *slot = *node;
                slot.parent = listing.parent;
                slot.next_sibling = if first + index + 1 == end {
                    NO_INDEX
                } else {
                    (first + index + 1) as u32
                };
                let (shard, index) = split_name_id(node.name_id);
                slot.name_id = StringId(
                    ((shard as u32) << SHARD_SHIFT)
                        | (bases.get(shard).copied().unwrap_or(0) + index as u32),
                );
                slot.first_child = pending.first_child;
                slot.flags |= pending.flags;
            }
            let parent = listing.parent as usize;
            self.grow_to(parent + 1);
            self.arena[parent].first_child = listing.first_slot;
        }

        self.bases = bases;

        for &slot in &denied {
            self.grow_to(slot as usize + 1);
            self.arena[slot as usize].flags |= FileNode::FLAG_NO_PERMISSION;
        }

        for (ext, size, files) in extensions {
            let total = self.extensions.entry(ext).or_insert((0, 0));
            total.0 += size;
            total.1 += files;
        }
        self.dirty = true;
    }

    /// The tree so far: every slot up to the first unfilled one, sizes propagated. Links into
    /// the unfilled part are cut: those directories look empty until their listing arrives.
    fn snapshot(&mut self) -> FileArenaSnapshot {
        while self.filled < self.arena.len()
            && !is_placeholder(self.filled, &self.arena[self.filled])
        {
            self.filled += 1;
        }
        let filled = self.filled as u32;
        // Every name these nodes hold was sent before this request.
        let (string_pool, resolver) = self.names.snapshot();
        let mut nodes: Vec<FileNode> = self.arena[..self.filled]
            .iter()
            .map(|node| {
                let mut node = *node;
                if node.first_child >= filled {
                    node.first_child = NO_INDEX;
                }
                if node.next_sibling >= filled {
                    node.next_sibling = NO_INDEX;
                }
                node
            })
            .collect();
        let dir_counts = propagate_all_sizes_bottom_up(&mut nodes, |id| resolver.resolve(id));
        FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(nodes)),
            string_pool: Arc::new(string_pool),
            dir_counts: Arc::new(dir_counts),
            extensions: crate::extensions::ExtensionStore::default(),
        }
    }

    /// The finished tree, once every producer is done: then every reserved slot is filled.
    fn finish(self) -> FileArenaSnapshot {
        debug_assert!(
            self.arena
                .iter()
                .enumerate()
                .all(|(slot, node)| !is_placeholder(slot, node)),
            "every reserved slot is filled at the end of a scan"
        );
        // The names are copied out while the sizes are summed.
        let Self {
            mut arena, names, ..
        } = self;
        let (parts, resolver) = names.finish();
        let (string_pool, dir_counts) = std::thread::scope(|scope| {
            let pool = scope.spawn(move || parts.into_pool());
            resolve_names(&mut arena, &resolver);
            let dir_counts = propagate_all_sizes_bottom_up(&mut arena, |id| id);
            (pool.join().unwrap_or_default(), dir_counts)
        });
        FileArenaSnapshot {
            nodes: Arc::new(NodeStorage::Owned(arena)),
            string_pool: Arc::new(string_pool),
            dir_counts: Arc::new(dir_counts),
            extensions: crate::extensions::ExtensionStore::default(),
        }
    }

    /// (ext, total size, file count), sorted descending by total size.
    fn extension_stats(&self) -> Vec<(CompactString, u64, u32)> {
        let mut stats: Vec<(CompactString, u64, u32)> = self
            .extensions
            .iter()
            .map(|(ext, &(total_size, file_count))| (ext.clone(), total_size, file_count))
            .collect();
        stats.sort_by_key(|b| std::cmp::Reverse(b.1));
        stats
    }
}

/// Where a name was sent: its shard above `SHARD_SHIFT`, its index among that shard's names
/// below. The root's name stays out of the shards: it is the pool's first.
const ROOT_PLACE: u32 = u32::MAX;

/// Names, split by hash among threads that each intern their part ([`super::batch::name_shards`]),
/// fed in the order batches are placed. The pool a snapshot gets is the root's name, then each
/// shard's in turn, so ids stay dense and every name appears once.
struct Names {
    root: String,
    shards: Vec<NameShard>,
}

/// One interning thread.
struct NameShard {
    tx: Option<crossbeam::channel::Sender<NamesMessage>>,
    thread: Option<std::thread::JoinHandle<(NamePool, Vec<u32>)>>,
    /// Names sent to this shard so far.
    sent: u32,
}

enum NamesMessage {
    Batch {
        names: String,
        ends: Vec<u32>,
        hashes: Vec<u64>,
    },
    /// The shard's names so far (as a pool's arena and offsets) and each one's id in them.
    Snapshot(crossbeam::channel::Sender<(String, Vec<u32>, Vec<u32>)>),
}

/// Each shard's names laid out after the root's, and the id of every name sent.
struct Resolver {
    /// Where each shard's ids start in the pool.
    bases: Vec<u32>,
    /// Each shard's id of each name sent to it.
    interned: Vec<Vec<u32>>,
}

impl Resolver {
    fn resolve(&self, place: StringId) -> StringId {
        if place.0 == ROOT_PLACE {
            return StringId(0);
        }
        let shard = (place.0 >> SHARD_SHIFT) as usize;
        let index = (place.0 & ((1 << SHARD_SHIFT) - 1)) as usize;
        let id = self
            .interned
            .get(shard)
            .and_then(|ids| ids.get(index))
            .copied()
            .unwrap_or(0);
        StringId(self.bases.get(shard).copied().unwrap_or(0) + id)
    }
}

/// The root's name and each shard's names and ids, to be laid out as one pool.
struct NameParts {
    root: String,
    shards: Vec<(String, Vec<u32>)>,
}

impl NameParts {
    /// Where each shard's ids start in the pool, and whether it fits there (a pool holds
    /// 4 GiB of names).
    fn bases(&self) -> Vec<(u32, bool)> {
        let (mut bytes, mut ids) = (self.root.len(), 1u32);
        self.shards
            .iter()
            .map(|(shard_bytes, shard_offsets)| {
                let base = ids;
                let fits = u32::try_from(bytes + shard_bytes.len()).is_ok();
                if fits {
                    bytes += shard_bytes.len();
                    ids += shard_offsets.len().saturating_sub(1) as u32;
                }
                (base, fits)
            })
            .collect()
    }

    /// The pool's arena and offsets: the root's name, then each shard's; and where each
    /// shard's ids start.
    fn layout(&self) -> (String, Vec<u32>, Vec<u32>) {
        let placed = self.bases();
        let total = self.root.len()
            + self
                .shards
                .iter()
                .zip(&placed)
                .filter(|(_, (_, fits))| *fits)
                .map(|((shard_bytes, _), _)| shard_bytes.len())
                .sum::<usize>();
        let mut bytes = String::with_capacity(total);
        let mut offsets = Vec::with_capacity(
            2 + self
                .shards
                .iter()
                .map(|(_, shard_offsets)| shard_offsets.len())
                .sum::<usize>(),
        );
        bytes.push_str(&self.root);
        offsets.extend([0, self.root.len() as u32]);
        for ((shard_bytes, shard_offsets), &(_, fits)) in self.shards.iter().zip(&placed) {
            if !fits {
                continue;
            }
            let start = bytes.len() as u32;
            bytes.push_str(shard_bytes);
            offsets.extend(shard_offsets.iter().skip(1).map(|&end| start + end));
        }
        (
            bytes,
            offsets,
            placed.into_iter().map(|(base, _)| base).collect(),
        )
    }

    fn into_pool(self) -> StringPool {
        let (bytes, offsets, _) = self.layout();
        StringPool::frozen(Arc::from(bytes), offsets)
    }
}

impl Names {
    fn start(root: &str) -> Self {
        Self::with_shards(root, super::batch::name_shards())
    }

    fn with_shards(root: &str, threads: usize) -> Self {
        let shards = (0..threads)
            .map(|_| {
                let (tx, rx) = crossbeam::channel::unbounded::<NamesMessage>();
                let thread = std::thread::spawn(move || {
                    let mut pool = NamePool::with_capacity(NAMES_RESERVED / threads);
                    let mut interned = Vec::with_capacity(NAMES_RESERVED / threads);
                    while let Ok(message) = rx.recv() {
                        match message {
                            NamesMessage::Batch {
                                names,
                                ends,
                                hashes,
                            } => {
                                let mut start = 0;
                                for (&end, &hash) in ends.iter().zip(&hashes) {
                                    let name = names.get(start..end as usize).unwrap_or_default();
                                    interned.push(pool.intern(name, hash));
                                    start = end as usize;
                                }
                            }
                            NamesMessage::Snapshot(reply) => {
                                let _ = reply.send((
                                    pool.bytes.clone(),
                                    pool.offsets.clone(),
                                    interned.clone(),
                                ));
                            }
                        }
                    }
                    (pool, interned)
                });
                NameShard {
                    tx: Some(tx),
                    thread: Some(thread),
                    sent: 0,
                }
            })
            .collect();
        Self {
            root: root.to_owned(),
            shards,
        }
    }

    /// Hands each shard its part of a batch's names; `bases` gets how many names each shard
    /// had before.
    fn send(&mut self, names: Vec<BatchNames>, bases: &mut Vec<u32>) {
        bases.clear();
        for (shard, part) in self.shards.iter_mut().zip(names) {
            bases.push(shard.sent);
            if part.ends.is_empty() {
                continue;
            }
            shard.sent = shard.sent.saturating_add(part.ends.len() as u32);
            if let Some(tx) = &shard.tx {
                let _ = tx.send(NamesMessage::Batch {
                    names: part.names,
                    ends: part.ends,
                    hashes: part.hashes,
                });
            }
        }
    }

    /// Every name sent so far, as a pool, and how to find each one's id in it.
    fn snapshot(&self) -> (StringPool, Resolver) {
        let replies: Vec<_> = self
            .shards
            .iter()
            .map(|shard| {
                let (reply, answer) = crossbeam::channel::bounded(1);
                if let Some(tx) = &shard.tx {
                    let _ = tx.send(NamesMessage::Snapshot(reply));
                }
                answer
            })
            .collect();
        let mut parts = NameParts {
            root: self.root.clone(),
            shards: Vec::with_capacity(replies.len()),
        };
        let mut interned = Vec::with_capacity(replies.len());
        for answer in replies {
            let (bytes, offsets, ids) = answer.recv().unwrap_or_default();
            parts.shards.push((bytes, offsets));
            interned.push(ids);
        }
        let (bytes, offsets, bases) = parts.layout();
        (
            StringPool::frozen(Arc::from(bytes), offsets),
            Resolver { bases, interned },
        )
    }

    /// Every name, once the threads are done with them, and how to find each one's id.
    fn finish(self) -> (NameParts, Resolver) {
        let mut parts = NameParts {
            root: self.root,
            shards: Vec::with_capacity(self.shards.len()),
        };
        let mut interned = Vec::with_capacity(self.shards.len());
        for mut shard in self.shards {
            shard.tx = None;
            let (pool, ids) = shard
                .thread
                .take()
                .and_then(|thread| thread.join().ok())
                .unwrap_or_default();
            parts.shards.push((pool.bytes, pool.offsets));
            interned.push(ids);
        }
        let bases = parts.bases().into_iter().map(|(base, _)| base).collect();
        (parts, Resolver { bases, interned })
    }
}

/// Names interned on the hashes their producers computed, laid out as a frozen
/// [`StringPool`]: name `i` is `bytes[offsets[i]..offsets[i + 1]]`.
struct NamePool {
    table: HashTable<u32>,
    hashes: Vec<u64>,
    bytes: String,
    offsets: Vec<u32>,
}

/// Names a scan's pools make room for up front, between them, so that a large scan does not
/// rehash every name on its way there.
const NAMES_RESERVED: usize = 1 << 16;

impl Default for NamePool {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl NamePool {
    fn with_capacity(names: usize) -> Self {
        Self {
            table: HashTable::with_capacity(names),
            hashes: Vec::with_capacity(names),
            bytes: String::with_capacity(names * 16),
            offsets: Vec::from([0]),
        }
    }

    fn get(&self, id: u32) -> &str {
        let id = id as usize;
        let (Some(&start), Some(&end)) = (self.offsets.get(id), self.offsets.get(id + 1)) else {
            return "";
        };
        self.bytes
            .get(start as usize..end as usize)
            .unwrap_or_default()
    }

    /// The id of `name`, whose [`super::batch::name_hash`] is `hash`: the existing one, or a
    /// new one (0, the root's, once 4 GiB of names would overflow the offsets).
    fn intern(&mut self, name: &str, hash: u64) -> u32 {
        if let Some(&id) = self.table.find(hash, |&id| self.get(id) == name) {
            return id;
        }
        let Some(end) = u32::try_from(self.bytes.len() + name.len())
            .ok()
            .filter(|_| self.hashes.len() < NO_INDEX as usize)
        else {
            return 0;
        };
        let id = self.hashes.len() as u32;
        self.bytes.push_str(name);
        self.offsets.push(end);
        self.hashes.push(hash);
        let hashes = &self.hashes;
        self.table
            .insert_unique(hash, id, |&id| hashes[id as usize]);
        id
    }
}

/// Gives every node its name's id, on several threads: nodes do not depend on one another.
fn resolve_names(arena: &mut [FileNode], resolver: &Resolver) {
    let threads = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(16);
    let chunk = arena.len().div_ceil(threads).max(1 << 16);
    std::thread::scope(|scope| {
        for part in arena.chunks_mut(chunk) {
            scope.spawn(move || {
                for node in part {
                    node.name_id = resolver.resolve(node.name_id);
                }
            });
        }
    });
}

/// Accumulates file sizes, counts, and latest timestamps from leaf nodes up
/// to the root in a single, cache-friendly, reverse O(N) sweep, which also counts each
/// directory's subdirectories (recursively). Children come after their parent in the arena.
fn propagate_all_sizes_bottom_up(
    arena: &mut [FileNode],
    name: impl Fn(StringId) -> StringId,
) -> Vec<u32> {
    let mut dir_counts = vec![0u32; arena.len()];
    for idx in (0..arena.len()).rev() {
        arena[idx].name_id = name(arena[idx].name_id);
        let parent = arena[idx].parent;
        if parent != NO_INDEX && (parent as usize) < idx {
            let parent_idx = parent as usize;
            let size = arena[idx].size;
            let file_count = if arena[idx].is_directory() {
                dir_counts[parent_idx] += 1 + dir_counts[idx];
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
    dir_counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::batch::{BatchBuilder, Slots};
    use crate::traversal::TraversalStats;

    fn file(size: u64, modified: u32) -> FileNode {
        let mut node = FileNode::new(StringId(0), None, false, false, modified, 0);
        node.size = size;
        node
    }

    fn dir() -> FileNode {
        FileNode::new(StringId(0), None, true, false, 0, 0)
    }

    /// Lists `entries` as the content of `parent`, returning the batch and the first slot.
    fn listing(slots: &Slots, parent: u32, entries: &[(&str, FileNode)]) -> (ScanBatch, u32) {
        let mut builder = BatchBuilder::default();
        for (name, node) in entries {
            builder.push(name, *node);
        }
        let first = builder.finish_listing(parent, slots).unwrap_or(NO_INDEX);
        let (tx, rx) = crossbeam::channel::unbounded();
        builder.flush(&tx, &TraversalStats::default());
        (rx.try_recv().unwrap_or_default(), first)
    }

    fn run(batches: Vec<ScanBatch>) -> Arc<SharedState> {
        let shared = Arc::new(SharedState::new());
        let (tx, rx) = crossbeam::channel::unbounded();
        for batch in batches {
            let _ = tx.send(batch);
        }
        drop(tx);
        Coordinator::new(rx, shared.clone()).run_coordinator_loop("/root");
        shared
    }

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

        let dir_counts = propagate_all_sizes_bottom_up(&mut arena, |id| id);

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
        // Subdirectories, counted recursively
        assert_eq!(dir_counts, [1, 0, 0, 0]);
    }

    #[test]
    fn test_coordinator_builds_tree_from_listings() {
        let slots = Slots::default();
        let (root, first) = listing(&slots, 0, &[("d1", dir()), ("f2", file(36, 0))]);
        let (d1, _) = listing(&slots, first, &[("f1", file(64, 0))]);
        let shared = run(vec![root, d1]);

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 4);

        let root = &snapshot.nodes[0];
        assert!(root.is_directory());
        assert_eq!(root.size, 100);
        assert_eq!(root.file_count, 2);
        assert_eq!(root.first_child, 1);

        let d1 = &snapshot.nodes[1];
        assert_eq!(snapshot.string_pool.get(d1.name_id), Some("d1"));
        assert!(d1.is_directory());
        assert_eq!((d1.parent, d1.next_sibling, d1.first_child), (0, 2, 3));
        assert_eq!(d1.size, 64);
        assert_eq!(d1.file_count, 1);

        assert_eq!(snapshot.get_full_path(0), "/root");
        assert_eq!(snapshot.get_full_path(1), "/root/d1");
        assert_eq!(snapshot.get_full_path(2), "/root/f2");
        assert_eq!(snapshot.get_full_path(3), "/root/d1/f1");
        assert_eq!(snapshot.dir_counts.as_slice(), [1, 0, 0, 0]);
    }

    #[test]
    fn test_coordinator_places_listings_in_any_order() {
        // root <- a/ <- b/ <- c.txt; the deepest listing arrives first.
        let slots = Slots::default();
        let (root, a) = listing(&slots, 0, &[("a", dir())]);
        let (a_listing, b) = listing(&slots, a, &[("b", dir()), ("same", file(1, 0))]);
        let (b_listing, _) = listing(&slots, b, &[("c.txt", file(10, 0)), ("same", file(2, 0))]);
        let shared = run(vec![b_listing, root, a_listing]);

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 6);
        assert_eq!(snapshot.nodes[0].size, 13);
        assert_eq!(snapshot.get_full_path(4), "/root/a/b/c.txt");
        let a_node = &snapshot.nodes[a as usize];
        assert_eq!(a_node.first_child, b);
        assert_eq!(a_node.size, 13);
        // Names are interned across batches
        assert_eq!(snapshot.nodes[3].name_id, snapshot.nodes[5].name_id);
        assert_eq!(snapshot.dir_counts[0], 2);
    }

    #[test]
    fn test_coordinator_permission_denied_sets_flag() {
        let slots = Slots::default();
        let (root, d) = listing(&slots, 0, &[("d", dir())]);
        let mut builder = BatchBuilder::default();
        builder.deny(d);
        let (tx, rx) = crossbeam::channel::unbounded();
        builder.flush(&tx, &TraversalStats::default());
        let denied = rx.try_recv().unwrap_or_default();
        // The denial arrives before the directory's own listing: the flag must survive it.
        let shared = run(vec![denied, root]);

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 2);
        assert!(!snapshot.nodes[0].has_no_permission());
        assert_eq!(
            snapshot.string_pool.get(snapshot.nodes[1].name_id),
            Some("d")
        );
        assert!(snapshot.nodes[1].has_no_permission());
    }

    #[test]
    fn test_coordinator_extension_stats_merge_batches() {
        let slots = Slots::default();
        let (first, _) = listing(&slots, 0, &[("A.TXT", file(10, 0)), ("x.rs", file(1, 0))]);
        let (second, _) = listing(&slots, 0, &[("b.txt", file(20, 0))]);
        let shared = run(vec![first, second]);

        let stats = shared.extension_stats.load();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].0.as_str(), "txt");
        assert_eq!(stats[0].1, 30);
        assert_eq!(stats[0].2, 2);
        assert_eq!(stats[1].0.as_str(), "rs");
    }

    #[test]
    fn test_coordinator_populates_dataless_and_special_flags() {
        let slots = Slots::default();
        let mut cloud = file(1_000_000, 0);
        cloud.flags |= FileNode::FLAG_DATALESS;
        let mut socket = file(0, 0);
        socket.flags |= FileNode::FLAG_SPECIAL;
        let (batch, _) = listing(&slots, 0, &[("cloud.mp4", cloud), ("ipc.sock", socket)]);
        let shared = run(vec![batch]);

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.nodes.len(), 3);
        assert!(snapshot.nodes[1].is_dataless());
        assert!(!snapshot.nodes[1].is_special());
        assert!(!snapshot.nodes[2].is_dataless());
        assert!(snapshot.nodes[2].is_special());
    }

    /// A live snapshot holds only the slots up to the first unfilled one, with no link past it.
    #[test]
    fn test_snapshot_publishes_filled_prefix() {
        let slots = Slots::default();
        let (root, a) = listing(&slots, 0, &[("a", dir()), ("b", dir())]);
        let (a_listing, _) = listing(&slots, a, &[("in_a", file(5, 0))]);
        let (b_listing, _) = listing(&slots, a + 1, &[("in_b", file(7, 0))]);

        let mut tree = Tree::new("/root");
        tree.place(root);
        tree.place(b_listing);
        let snapshot = tree.snapshot();
        // Slot 3 (a's listing) is missing: b's content, in slot 4, is not shown yet.
        assert_eq!(snapshot.nodes.len(), 3);
        assert!(snapshot.nodes.iter().all(|n| {
            n.first_child == NO_INDEX || (n.first_child as usize) < snapshot.nodes.len()
        }));
        assert_eq!(snapshot.nodes[2].first_child, NO_INDEX);

        tree.place(a_listing);
        let snapshot = tree.snapshot();
        assert_eq!(snapshot.nodes.len(), 5);
        assert_eq!(snapshot.nodes[0].size, 12);
        assert_eq!(snapshot.dir_counts[0], 2);

        let finished = tree.finish();
        assert_eq!(finished.nodes.len(), 5);
        assert_eq!(
            finished.string_pool.get(finished.nodes[4].name_id),
            Some("in_b")
        );
    }

    #[test]
    fn test_name_pool_deduplicates() {
        let mut names = NamePool::default();
        let a = names.intern("a", crate::engine::batch::name_hash("a"));
        let b = names.intern("b", crate::engine::batch::name_hash("b"));
        assert_ne!(a, b);
        assert_eq!(names.intern("a", crate::engine::batch::name_hash("a")), a);
        assert_eq!(names.get(a), "a");
        assert_eq!(names.get(b), "b");
    }

    /// Names split among shards resolve to themselves, each name once, the root's first.
    #[test]
    fn test_sharded_names_resolve_densely() {
        // Split as producers split them: one shard per name, always the same.
        let batch = |words: &[String]| {
            let mut parts_by_shard: Vec<BatchNames> =
                (0..3).map(|_| BatchNames::default()).collect();
            let mut ids = Vec::new();
            for word in words {
                let hash = crate::engine::batch::name_hash(word);
                let shard = (hash >> 32) as usize % 3;
                let part = &mut parts_by_shard[shard];
                ids.push(((shard as u32) << SHARD_SHIFT) | part.ends.len() as u32);
                part.names.push_str(word);
                part.ends.push(part.names.len() as u32);
                part.hashes.push(hash);
            }
            (parts_by_shard, ids)
        };
        let first: Vec<String> = (0..500).map(|i| format!("name-{i}")).collect();
        // The second batch repeats half of the first.
        let second: Vec<String> = (250..750).map(|i| format!("name-{i}")).collect();

        let mut names = Names::with_shards("/root", 3);
        let mut bases = Vec::new();
        let mut sent = Vec::new();
        for words in [&first, &second] {
            let (parts_by_shard, ids) = batch(words);
            names.send(parts_by_shard, &mut bases);
            sent.extend(words.iter().cloned().zip(ids.iter().map(|&id| {
                let (shard, index) = split_name_id(StringId(id));
                ((shard as u32) << SHARD_SHIFT) | (bases[shard] + index as u32)
            })));
        }

        let (pool, resolver) = names.snapshot();
        assert_eq!(resolver.resolve(StringId(ROOT_PLACE)), StringId(0));
        assert_eq!(pool.get(StringId(0)), Some("/root"));
        for (word, place) in &sent {
            assert_eq!(
                pool.get(resolver.resolve(StringId(*place))),
                Some(word.as_str())
            );
        }

        let (parts, resolver) = names.finish();
        let pool = parts.into_pool();
        let mut ids: Vec<u32> = sent
            .iter()
            .map(|(word, place)| {
                let id = resolver.resolve(StringId(*place));
                assert_eq!(pool.get(id), Some(word.as_str()));
                id.0
            })
            .collect();
        ids.sort_unstable();
        ids.dedup();
        // 750 distinct names, in ids 1..=750 after the root's.
        assert_eq!(ids.len(), 750);
        assert_eq!((ids.first(), ids.last()), (Some(&1), Some(&750)));
    }
}
