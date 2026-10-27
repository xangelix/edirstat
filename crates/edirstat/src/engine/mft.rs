//! The NTFS Master File Table scanner, on `ntfs-ng`.
//!
//! A raw NTFS volume (Windows `\\.\C:`, a Linux block device) or an exported `$MFT` file is read
//! by `ntfs-ng`'s parallel scan into its disk-usage table (every name of every live file, with
//! its size and two times, extension records folded in) and a directory index; the tree under the
//! scanned folder is then streamed as listings ([`ScanBatch`]es), directories before their
//! contents.

use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam::channel::Sender;
use ntfs_ng::{
    index::{DirId, PathPolicy, group},
    mft::Mft,
    records,
    scan::Scan,
    source::ReadAt,
    usage::{Usage, usage_flags},
};

use super::{
    batch::{BatchBuilder, ScanBatch, Slots},
    traversal::TraversalStats,
};
use crate::arena::{FileNode, StringId};

/// Most records scanned: a guard against a damaged volume that claims an absurd `$MFT`.
const MAX_RECORDS: u64 = 50_000_000;
/// Most threads reading and parsing `$MFT`: more only contend for the same memory bandwidth
/// (measured: 8 threads 15% faster than 32 on a 16-core machine, with a third of the CPU time).
const SCAN_THREADS: usize = 8;

fn invalid(error: impl std::fmt::Display) -> crate::EdirstatError {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string()).into()
}

/// Scans `mft` and streams the tree under `search_root` (a folder whose content is the volume's,
/// or the `$MFT` file itself when there is none: then the whole volume).
fn scan_mft<S: ReadAt>(
    mft: &Mft<S>,
    search_root: &Path,
    inside_volume: Option<&Path>,
    allow_manual_fs_checks: bool,
    scan_cancel: &Arc<AtomicBool>,
    event_tx: &Sender<ScanBatch>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    let records = mft.records();
    if records == 0 || records > MAX_RECORDS {
        return Err(invalid(
            "MFT size descriptor is invalid or excessively large",
        ));
    }
    let record_size = mft.record_size() as usize;
    let mut scanned = 0usize;
    let threads = std::thread::available_parallelism()
        .map_or(4, std::num::NonZeroUsize::get)
        .min(SCAN_THREADS);
    let scan = Scan::new(mft).threads(threads).cancel(scan_cancel);
    let usage = match Usage::scan_observed(&scan, records, PathPolicy::LIVE, &mut |chunk| {
        scanned += chunk.records;
        stats.files_scanned.store(scanned, Ordering::Relaxed);
        stats
            .bytes_scanned
            .store(scanned * record_size, Ordering::Relaxed);
    }) {
        Ok(usage) => usage,
        Err(ntfs_ng::Error::Cancelled) => return Ok(()),
        Err(error) => return Err(invalid(error)),
    };
    stats.reset();

    let parents = usage.parents(threads);
    let index = &usage.index;
    let (offsets, members) = group(&parents, index.len());
    let children = |dir: DirId| {
        let start = offsets.get(dir as usize).copied().unwrap_or(0) as usize;
        let end = offsets.get(dir as usize + 1).copied().unwrap_or(0) as usize;
        members.get(start..end).unwrap_or_default()
    };
    let Some(root) = index.directory(records::ROOT) else {
        return Err(invalid("the volume has no root directory"));
    };

    // The folder scanned: its components on the volume (below its mount point, on Linux)
    // followed from the root, case-insensitively; the root when they cannot be (an exported
    // $MFT, a path outside the volume).
    let mut target = root;
    for component in inside_volume.unwrap_or(search_root).components() {
        let std::path::Component::Normal(name) = component else {
            continue;
        };
        let name = name.to_string_lossy();
        let found = children(target).iter().find_map(|&member| {
            let (entry, entry_name) = usage.entry(member)?;
            (entry.is_directory() && entry_name.eq_ignore_ascii_case(&name))
                .then(|| index.directory(entry.record))
                .flatten()
        });
        let Some(dir) = found else {
            target = root;
            break;
        };
        target = dir;
    }
    let base_depth = index.depth(target);

    // Depth-first from the target: each directory listed whole, its subdirectories after.
    let slots = Slots::default();
    let mut builder = BatchBuilder::default();
    // The folder scanned counts as a directory read, as the walker counts it.
    builder.add_dirs(1);
    let mut stack = vec![(target, 0u32)];
    let mut visited = vec![0u64; index.len().div_ceil(64)];
    if let Some(word) = visited.get_mut(target as usize / 64) {
        *word |= 1 << (target % 64);
    }
    let mut subdirs = Vec::new();
    let mut ancestors = Vec::new();
    while let Some((dir, slot)) = stack.pop() {
        if scan_cancel.load(Ordering::Relaxed) {
            break;
        }
        for &member in children(dir) {
            let Some((entry, name)) = usage.entry(member) else {
                continue;
            };
            let (created, modified) = (entry.created, entry.modified);
            if entry.is_directory() {
                let Some(child) = index.directory(entry.record) else {
                    continue;
                };
                let Some(word) = visited.get_mut(child as usize / 64) else {
                    continue;
                };
                let bit = 1u64 << (child % 64);
                if *word & bit != 0 {
                    continue;
                }
                *word |= bit;
                builder.add_dirs(1);
                subdirs.push((child, builder.listing_len() as u32));
                builder.push(
                    name,
                    FileNode::new(StringId(0), None, true, false, modified, created),
                );
            } else {
                let link = entry.flags & usage_flags::LINK != 0;
                let mut size = entry.size;
                if allow_manual_fs_checks && entry.size_untrusted() {
                    // The volume knows better than a stale record: ask it. A link's own size, as
                    // the walker counts it, not its target's; other reparse points (a cloud
                    // placeholder) are followed to the size they stand for.
                    index.ancestors(dir, &mut ancestors);
                    let mut path = PathBuf::from(search_root);
                    for &ancestor in ancestors.get(base_depth..).unwrap_or_default() {
                        path.push(index.name_str(ancestor));
                    }
                    path.push(name);
                    let meta = if link {
                        std::fs::symlink_metadata(&path)
                    } else {
                        std::fs::metadata(&path)
                    };
                    if let Ok(meta) = meta {
                        size = meta.len();
                    }
                }
                let mut node = FileNode::new(StringId(0), None, false, link, modified, created);
                node.size = size;
                builder.push(name, node);
            }
        }
        if let Some(first_slot) = builder.finish_listing(slot, &slots) {
            stack.extend(
                subdirs
                    .iter()
                    .map(|&(child, index)| (child, first_slot + index)),
            );
        }
        subdirs.clear();
        if builder.is_full() {
            builder.flush(event_tx, stats);
        }
    }
    builder.flush(event_tx, stats);
    Ok(())
}

