# Changelog

All notable changes to **eDirStat** will be documented in this file.

---

## [Unreleased]

### Added

- **📄 Changelog:** Created a clean, practical, and exciting changelog documenting all previous releases.
- **💖 GitHub Sponsors:** Added sponsor/funding configuration pointing to `xangelix` to support the project.
- **🍏 macOS Sandboxing Entitlements:** Packaged the macOS app with proper sandboxing entitlements for improved security and smoother OS integration.
- **🗂️ WinDirStat Layout Mode:** Added a brand new WinDirStat-style layout mode for a classic, familiar disk usage visualization.
- **☑️ Multi-Select & Multi-Operations:** Added support for selecting multiple items and performing bulk operations in the GUI.
- **❓ "How it Works" Modal:** Added an informative explanation modal to help users understand the deduplication process.
- **🌳 Default Root Selection:** Automatically selects the root directory node upon loading, improving the initial navigation flow.
- **⚙️ EditorConfig:** Added an `.editorconfig` file to establish and enforce consistent coding styles across different editors.
- **🚀 CLI / Positional Arguments:** Added support for passing a directory path as a positional command-line argument to automatically start scanning on startup.
- **📊 Benchmark & Demo Media:** Added comparison benchmarks against QDirStat and WinDirStat in the README, along with high-quality demonstration videos (drag race, deduplicator demo, and WinDirStat layout mode).
- **🪟 Windows NTFS MFT Driver:** Integrated a new Windows-native NTFS driver utilizing the Master File Table (MFT) for near-instantaneous drive scanning on Windows.
- **⏱️ Scan Stats Persistence:** Retained the elapsed scan time and scanning speed in the status bar even after the scan has finished.
- **📋 Clipboard Tools:** Added "Copy Name" and "Copy Path" tools to easily copy file metadata from the directory tree.
- **🔆 Scan Button Highlight:** Added a subtle glow/highlight effect to the "Scan Directory" button when no scan has been run or directory data is empty.
- **📊 New Table Implementation:** Switched to a robust, feature-rich table view powered by `egui-table-kit`, supporting horizontal scrolling and cleaner row selection/operations traits.
- **📅 Created Time Column:** Added a "Created" column in the directory tree/explorer view (positioned before "Modified") to view file creation dates.
- **🔄 Root Refresh Support:** Restored the ability to refresh/rescan the root node directly in the directory explorer.
- **📦 Windows Installer (Inno Setup):** Added support for building proper Windows setup/installer binaries (`.exe` via Inno Setup) in the CI/CD pipeline on every commit.
- **⚖️ MIT License:** Formally initialized the repository with the MIT License.

### Changed

- **🎨 Rebranded Visuals:** Replaced existing logos and icons across the GUI and documentation with modern, high-quality SVG/raster variants, and added a utility script for icon generation.
- **⚡ Memory & Speed Optimizations:** Dramatically reduced allocations across the board by adopting `CompactString` in engine models, precalculating directory counts, and reducing deduplicator allocations.
- **⚡ Zero-Allocation & Branchless Search:** Implemented allocation-free, branchless case-insensitive search algorithms in the directory explorer for faster filtering.
- **⚡ Fast Traversal & SIMD:** Integrated SIMD loop optimizations and switched to stack-allocated `SmallVec` for tracking ancestors in deep directories.
- **⚡ Alloc-Free Treemap Rendering:** Optimized the interactive treemap with allocation-free file extension mapping.
- **🎨 UI & Layout Refinements:** Polished the user interface by organizing bottom panel controls, improving center panel padding, adjusting column spacing, and enhancing modal layouts with higher contrast.
- **🧹 Deduplicator Cleanup:** Disabled the file extensions panel while viewing the Deduplicator tab to reduce visual clutter, and cleaned up tab headings.
- **♻️ Recycle Icon:** Swapped the toolbar recycle icon with a larger, more visible version.
- **📦 Dependency Updates:** Bumped project dependencies to their latest versions for improved security and stability.
- **⚡ Treemap Render Optimizations:** Dramatically optimized interactive treemap rendering by switching to zero-allocation draw calls, discarding sub-pixel draws earlier, and adjusting the pixel precision limit to `0.2` for crisper rendering.
- **⚡ Extension List Caching:** Implemented a cached rendering strategy for file extension statistics, reducing draw calls by up to 50%.
- **⚡ Performance Polish:** Switched key hashing tables to `ahash`'s `RandomState` and replaced the extension map string types with `CompactString` to further minimize heap allocations, and reduced load on the current snapshot mutex lock.
- **✏️ Explorer Column Renaming:** Renamed the "Last Modified" column to "Modified" for a cleaner and more compact header layout.
- **🎨 UI Enhancements:** Improved the table operations buttons styling (removing gray backgrounds for a cleaner look) and styled warning/detail modals with stronger, higher-contrast text.
- **💾 Persistence Format V2:** Bumped the snapshot persistence header to `v2` to support new metadata features.
- **📦 Dependency Upgrades:** Upgraded underlying project dependencies, including updating `egui-table-kit` to version `0.1.5`.
- **🪟 Console-Free Windows Executable:** Configured the application so that the console command window does not spawn when launching the GUI on Windows.

### Fixed

- **🍏 macOS Packaging:** Fixed incorrect icon paths in the macOS CI/CD release configuration.
- **🖱️ Selection Behavior:** Restored the ability to de-select nodes in both the directory explorer and the interactive treemap.
- **🔘 Dropdown Interactivity:** Fixed the click hitbox for dropdown buttons in the directory explorer.
- **🪟 Windows Ancestor Traversal:** Resolved a bug in the directory traverser that caused incorrect ancestor resolution on Windows.
- **🖱️ Treemap & Scatter Plot Bugfixes:** Truncated treemap depth past a set limit to prevent rendering overflows, resolved a block selection issue in the treemap, and improved hover text wrapping on plots.
- **🪟 Windows Drive Resolution:** Fixed a path-resolution issue for Windows root drives in the directory allocator arena.
- **📦 Cross-Platform Compilation:** Fixed build configurations so that the `windows` crate dependency is only compiled on Windows targets and correctly supports MSVC builds.
- **📝 Clippy Warnings in Docs:** Fixed documentation formatting and naming to eliminate Clippy lints.
- **♻️ Cache Clearing on Refresh:** Fixed a bug where cached GUI elements were not cleared during a directory refresh or rescan.
- **🪟 Windows GUI Launching:** Fixed the entry point executable logic to properly apply the Windows GUI subsystem directive.

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
