use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crossbeam::{
    channel::Sender,
    deque::{Injector, Steal, Stealer, Worker},
};

use super::batch::{BatchBuilder, ScanBatch, Slots};
use crate::arena::{FileNode, StringId};
pub use edirstat_core::{file_id::get_file_id, state::TraversalStats};

/// A batch waiting this long is sent after the directory being read, full or not, so that the
/// live tree keeps up with a slow volume.
const FLUSH_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct ScanTask {
    pub path: PathBuf,
    /// The directory's arena slot.
    pub slot: u32,
    /// The directory's own device (0 when unknown).
    pub device_id: u64,
    /// The directory's size as its parent's listing saw it (0 when unknown).
    pub dir_size: u64,
    /// Whether the directory is on btrfs, when known (it is once its parent was read on the
    /// same device).
    pub btrfs: Option<bool>,
    pub ancestors: smallvec::SmallVec<[(u64, u64); 16]>,
    /// The device/volume identifier to restrict traversal within.
    pub expected_device_id: Option<u64>,
    /// Allowed secondary device identifier (e.g. macOS APFS Data volume paired with System root).
    pub allowed_secondary_device_id: Option<u64>,
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
        event_tx: Sender<ScanBatch>,
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

            // Overlay filesystem mountpoints (e.g. a running container's
            // `overlay2/<id>/merged`) duplicate bytes already counted under
            // `diff/`; workers show them as empty nodes without descending.
            let overlay_mounts = super::mounts::load_overlay_mounts();

            // Build initial scan task
            let root_id = (0, 0); // Placeholder for root
            let root_metadata = fs::metadata(&root_path);
            let root_file_id = root_metadata
                .as_ref()
                .map_or(root_id, |m| get_file_id(&root_path, m));
            let (expected_device_id, allowed_secondary_device_id) = if same_filesystem {
                (
                    root_metadata
                        .as_ref()
                        .map(|m| get_device_id(&root_path, m))
                        .ok(),
                    get_secondary_device_id(&root_path),
                )
            } else {
                (None, None)
            };

            let initial_task = ScanTask {
                path: root_path.clone(),
                slot: 0,
                device_id: root_file_id.0,
                dir_size: root_metadata.as_ref().map_or(0, fs::Metadata::len),
                btrfs: None,
                ancestors: smallvec::smallvec![root_file_id],
                expected_device_id,
                allowed_secondary_device_id,
            };
            injector.push(initial_task);

            // Local queues are LIFO (depth-first: few queued tasks, the subtree just listed
            // read while it is hot); thieves take the oldest tasks, the largest subtrees.
            let workers: Vec<Worker<ScanTask>> =
                (0..num_threads).map(|_| Worker::new_lifo()).collect();
            let stealers: Arc<Vec<Stealer<ScanTask>>> =
                Arc::new(workers.iter().map(Worker::stealer).collect());
            let slots = Arc::new(Slots::default());
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

            for (worker_idx, local_worker) in workers.into_iter().enumerate() {
                let stealers = stealers.clone();
                let injector = injector.clone();
                let in_flight_tasks = in_flight_tasks.clone();
                let done = done.clone();
                let event_tx = event_tx.clone();
                let scan_cancel = scan_cancel.clone();
                let overlay_mounts = overlay_mounts.clone();
                let slots = slots.clone();

                let stats = stats.clone();

                thread_handles.push(thread::spawn(move || {
                    #[cfg(all(target_os = "linux", feature = "unsafe"))]
                    linux::own_descriptor_table();
                    let mut ctx = WorkerContext {
                        builder: BatchBuilder::default(),
                        last_flush: Instant::now(),
                        subdirs: Vec::new(),
                        #[cfg(target_os = "linux")]
                        dirents: vec![std::mem::MaybeUninit::uninit(); 32 * 1024],
                        event_tx: &event_tx,
                        local_worker: &local_worker,
                        stats: &stats,
                        scan_cancel: &scan_cancel,
                        in_flight_tasks: &in_flight_tasks,
                        overlay_mounts: &overlay_mounts,
                        slots: &slots,
                    };

                    loop {
                        if ctx.scan_cancel.load(Ordering::Relaxed) {
                            done.store(true, Ordering::SeqCst);
                        }
                        if done.load(Ordering::SeqCst) {
                            break;
                        }

                        let task_opt = ctx
                            .local_worker
                            .pop()
                            .or_else(|| steal(&injector, &stealers, worker_idx, &local_worker));

                        if let Some(task) = task_opt {
                            // Process the directory scan task; its in-flight
                            // count is released only once the scan completes.
                            scan_directory(&task, &mut ctx);
                            in_flight_tasks.fetch_sub(1, Ordering::SeqCst);
                        } else {
                            // Idle: hand over what was listed so far.
                            ctx.flush();

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

                    // Flush final batch remaining in buffer
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

/// A task from the injector, or else from another worker (half its queue, the oldest tasks),
/// starting with the next one so that thieves spread out.
fn steal(
    injector: &Injector<ScanTask>,
    stealers: &[Stealer<ScanTask>],
    worker_idx: usize,
    local_worker: &Worker<ScanTask>,
) -> Option<ScanTask> {
    loop {
        let mut retry = false;
        match injector.steal() {
            Steal::Success(task) => return Some(task),
            Steal::Retry => retry = true,
            Steal::Empty => {}
        }
        for offset in 1..stealers.len() {
            let victim = &stealers[(worker_idx + offset) % stealers.len()];
            match victim.steal_batch_and_pop(local_worker) {
                Steal::Success(task) => return Some(task),
                Steal::Retry => retry = true,
                Steal::Empty => {}
            }
        }
        if !retry {
            return None;
        }
    }
}

struct WorkerContext<'a> {
    builder: BatchBuilder,
    last_flush: Instant,
    /// Scratch: the subdirectories of the directory being read (listing index, path, id, size).
    subdirs: Vec<(u32, PathBuf, (u64, u64), u64)>,
    /// Scratch: the buffer directory entries are read into.
    #[cfg(target_os = "linux")]
    dirents: Vec<std::mem::MaybeUninit<u8>>,
    event_tx: &'a Sender<ScanBatch>,
    local_worker: &'a Worker<ScanTask>,
    stats: &'a TraversalStats,
    scan_cancel: &'a Arc<AtomicBool>,
    in_flight_tasks: &'a AtomicUsize,
    overlay_mounts: &'a HashSet<PathBuf>,
    slots: &'a Slots,
}

impl WorkerContext<'_> {
    fn flush(&mut self) {
        self.builder.flush(self.event_tx, self.stats);
        self.last_flush = Instant::now();
    }
}

/// How reading a directory ended.
enum Read {
    /// Every entry was listed. On btrfs, whether the directory is.
    Listed { btrfs: Option<bool> },
    /// The scan was cancelled midway.
    Cancelled,
    /// The directory could not be opened.
    Unreadable,
}

fn scan_directory(task: &ScanTask, ctx: &mut WorkerContext<'_>) {
    if ctx.scan_cancel.load(Ordering::Relaxed) {
        return;
    }
    ctx.subdirs.clear();

    #[cfg(target_os = "linux")]
    let read = linux::read_directory(task, ctx);
    #[cfg(not(target_os = "linux"))]
    let read = read_directory(task, ctx);

    let btrfs = match read {
        Read::Listed { btrfs } => btrfs,
        Read::Cancelled => {
            // An interrupted listing is never sent: the scan is thrown away.
            ctx.builder.discard_listing();
            return;
        }
        Read::Unreadable => return,
    };

    // The listing's slots exist from here on: its subdirectories can be queued.
    if let Some(first_slot) = ctx.builder.finish_listing(task.slot, ctx.slots) {
        // Keep the in-flight count nonzero for as long as queued work
        // exists; this scan's own count is released after it returns, so
        // the increment always precedes that decrement.
        ctx.in_flight_tasks
            .fetch_add(ctx.subdirs.len(), Ordering::SeqCst);
        for (index, path, file_id, dir_size) in ctx.subdirs.drain(..) {
            let mut ancestors = task.ancestors.clone();
            if file_id != (0, 0) {
                ancestors.push(file_id);
            }
            ctx.local_worker.push(ScanTask {
                path,
                slot: first_slot + index,
                device_id: file_id.0,
                dir_size,
                // Another device may be another filesystem: it is checked when read.
                btrfs: btrfs.filter(|_| file_id.0 == task.device_id),
                ancestors,
                expected_device_id: task.expected_device_id,
                allowed_secondary_device_id: task.allowed_secondary_device_id,
            });
        }
    }

    if ctx.builder.is_full() || ctx.last_flush.elapsed() >= FLUSH_INTERVAL {
        ctx.flush();
    }
}

/// Lists one entry of the directory `task` reads: a node, and a task for a subdirectory to
/// descend into. `child_path` makes the subdirectory's path, only when one is needed.
fn add_entry(
    task: &ScanTask,
    ctx: &mut WorkerContext<'_>,
    name: &str,
    meta: &crate::arena::EntryMetadata,
    child_path: impl FnOnce() -> PathBuf,
) {
    if !meta.is_dir {
        ctx.builder
            .push(name, FileNode::from_metadata(StringId(0), None, meta));
        return;
    }

    // Skip virtual filesystems, device nodes, network mounts, and macOS APFS
    // sub-volume mirror mount points when scanning the system root.
    if should_skip_root_entry(&task.path, name) {
        return;
    }

    // Mount Point / Device boundary safety protection check
    if is_device_boundary_crossed(
        task.expected_device_id,
        task.allowed_secondary_device_id,
        meta.file_id.0,
    ) {
        // Do not descend into subdirectories across filesystem boundaries (e.g. /sys or /proc)
        return;
    }

    // Cycle Detection
    if meta.file_id != (0, 0) && task.ancestors.contains(&meta.file_id) {
        return;
    }

    let index = ctx.builder.listing_len() as u32;
    ctx.builder
        .push(name, FileNode::from_metadata(StringId(0), None, meta));

    // Overlay mountpoints (e.g. a running container's `…/merged`)
    // duplicate bytes already counted under the layer `diff/` dirs.
    // The node stays visible (size 0) but no scan task is queued.
    // Discovered directories are never the traversal root itself, so
    // an explicit root scan of a mountpoint is unaffected. A mountpoint
    // is on another device than its parent: only those are looked up.
    let child_path = child_path();
    if !ctx.overlay_mounts.is_empty()
        && meta.file_id.0 != task.device_id
        && super::mounts::is_overlay_mountpoint(ctx.overlay_mounts, &child_path)
    {
        return;
    }

    ctx.subdirs
        .push((index, child_path, meta.file_id, meta.len));
}

/// Reads a directory through the standard library.
#[cfg(not(target_os = "linux"))]
fn read_directory(task: &ScanTask, ctx: &mut WorkerContext<'_>) -> Read {
    let entries = match fs::read_dir(&task.path) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ctx.builder.deny(task.slot);
            }
            return Read::Unreadable;
        }
    };
    ctx.builder.add_dirs(1);

    for (entry_idx, entry_res) in entries.enumerate() {
        if entry_idx % 256 == 0 && ctx.scan_cancel.load(Ordering::Relaxed) {
            return Read::Cancelled;
        }
        let Ok(entry) = entry_res else { continue };
        let Some(meta) = crate::arena::EntryMetadata::from_dir_entry(&entry) else {
            continue;
        };
        add_entry(task, ctx, &meta.name, &meta, || entry.path());
    }
    Read::Listed { btrfs: None }
}

/// Reads directories with `getdents64` and `statx` (relative to the directory), as the standard
/// library does, but with no allocation per entry and no `fstat` per directory; and on btrfs
/// without the last, empty `getdents64`: a btrfs directory's size is twice the length of its
/// entries' names, so once that many have been read, the directory is done (each `getdents64`
/// on btrfs searches the filesystem tree, and the empty one is half of them).
#[cfg(target_os = "linux")]
mod linux {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _, sync::atomic::Ordering};

    use compact_str::CompactString;
    use rustix::{
        fs::{
            AtFlags, CWD, FileType, Mode, OFlags, RawDir, StatxFlags, fstatfs, makedev, openat,
            statx,
        },
        io::Errno,
    };

    use super::{Read, ScanTask, WorkerContext, add_entry};
    use crate::arena::EntryMetadata;

    const BTRFS_SUPER_MAGIC: u32 = 0x9123_683e;

    /// Gives the calling worker a table of file descriptors of its own, holding only standard
    /// input, output and error. The process's table has one lock, which every `openat` and
    /// `close` of a directory takes: 32 workers spent a sixth of their time waiting on it. Before
    /// Linux 5.9 (no `close_range`), or without the `unsafe` feature, the worker keeps sharing
    /// the process's table.
    #[cfg(feature = "unsafe")]
    pub(super) fn own_descriptor_table() {
        // SAFETY: the descriptors a worker uses are its own: each directory is opened, read and
        // closed in `read_directory`, and none is handed to or taken from another thread (the
        // copies of the process's others are closed in this one's table, not in the process's).
        #[allow(unsafe_code)]
        unsafe {
            libc::syscall(
                libc::SYS_close_range,
                3,
                libc::c_uint::MAX,
                libc::CLOSE_RANGE_UNSHARE,
            );
        }
    }

    /// Seconds since the epoch, as `time_utils::system_time_to_unix_timestamp` gives them.
    fn seconds(tv_sec: i64) -> u32 {
        u32::try_from(tv_sec.max(0)).unwrap_or(u32::MAX)
    }

    /// The entry's metadata, as `EntryMetadata::from_dir_entry` makes it, but for its name,
    /// which the caller keeps.
    fn metadata(kind: FileType, stat: &Result<rustix::fs::Statx, Errno>) -> Option<EntryMetadata> {
        let name = CompactString::default();
        match stat {
            Ok(stat) => {
                let kind = FileType::from_raw_mode(u32::from(stat.stx_mode));
                let created = if stat.stx_mask & StatxFlags::BTIME.bits() == 0 {
                    0
                } else {
                    seconds(stat.stx_btime.tv_sec)
                };
                Some(EntryMetadata {
                    name,
                    is_dir: kind == FileType::Directory,
                    is_symlink: kind == FileType::Symlink,
                    is_dataless: false,
                    is_special: matches!(
                        kind,
                        FileType::Fifo
                            | FileType::Socket
                            | FileType::BlockDevice
                            | FileType::CharacterDevice
                    ),
                    len: stat.stx_size,
                    modified_timestamp: seconds(stat.stx_mtime.tv_sec),
                    created_timestamp: created,
                    file_id: (
                        makedev(stat.stx_dev_major, stat.stx_dev_minor),
                        stat.stx_ino,
                    ),
                    no_permission: false,
                })
            }
            Err(Errno::ACCESS | Errno::PERM) => Some(EntryMetadata {
                name,
                is_dir: kind == FileType::Directory,
                is_symlink: kind == FileType::Symlink,
                is_dataless: false,
                is_special: false,
                len: 0,
                modified_timestamp: 0,
                created_timestamp: 0,
                file_id: (0, 0),
                no_permission: true,
            }),
            Err(_) => None,
        }
    }

    pub(super) fn read_directory(task: &ScanTask, ctx: &mut WorkerContext<'_>) -> Read {
        let fd = match openat(
            CWD,
            &task.path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(Errno::ACCESS | Errno::PERM) => {
                ctx.builder.deny(task.slot);
                return Read::Unreadable;
            }
            Err(_) => return Read::Unreadable,
        };
        ctx.builder.add_dirs(1);

        let btrfs = task.btrfs.or_else(|| {
            fstatfs(&fd)
                .ok()
                .map(|fs| fs.f_type as u32 == BTRFS_SUPER_MAGIC)
        });
        // The names' length a btrfs directory of this size holds.
        let expected = btrfs.unwrap_or(false).then_some(task.dir_size);
        if expected == Some(0) {
            return Read::Listed { btrfs };
        }

        let mut buffer = std::mem::take(&mut ctx.dirents);
        let read = list(task, ctx, &fd, &mut buffer, btrfs, expected);
        ctx.dirents = buffer;
        read
    }

    /// Lists every entry of the open directory `fd`, stopping once the names reach `expected`.
    fn list(
        task: &ScanTask,
        ctx: &mut WorkerContext<'_>,
        fd: &rustix::fd::OwnedFd,
        buffer: &mut [std::mem::MaybeUninit<u8>],
        btrfs: Option<bool>,
        expected: Option<u64>,
    ) -> Read {
        let mut entries = RawDir::new(fd, buffer);
        let mut name_bytes = 0u64;
        let mut seen = 0usize;
        while let Some(entry) = entries.next() {
            let Ok(entry) = entry else { break };
            let name = entry.file_name().to_bytes();
            if name == b"." || name == b".." {
                continue;
            }
            seen += 1;
            if seen.is_multiple_of(256) && ctx.scan_cancel.load(Ordering::Relaxed) {
                return Read::Cancelled;
            }
            name_bytes += name.len() as u64;
            let stat = statx(
                fd,
                entry.file_name(),
                AtFlags::SYMLINK_NOFOLLOW,
                StatxFlags::BASIC_STATS | StatxFlags::BTIME,
            );
            if let Some(meta) = metadata(entry.file_type(), &stat) {
                // Borrowed unless the name is not UTF-8, as `to_string_lossy` gives it.
                let text = String::from_utf8_lossy(name);
                add_entry(task, ctx, &text, &meta, || {
                    task.path.join(OsStr::from_bytes(name))
                });
            }
            if expected == Some(name_bytes * 2) {
                break;
            }
        }
        Read::Listed { btrfs }
    }
}

#[cfg(unix)]
fn get_device_id(_path: &Path, meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt as _;

    meta.dev()
}

#[cfg(windows)]
fn get_device_id(path: &Path, meta: &fs::Metadata) -> u64 {
    get_file_id(path, meta).0
}

#[cfg(not(any(unix, windows)))]
fn get_device_id(_path: &Path, _meta: &fs::Metadata) -> u64 {
    0
}

#[cfg(target_os = "macos")]
fn get_secondary_device_id(root_path: &Path) -> Option<u64> {
    // When scanning root "/" on macOS, the system partition (read-only snapshot)
    // is paired with the Data partition mounted at "/System/Volumes/Data".
    // Firmlinks (such as /Users, /Applications, /Library) reside on this Data volume.
    if root_path == Path::new("/") {
        fs::metadata("/System/Volumes/Data")
            .ok()
            .as_ref()
            .map(|m| get_device_id(Path::new("/System/Volumes/Data"), m))
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
const fn get_secondary_device_id(_root_path: &Path) -> Option<u64> {
    None
}

/// Checks whether a subdirectory crosses outside the allowed filesystem/device boundaries.
#[must_use]
pub(crate) const fn is_device_boundary_crossed(
    expected_dev: Option<u64>,
    secondary_dev: Option<u64>,
    entry_dev: u64,
) -> bool {
    if let Some(expected) = expected_dev
        && entry_dev != 0
        && entry_dev != expected
    {
        return match secondary_dev {
            Some(sec) => sec != entry_dev,
            None => true,
        };
    }
    false
}

/// Determines if a directory entry should be skipped when traversing the system root.
///
/// Prevents recursion into virtual filesystems, device nodes, transient mounts,
/// and macOS APFS volume mirrors (`/Volumes`, `/System/Volumes`).
#[must_use]
pub(crate) fn should_skip_root_entry(task_path: &Path, name: &str) -> bool {
    // 1. Direct children of the system root ("/")
    if task_path == Path::new("/") {
        // Universal / Linux virtual filesystems, device nodes, and transient mounts
        if matches!(
            name,
            "proc" | "sys" | "dev" | "run" | "tmp" | "mnt" | "media" | "lost+found"
        ) {
            return true;
        }

        // macOS / Darwin root mounts, virtual filesystems, and index stores
        #[cfg(any(target_os = "macos", test))]
        if matches!(
            name,
            "Volumes"
                | "volumes"
                | "Network"
                | "network"
                | "cores"
                | ".vol"
                | ".file"
                | ".Spotlight-V100"
                | ".spotlight-v100"
                | ".fseventsd"
                | ".DocumentRevisions-V100"
                | ".documentrevisions-v100"
                | ".MobileBackups"
                | ".mobilebackups"
                | ".PKInstallSandboxManager"
                | ".PKInstallSandboxManager-SystemSoftware"
                | ".cleverfiles"
                | ".Trashes"
                | ".trashes"
        ) {
            return true;
        }
    }

    // 2. macOS /System sub-mounts: /System/Volumes contains /System/Volumes/Data
    // (which mirrors /Users, /Applications, etc. via firmlinks) plus Preboot, Recovery, and VM.
    #[cfg(any(target_os = "macos", test))]
    if (task_path == Path::new("/System")
        || (task_path
            .file_name()
            .is_some_and(|f| f.eq_ignore_ascii_case("System"))
            && task_path.parent() == Some(Path::new("/"))))
        && (name == "Volumes" || name == "volumes")
    {
        return true;
    }

    false
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

    /// Every entry of a folder larger than one read of its entries, whatever the bytes of its
    /// names: on btrfs the listing stops when the names add up to the folder's size.
    #[test]
    #[cfg(unix)]
    fn test_traversal_large_folder_odd_names_all_listed() -> Result<(), crate::EdirstatError> {
        use std::os::unix::ffi::OsStrExt as _;

        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_large_folder");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("sub"))?;
        for i in 0..3000 {
            std::fs::write(
                temp_dir.join(format!("{i:04}-{}.dat", "x".repeat(40))),
                b"a",
            )?;
        }
        std::fs::write(temp_dir.join("ünïcødé-名前.txt"), b"bb")?;
        // A name that is not UTF-8, where the filesystem takes one (APFS refuses it: EILSEQ).
        let not_utf8 = match std::fs::write(
            temp_dir.join(std::ffi::OsStr::from_bytes(b"latin1-\xe9t\xe9.txt")),
            b"ccc",
        ) {
            Ok(()) => true,
            Err(error) if error.raw_os_error() == Some(libc::EILSEQ) => false,
            Err(error) => return Err(error.into()),
        };
        std::fs::write(temp_dir.join("sub").join("inner"), b"dddd")?;

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
        let (odd_files, odd_bytes) = if not_utf8 { (1, 3) } else { (0, 0) };
        assert_eq!(snapshot.nodes[0].file_count, 3002 + odd_files);
        assert_eq!(snapshot.nodes[0].size, 3000 + 2 + 4 + odd_bytes);
        assert!(node_index(&snapshot, "ünïcødé-名前.txt").is_some());
        assert!(!not_utf8 || node_index(&snapshot, "latin1-\u{fffd}t\u{fffd}.txt").is_some());
        assert!(node_index(&snapshot, "inner").is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    /// Workers close the process's descriptors in their own tables only: a file the process holds
    /// open stays open, and readable, after a scan.
    #[test]
    #[cfg(target_os = "linux")]
    fn test_traversal_keeps_process_descriptors_open() -> Result<(), crate::EdirstatError> {
        use std::io::{Read as _, Seek as _};

        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_descriptors");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("a").join("b"))?;
        std::fs::write(temp_dir.join("a").join("file"), b"kept")?;
        let mut held = std::fs::File::open(temp_dir.join("a").join("file"))?;

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
        assert!(node_index(&snapshot, "file").is_some());
        let mut contents = String::new();
        held.rewind()?;
        held.read_to_string(&mut contents)?;
        assert_eq!(contents, "kept");

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    /// A folder that can be listed but not searched: its entries are there, marked unreadable.
    #[test]
    #[cfg(unix)]
    fn test_traversal_unsearchable_folder_entries_marked() -> Result<(), crate::EdirstatError> {
        use std::os::unix::fs::PermissionsExt as _;

        let is_root = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .is_some_and(|uid| uid.trim() == "0");
        if is_root {
            return Ok(());
        }

        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_traversal_unsearchable");
        let locked = temp_dir.join("listable");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(locked.join("child_dir"))?;
        std::fs::write(locked.join("child_file"), b"secret")?;
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o444))?;

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

        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))?;
        let snapshot = shared_state.current_snapshot.load();
        let file = node_index(&snapshot, "child_file").map(|i| snapshot.nodes[i]);
        let dir = node_index(&snapshot, "child_dir").map(|i| snapshot.nodes[i]);
        assert!(file.is_some_and(|n| n.has_no_permission() && !n.is_directory() && n.size == 0));
        assert!(dir.is_some_and(|n| n.has_no_permission() && n.is_directory()));

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

    #[test]
    fn test_should_skip_root_entry_universal_and_linux() {
        let root = Path::new("/");
        let linux_skips = [
            "proc",
            "sys",
            "dev",
            "run",
            "tmp",
            "mnt",
            "media",
            "lost+found",
        ];
        for name in linux_skips {
            assert!(
                should_skip_root_entry(root, name),
                "expected root skip for '{name}'"
            );
        }

        // Standard user and system dirs at root must NOT be skipped
        let allowed = ["home", "etc", "var", "usr", "opt", "bin", "boot", "srv"];
        for name in allowed {
            assert!(
                !should_skip_root_entry(root, name),
                "expected '{name}' at root to be allowed"
            );
        }
    }

    #[test]
    fn test_should_skip_root_entry_macos() {
        let root = Path::new("/");
        let macos_root_skips = [
            "Volumes",
            "volumes",
            "Network",
            "network",
            "cores",
            ".vol",
            ".file",
            ".Spotlight-V100",
            ".spotlight-v100",
            ".fseventsd",
            ".DocumentRevisions-V100",
            ".documentrevisions-v100",
            ".MobileBackups",
            ".mobilebackups",
            ".PKInstallSandboxManager",
            ".PKInstallSandboxManager-SystemSoftware",
            ".cleverfiles",
            ".Trashes",
            ".trashes",
        ];
        for name in macos_root_skips {
            assert!(
                should_skip_root_entry(root, name),
                "expected macOS root skip for '{name}'"
            );
        }

        // macOS standard firmlinks and root directories must NOT be skipped
        let macos_allowed = ["Users", "Applications", "Library", "System", "private"];
        for name in macos_allowed {
            assert!(
                !should_skip_root_entry(root, name),
                "expected '{name}' at root to be allowed"
            );
        }

        // macOS /System/Volumes sub-mount must be skipped to avoid double-counting
        let sys_path = Path::new("/System");
        assert!(should_skip_root_entry(sys_path, "Volumes"));
        assert!(should_skip_root_entry(sys_path, "volumes"));
        assert!(!should_skip_root_entry(sys_path, "Library"));
        assert!(!should_skip_root_entry(sys_path, "Applications"));
        assert!(!should_skip_root_entry(sys_path, "Cryptexes"));

        // With trailing slash on /System/
        let sys_trailing = Path::new("/System/");
        assert!(should_skip_root_entry(sys_trailing, "Volumes"));
        assert!(should_skip_root_entry(sys_trailing, "volumes"));
        assert!(!should_skip_root_entry(sys_trailing, "Library"));
    }

    #[test]
    fn test_should_skip_root_entry_non_root_paths() {
        // When scanning non-root folders, directories with names like "Volumes", "proc",
        // or "tmp" must NOT be skipped.
        let user_path = Path::new("/home/tux/Volumes");
        assert!(!should_skip_root_entry(user_path, "Volumes"));
        assert!(!should_skip_root_entry(user_path, "proc"));
        assert!(!should_skip_root_entry(user_path, "tmp"));

        let mac_user_path = Path::new("/Users/tux/Projects");
        assert!(!should_skip_root_entry(mac_user_path, "Volumes"));
        assert!(!should_skip_root_entry(mac_user_path, "cores"));
        assert!(!should_skip_root_entry(mac_user_path, "media"));

        let ext_drive = Path::new("/Volumes/External/Data");
        assert!(!should_skip_root_entry(ext_drive, "Volumes"));
        assert!(!should_skip_root_entry(ext_drive, "proc"));
    }

    #[test]
    fn test_is_device_boundary_crossed() {
        // When same_filesystem is false (expected_dev is None), never consider crossed
        assert!(!is_device_boundary_crossed(None, None, 100));
        assert!(!is_device_boundary_crossed(None, Some(200), 300));

        let root_dev = Some(100);
        let data_dev = Some(200);

        // Same filesystem as root
        assert!(!is_device_boundary_crossed(root_dev, None, 100));
        assert!(!is_device_boundary_crossed(root_dev, data_dev, 100));

        // Unknown device ID (0) should not trigger boundary check
        assert!(!is_device_boundary_crossed(root_dev, None, 0));
        assert!(!is_device_boundary_crossed(root_dev, data_dev, 0));

        // Different filesystem without secondary device allowed -> boundary crossed
        assert!(is_device_boundary_crossed(root_dev, None, 200));

        // Paired secondary device (e.g. APFS Data volume paired with System root) -> NOT crossed
        assert!(!is_device_boundary_crossed(root_dev, data_dev, 200));

        // Third filesystem (e.g. external volume or disk image) -> boundary crossed
        assert!(is_device_boundary_crossed(root_dev, data_dev, 300));
    }
}