/// Scans an exported, raw `$MFT` file on any platform.
///
/// If `search_root_path` is `Some` then it should contain the path to a directory
/// with the same content as the parsed data of the $MFT file.
fn scan_mft_file(
    file_path: &Path,
    search_root_path: Option<&Path>,
    scan_cancel: &Arc<AtomicBool>,
    event_tx: &Sender<ScanBatch>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    let source = ntfs_ng::source::FileSource::open(file_path)?;
    let mft = Mft::from_file(source).map_err(invalid)?;
    let inside_volume = search_root_path.and_then(path_inside_volume);
    scan_mft(
        &mft,
        search_root_path.unwrap_or(file_path),
        inside_volume.as_deref(),
        search_root_path.is_some(),
        scan_cancel,
        event_tx,
        stats,
    )
}

/// Primary entry point for raw MFT scanning.
/// Parses either the raw drive partition on Windows or a standalone MFT copy on any platform.
pub fn try_scan_mft(
    root_path: &Path,
    scan_cancel: &Arc<AtomicBool>,
    event_tx: &Sender<ScanBatch>,
    stats: &TraversalStats,
) -> Result<(), crate::EdirstatError> {
    if root_path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("$mft"))
    {
        return scan_mft_file(root_path, None, scan_cancel, event_tx, stats);
    }

    // The volume's `$MFT` file is read on ntfs-3g (FUSE), which serves it as it is now. The
    // kernel's ntfs3 serves it as it was when the volume was mounted, without anything written
    // since: there the device is read, or else (without access to it) the folders are walked.
    #[cfg(target_os = "linux")]
    {
        let fuse = {
            let disks = sysinfo::Disks::new_with_refreshed_list();
            get_disk(root_path, &disks)
                .is_some_and(|disk| disk.file_system().eq_ignore_ascii_case("fuseblk"))
        };
        if fuse {
            let Some(mft_path) = find_mft_file_at_mount(root_path) else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "$MFT file not found in root of disk",
                )
                .into());
            };
            return scan_mft_file(&mft_path, Some(root_path), scan_cancel, event_tx, stats);
        }
    }

    let volume_path = get_volume_path(root_path).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "Unable to determine partition drive path from target path",
        )
    });

    // Direct reads that bypass the system cache: a full scan should not evict everything else.
    let options = ntfs_ng::source::DeviceOptions {
        unbuffered: cfg!(target_os = "windows"),
        ..ntfs_ng::source::DeviceOptions::default()
    };
    let device =
        volume_path.and_then(|volume_path| ntfs_ng::source::Device::open(&volume_path, options));

    let volume = ntfs_ng::volume::Volume::open(device?).map_err(invalid)?;
    let inside_volume = path_inside_volume(root_path);
    scan_mft(
        volume.mft(),
        root_path,
        inside_volume.as_deref(),
        true,
        scan_cancel,
        event_tx,
        stats,
    )
}

