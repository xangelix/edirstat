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
pub const fn get_file_id(_meta: &fs::Metadata) -> (u64, u64) {
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_fixture_dir(name: &str) -> std::io::Result<std::path::PathBuf> {
        let dir = std::env::current_dir()?.join("target").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn test_same_file_same_id() -> Result<(), crate::EdirstatError> {
        let dir = fresh_fixture_dir("file_id_test_same")?;
        let path = dir.join("a.txt");
        fs::write(&path, b"x")?;

        let id1 = get_file_id(&fs::metadata(&path)?);
        let id2 = get_file_id(&fs::metadata(&path)?);
        assert_eq!(id1, id2);

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    // The fallback impl returns (0, 0) for every file; identity checks are
    // only meaningful on platforms with a real native identifier.
    #[cfg(unix)]
    #[test]
    fn test_different_files_different_ids() -> Result<(), crate::EdirstatError> {
        let dir = fresh_fixture_dir("file_id_test_different")?;
        let first_path = dir.join("a.txt");
        let second_path = dir.join("b.txt");
        fs::write(&first_path, b"a")?;
        fs::write(&second_path, b"b")?;

        let first_id = get_file_id(&fs::metadata(&first_path)?);
        let second_id = get_file_id(&fs::metadata(&second_path)?);
        assert_ne!(first_id, second_id);

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_hardlink_shares_id() -> Result<(), crate::EdirstatError> {
        let dir = fresh_fixture_dir("file_id_test_hardlink")?;
        let original = dir.join("orig.txt");
        let link = dir.join("link.txt");
        fs::write(&original, b"data")?;
        fs::hard_link(&original, &link)?;

        let original_id = get_file_id(&fs::metadata(&original)?);
        let link_id = get_file_id(&fs::metadata(&link)?);
        assert_eq!(original_id, link_id);

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
