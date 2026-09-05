//! System file opening and file manager reveal operations.
//!
//! Provides cross-platform helpers to:
//! 1. Open files in their registered default application.
//! 2. Reveal and select items in the native system file manager
//!    (Finder on macOS, File Explorer on Windows, and `FreeDesktop` `FileManager1` / Dolphin / Nautilus on Linux).

use std::path::Path;

/// Opens a file in the user's default registered system application.
pub fn open_file(path: &Path) -> std::io::Result<()> {
    #[cfg(not(target_family = "wasm"))]
    {
        open::that(path)
    }
    #[cfg(target_family = "wasm")]
    {
        let _ = path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Cannot open files in web viewer",
        ))
    }
}

/// Opens the operating system's file manager and reveals/selects the given file or directory.
///
/// - **Directories**: Opens the directory directly in the file manager.
/// - **Files on macOS**: Runs `/usr/bin/open -R -- <path>`, which reveals and selects the item in Finder.
/// - **Files on Windows**: Runs `explorer.exe /select,"<path>"` to highlight the item in File Explorer.
/// - **Files on Linux**: Invokes the `FreeDesktop` `org.freedesktop.FileManager1.ShowItems` `D-Bus`
///   method, `dolphin --select`, or `nautilus --select` to highlight the item. Falls back to opening
///   the parent directory if selection tools are unavailable.
pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    #[cfg(target_family = "wasm")]
    {
        let _ = path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Cannot reveal files in web viewer",
        ))
    }
    #[cfg(all(not(target_family = "wasm"), target_os = "macos"))]
    {
        reveal_macos(path)
    }
    #[cfg(all(not(target_family = "wasm"), target_os = "windows"))]
    {
        reveal_windows(path)
    }
    #[cfg(all(not(target_family = "wasm"), target_os = "linux"))]
    {
        reveal_linux(path)
    }
    #[cfg(all(
        not(target_family = "wasm"),
        not(any(target_os = "macos", target_os = "windows", target_os = "linux"))
    ))]
    {
        reveal_fallback(path)
    }
}

#[cfg(all(not(target_family = "wasm"), target_os = "macos"))]
fn reveal_macos(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        open::that(path)
    } else {
        std::process::Command::new("/usr/bin/open")
            .arg("-R")
            .arg("--")
            .arg(path)
            .spawn()?;
        Ok(())
    }
}

#[cfg(all(not(target_family = "wasm"), target_os = "windows"))]
fn reveal_windows(path: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt as _;

    if path.is_dir() {
        open::that(path)
    } else {
        let mut cmd = std::process::Command::new("explorer");
        // Windows explorer.exe parses `/select,"<path>"` specifically.
        cmd.raw_arg(format!("/select,\"{}\"", path.display()));
        cmd.spawn()?;
        Ok(())
    }
}

#[cfg(all(not(target_family = "wasm"), target_os = "linux"))]
fn reveal_linux(path: &Path) -> std::io::Result<()> {
    // 1. If it is a directory, open the directory itself directly
    if path.is_dir() {
        return open::that(path);
    }

    // 2. For files, attempt to highlight/select the file in the file manager
    if let Ok(canonical) = path.canonicalize() {
        let uri = format!("file://{}", canonical.display());

        // Try FreeDesktop FileManager1 D-Bus method
        let dbus_status = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "--reply-timeout=500",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{uri}"),
                "string:",
            ])
            .status();

        if let Ok(status) = dbus_status
            && status.success()
        {
            return Ok(());
        }

        // Try KDE Dolphin native select
        if std::process::Command::new("dolphin")
            .arg("--select")
            .arg(&canonical)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // Try GNOME Nautilus native select
        if std::process::Command::new("nautilus")
            .arg("--select")
            .arg(&canonical)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    // 3. Fallback: Open parent directory with default opener
    reveal_fallback(path)
}

#[cfg(not(target_family = "wasm"))]
fn reveal_fallback(path: &Path) -> std::io::Result<()> {
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    open::that(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reveal_fallback_resolution() {
        let file_path = Path::new("/some/nested/directory/file.txt");
        let dir = if file_path.is_dir() {
            file_path
        } else {
            file_path.parent().unwrap_or(file_path)
        };
        assert_eq!(dir, Path::new("/some/nested/directory"));

        let root_dir = Path::new("/");
        let root_res = if root_dir.is_dir() {
            root_dir
        } else {
            root_dir.parent().unwrap_or(root_dir)
        };
        assert_eq!(root_res, Path::new("/"));
    }
}