/// Where `path` is on the volume mounted at `mount_point`: on Linux a volume's folders are under
/// its mount point (`/mnt/windows/Users` is `Users` on the volume).
fn relative_to_mount(path: &Path, mount_point: &Path) -> Option<PathBuf> {
    path.strip_prefix(mount_point).ok().map(Path::to_path_buf)
}

/// Where `path` is on its volume (Linux), from its canonical path: the one the volume was found
/// by. `None` elsewhere, where paths already start at the volume (`C:\Users`).
#[cfg(target_os = "linux")]
fn path_inside_volume(path: &Path) -> Option<PathBuf> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mount_point = get_disk(path, &disks)?.mount_point().to_path_buf();
    relative_to_mount(&path.canonicalize().ok()?, &mount_point)
}

#[cfg(not(target_os = "linux"))]
const fn path_inside_volume(_path: &Path) -> Option<PathBuf> {
    None
}

/// Resolves the Windows partition volume path from a standard file path
#[cfg(target_os = "windows")]
#[must_use]
fn get_volume_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let trimmed = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    let mut chars = trimmed.chars();
    if let Some(drive) = chars.next()
        && chars.next() == Some(':')
        && drive.is_ascii_alphabetic()
    {
        Some(format!("\\\\.\\{}:", drive.to_ascii_uppercase()))
    } else {
        None
    }
}

/// Resolves the partition volume path from a mount file path
#[cfg(target_os = "linux")]
#[must_use]
fn get_volume_path(path: &Path) -> Option<String> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disk = get_disk(path, &disks)?;
    Some(disk.name().to_str()?.to_owned())
}

/// Resolves partition paths on non-Windows targets.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[allow(clippy::unnecessary_wraps)]
#[must_use]
fn get_volume_path(path: &Path) -> Option<String> {
    Some(path.to_string_lossy().into_owned())
}

/// Probes disk mount points to find an accessible `$MFT` system file.
#[cfg(target_os = "linux")]
fn find_mft_file_at_mount(path: &Path) -> Option<PathBuf> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disk = get_disk(path, &disks)?;
    crate::fs_utils::find_mft_file(disk.mount_point())
}

/// Find the disk with the most specific match (the longest matching prefix) for the specified path.
fn get_disk<'disk>(path: &Path, disks: &'disk sysinfo::Disks) -> Option<&'disk sysinfo::Disk> {
    // Canonicalize the target path so symlinks and trailing slashes are resolved
    let target_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Strip Windows UNC prefixes so the path matches sysinfo's mount points (e.g. C:\)
    #[cfg(windows)]
    let target_path = {
        let target_path_string = target_path.to_string_lossy();
        let cleaned_str = crate::model::arena::clean_unc_path(&target_path_string);
        Path::new(cleaned_str.as_ref()).to_path_buf()
    };

    // Find the disk with the most specific match (the longest matching prefix).
    disks
        .iter()
        .filter(|disk| target_path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
}

