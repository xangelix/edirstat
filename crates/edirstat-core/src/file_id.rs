use std::fs;

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(unix)]
#[must_use]
pub fn get_file_id(meta: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt as _;

    (meta.dev(), meta.ino())
}

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(windows)]
#[must_use]
pub fn get_file_id(meta: &fs::Metadata) -> (u64, u64) {
    use std::os::windows::fs::MetadataExt as _;

    (
        meta.volume_serial_number().unwrap_or(0) as u64,
        meta.file_index().unwrap_or(0),
    )
}

/// Platform-specific unique file identifier (device, inode/file-index).
///
/// Used for hardlink detection and cycle/device boundary checks. Falls back
/// to `(0, 0)` on platforms without a native identifier (e.g. wasm).
#[cfg(not(any(unix, windows)))]
#[must_use]
pub fn get_file_id(_meta: &fs::Metadata) -> (u64, u64) {
    (0, 0)
}
