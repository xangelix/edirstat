use std::path::{Path, PathBuf};

/// Determines if the macOS BSD file flags indicate a dataless / cloud-evicted file (`UF_DATALESS`).
#[must_use]
pub const fn is_dataless_macos_flags(flags: u32) -> bool {
    const UF_DATALESS: u32 = 0x4000_0000;
    (flags & UF_DATALESS) != 0
}

/// Determines if the Windows file attributes indicate a cloud recall placeholder.
#[must_use]
pub const fn is_dataless_windows_attributes(attrs: u32) -> bool {
    const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
    const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
    (attrs & (FILE_ATTRIBUTE_RECALL_ON_OPEN | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)) != 0
}

/// Checks if a file is a dataless cloud placeholder (e.g. evicted iCloud Drive files on macOS,
/// or `OneDrive` / Cloud Filter recall-on-access files on Windows).
///
/// Attempting to read or hash dataless files will trigger an automatic download from the cloud provider,
/// consuming network bandwidth and blocking threads.
#[cfg(target_os = "macos")]
#[must_use]
pub fn is_dataless_file(meta: &std::fs::Metadata) -> bool {
    use std::os::macos::fs::MetadataExt as _;

    is_dataless_macos_flags(meta.st_flags())
}

/// Checks if a file is a dataless cloud placeholder (e.g. evicted iCloud Drive files on macOS,
/// or `OneDrive` / Cloud Filter recall-on-access files on Windows).
///
/// Attempting to read or hash dataless files will trigger an automatic download from the cloud provider,
/// consuming network bandwidth and blocking threads.
#[cfg(target_os = "windows")]
#[must_use]
pub fn is_dataless_file(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt as _;

    is_dataless_windows_attributes(meta.file_attributes())
}

/// Checks if a file is a dataless cloud placeholder (e.g. evicted iCloud Drive files on macOS,
/// or `OneDrive` / Cloud Filter recall-on-access files on Windows).
///
/// Attempting to read or hash dataless files will trigger an automatic download from the cloud provider,
/// consuming network bandwidth and blocking threads.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[must_use]
pub const fn is_dataless_file(_meta: &std::fs::Metadata) -> bool {
    false
}

/// Probes a directory path for the existence of an NTFS Master File Table (`$MFT` or `$mft`) file.
#[must_use]
pub fn find_mft_file(dir: &Path) -> Option<PathBuf> {
    let upper = dir.join("$MFT");
    if upper.is_file() {
        return Some(upper);
    }
    let lower = dir.join("$mft");
    if lower.is_file() {
        return Some(lower);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// Probe filesystem case sensitivity: on case-insensitive filesystems
    /// (default APFS, vfat, default NTFS mounts) a file is reachable under any
    /// casing, which changes which spelling `find_mft_file` can return.
    fn fs_is_case_insensitive(dir: &Path) -> Result<bool, crate::EdirstatError> {
        fs::write(dir.join("case_probe"), b"")?;
        let insensitive = dir.join("CASE_PROBE").exists();
        fs::remove_file(dir.join("case_probe"))?;
        Ok(insensitive)
    }

    #[test]
    fn test_find_mft_file_prefers_uppercase() -> Result<(), crate::EdirstatError> {
        let dir = std::env::current_dir()?
            .join("target")
            .join("fs_utils_test_mft_upper");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        assert_eq!(find_mft_file(&dir), None);

        fs::write(dir.join("$MFT"), b"mft")?;
        assert_eq!(find_mft_file(&dir), Some(dir.join("$MFT")));

        // When both casings exist, `$MFT` wins — only meaningful on
        // case-sensitive filesystems, where both spellings can coexist
        // (on case-insensitive ones the write above already IS `$MFT`).
        if !fs_is_case_insensitive(&dir)? {
            fs::write(dir.join("$mft"), b"mft")?;
            assert_eq!(find_mft_file(&dir), Some(dir.join("$MFT")));
        }

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_find_mft_file_lowercase_only_and_empty() -> Result<(), crate::EdirstatError> {
        let dir = std::env::current_dir()?
            .join("target")
            .join("fs_utils_test_mft_lower");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        assert_eq!(find_mft_file(&dir), None);

        fs::write(dir.join("$mft"), b"mft")?;
        // On case-insensitive filesystems `$MFT` resolves to this same file,
        // so either casing is a correct discovery of the lowercase-only file.
        let found = find_mft_file(&dir);
        let name = found
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|n| n.to_str());
        assert!(name.is_some_and(|n| n.eq_ignore_ascii_case("$mft")));
        assert!(found.is_some_and(|p| p.is_file()));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_dataless_flags_detection() {
        // macOS UF_DATALESS flag tests
        assert!(!is_dataless_macos_flags(0));
        assert!(!is_dataless_macos_flags(0x0000_0002)); // UF_IMMUTABLE
        assert!(is_dataless_macos_flags(0x4000_0000)); // UF_DATALESS
        assert!(is_dataless_macos_flags(0x4000_0002)); // UF_DATALESS | UF_IMMUTABLE

        // Windows cloud placeholder attribute tests
        assert!(!is_dataless_windows_attributes(0));
        assert!(!is_dataless_windows_attributes(0x0000_0020)); // FILE_ATTRIBUTE_ARCHIVE
        assert!(is_dataless_windows_attributes(0x0004_0000)); // FILE_ATTRIBUTE_RECALL_ON_OPEN
        assert!(is_dataless_windows_attributes(0x0040_0000)); // FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS
        assert!(is_dataless_windows_attributes(0x0044_0020)); // Both recall flags + ARCHIVE
    }

    #[test]
    fn test_is_dataless_file_local_file() -> Result<(), crate::EdirstatError> {
        let dir = std::env::current_dir()?
            .join("target")
            .join("fs_utils_test_dataless");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let file_path = dir.join("local.txt");
        fs::write(&file_path, b"hello")?;

        let meta = fs::metadata(&file_path)?;
        assert!(!is_dataless_file(&meta));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
