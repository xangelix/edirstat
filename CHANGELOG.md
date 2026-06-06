# Changelog

All notable changes to **eDirStat** will be documented in this file.

---

## [Unreleased]

### Added

- **📄 Changelog:** Created a clean, practical, and exciting changelog documenting all previous releases.
- **💖 GitHub Sponsors:** Added sponsor/funding configuration pointing to `xangelix` to support the project.
- **🍏 macOS Sandboxing Entitlements:** Packaged the macOS app with proper sandboxing entitlements for improved security and smoother OS integration.

---

## [v1.1.0] - 2026-06-05

### Added

- **⚡ Parallel Deduplication:** Hashing is now fully parallelized using the Rayon library, making duplicate checks dramatically faster on multi-core CPUs.
- **🗑️ Safe Trashing:** Added the option to move files to the system Trash/Recycle Bin instead of permanently deleting them, protecting you from accidental data loss.
- **🔗 Hardlink & Softlink Support:** You can now create and manage hard/soft links directly in the Deduplicator tab to reclaim space without losing file access.
- **📁 Local Directory Refresh:** Right-click a folder to refresh its contents instantly without having to scan your entire drive again.
- **📈 Duplicate Waste Chart:** Added a new visualization showing which file extensions are wasting the most space on duplicates.
- **🎯 Smart Duplicate Selection:** Added one-click helpers to select duplicates based on shortest, longest, oldest, newest, or root-most paths.
- **❌ Explorer Visuals:** Folders and files marked for deletion in the deduplicator are now highlighted directly in the main directory tree.

### Changed

- **⚙️ Layout Improvements:** Reorganized the Deduplicator tools into clean, logical dropdown menus and refined the color palette.
- **📦 Under-the-hood Refactoring:** Reorganized backend code modules for better performance and long-term stability.

---

## [v1.1.0-rc.1] - 2026-05-26

### Added

- **👥 First-Gen Deduplicator:** Introduced a new deduplication engine and UI tab to find and clean up byte-for-byte identical files.
- **🔍 Regex Explorer Search:** Added support for robust regular expression filtering in the directory explorer.
- **🌍 Commit Builds:** Set up automatic builds for macOS, Windows, and Linux on every commit to catch issues early.

### Changed

- **🖥️ Screen Friendly:** Adjusted the default window size to `1200x800` for a better fit on standard displays.
- **📊 Plot Organization:** Cleaned up GUI rendering by splitting up charts and plots into their respective modules.

### Fixed

- **🎨 View Tweaks:** Fixed the text color for "Monospace Paths" and updated the "Collapse All" menu option to use a clearer eject icon.

---

## [v1.0.10] - 2026-05-24

### Fixed

- **🍎 macOS Releases:** Fixed a file path typo in the release pipeline that prevented macOS aarch64 binaries from packaging correctly.

---

## [v1.0.9] - 2026-05-24

### Added

- **🍎 macOS Support:** Added macOS builds and native test coverage to our CI/CD pipelines to ensure macOS users get a first-class experience.

---

## [v1.0.8] - 2026-05-24

### Added

- **🪟 Windows Application Metadata:** Configured Windows resources so the executable displays the official eDirStat icon and details in File Explorer.

---

## [v1.0.7] - 2026-05-24

### Fixed

- **📄 Docs and Media:** Fixed a duplicate screenshot in the README and restored proper Git LFS settings for the screenshots directory.

---

## [v1.0.6] - 2026-05-24

### Fixed

- **🎨 Icon Quality:** Re-rendered the official eDirStat icon variants to scale perfectly across different screen resolutions.

---

## [v1.0.5] - 2026-05-24

### Changed

- **🧼 Repository Cleanup:** Removed heavy LFS (Large File Storage) configs and updated the main application icon to use a lightweight, non-LFS path.

---

## [v1.0.4] - 2026-05-24

### Added

- **🪟 Windows Builds:** Added Windows executable builds to the automated GitHub release matrix.

---

## [v1.0.3] - 2026-05-24

### Changed

- **🏷️ Binary Naming:** Renamed the compiled executable binary from `main` to `edirstat` for clarity.

---

## [v1.0.2] - 2026-05-24

### Changed

- **📄 Documentation:** Cleaned up code fence blocks and formatting in the README.

---

## [v1.0.1] - 2026-05-24

### Added

- **🎉 Initial Stable Release:** The first official release of **eDirStat**!
- **⚡ Lightning-Fast Directory Traverser:** A work-stealing, multithreaded directory walker that scans millions of files in seconds.
- **🗺️ Interactive Treemap:** A visual map of your storage using adaptive HSL gradients. Click blocks to find them in the tree view instantly.
- **💾 Fast Snapshots:** Save and load directory states in milliseconds using memory-mapped `.edst` files.
- **📈 Rich Analytics:** A suite of visual charts, including size distribution, file age vs. size, extension boxplots, and directory composition timelines.
- **🚀 Status & Control:** Live worker thread indicators and smooth, responsive navigation.