/// Checks the root of the specified directory path to verify if the file system is NTFS.
#[must_use]
pub fn is_ntfs(path: &Path) -> bool {
    #[cfg(target_os = "linux")]
    const VALID_FS_TYPES: &[&str] = &["ntfs", "ntfs3", "fuse.ntfs", "fuse.ntfs-3g"];
    #[cfg(target_os = "windows")]
    const VALID_FS_TYPES: &[&str] = &["NTFS"];
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    const VALID_FS_TYPES: &[&str] = &[];

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let Some(disk) = get_disk(path, &disks) else {
        return false;
    };

    #[cfg(target_os = "linux")]
    if disk.file_system().eq_ignore_ascii_case("fuseblk")
        && crate::fs_utils::find_mft_file(disk.mount_point()).is_some()
    {
        // Likely fuse NTFS driver on Linux since there is an "$MFT" file in the root directory.
        return true;
    }

    VALID_FS_TYPES
        .iter()
        .any(|valid| disk.file_system().eq_ignore_ascii_case(valid))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On Linux the folder scanned is followed from below the volume's mount point.
    #[test]
    fn test_relative_to_mount() {
        let mount = Path::new("/mnt/windows");
        assert_eq!(
            relative_to_mount(Path::new("/mnt/windows/Users/x"), mount),
            Some(PathBuf::from("Users/x"))
        );
        assert_eq!(
            relative_to_mount(Path::new("/mnt/windows"), mount),
            Some(PathBuf::new())
        );
        // Not on the volume: not followed from it.
        assert_eq!(
            relative_to_mount(Path::new("/elsewhere/Users"), mount),
            None
        );
        assert_eq!(relative_to_mount(Path::new("/mnt/windows2/x"), mount), None);
    }

    #[test]
    fn test_nt_time_to_unix_zero_and_pre_epoch() {
        assert_eq!(ntfs_ng::time::unix_seconds_u32(0), 0);
        // Exactly at the 1601 -> 1970 epoch offset is unix epoch 0.
        assert_eq!(
            ntfs_ng::time::unix_seconds_u32(11_644_473_600 * 10_000_000),
            0
        );
        // One 100ns tick before the offset still maps back to 0.
        assert_eq!(
            ntfs_ng::time::unix_seconds_u32(11_644_473_600 * 10_000_000 - 1),
            0
        );
    }

    #[test]
    fn test_nt_time_to_unix_known_vector_and_saturation() {
        // NT timestamp for unix epoch + 1000 seconds.
        assert_eq!(
            ntfs_ng::time::unix_seconds_u32((11_644_473_600 + 1000) * 10_000_000),
            1000
        );
        // Values beyond year 2106 saturate to u32::MAX.
        assert_eq!(ntfs_ng::time::unix_seconds_u32(u64::MAX), u32::MAX);
    }

    /// An empty (0-byte) exported `$MFT` file yields zero records and must error out.
    #[test]
    fn test_try_scan_mft_empty_file_is_invalid_data() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_mft_empty_file");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        let mft_path = temp_dir.join("$MFT");
        std::fs::write(&mft_path, b"")?;

        let scan_cancel = Arc::new(AtomicBool::new(false));
        let (event_tx, _event_rx) = crossbeam::channel::unbounded();
        let stats = TraversalStats::default();

        let result = try_scan_mft(&mft_path, &scan_cancel, &event_tx, &stats);
        assert!(
            matches!(result, Err(crate::EdirstatError::Io(ref e)) if e.kind() == std::io::ErrorKind::InvalidData),
            "expected Io/InvalidData error for empty $MFT file, got {result:?}"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    /// A temp dir on a typical Linux dev filesystem (btrfs/ext4/tmpfs) is not NTFS.
    #[test]
    fn test_is_ntfs_temp_dir_is_false() -> Result<(), crate::EdirstatError> {
        let temp_dir = std::env::current_dir()?
            .join("target")
            .join("test_mft_is_ntfs");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;

        assert!(!is_ntfs(&temp_dir));

        let _ = std::fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
