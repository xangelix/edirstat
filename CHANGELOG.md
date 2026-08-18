# Changelog

---

## [v2.1.0] - 2026-08-17

**eDirStat 2.1.0 is our biggest quality-of-life release yet — eight languages, four themes, a browser-based snapshot viewer, NTFS MFT scanning on Linux, and a faster v3 snapshot format, all backed by 100+ new tests.**

> **✨ Highlights**
>
> - 🌍 **Speak your language** — Full GUI localization in 8 languages with a built-in selector
> - 🎨 **Your theme, your way** — System (auto), Dark, Light, and High Contrast themes
> - 🌐 **Snapshots in the browser** — Explore saved `.edst` scans online, no install required
> - 🐧 **MFT scanning on Linux** — NTFS Master File Table parsing joins Windows, extension records included
> - 💾 **v3 snapshots** — Compact columnar encoding: smaller files, faster loads (v2 files still open)
> - ⏹️ **Total scan control** — Cancel any scan mid-flight, or restrict it to a single filesystem
> - 🖥️ **New Scan dialog** — Pick from detected drives and mount points, no typing required
> - 🏁 **Termination race fixed** — A rare work-stealing early-exit bug is gone, with stress tests to prove it

**🔧 Under the hood:** the project is now three crates (`edirstat-core`, `edirstat`, `edirstat-gui`) for cleaner crates.io publishing · view preferences persist between sessions · `Del` / `Shift+Del` trash & delete shortcuts · configurable timestamp formats · UI stack upgraded to egui v0.36 · 100+ new tests now guard the arena, snapshot format, scan engine, and every statistics chart.

### Added

#### GUI Enhancements & Features

- **🌍 Full GUI Localization:** Migrated the entire interface to Fluent translations and added a language selector to the View menu, shipping with translations for Spanish, German (community-reviewed), Dutch, French, Portuguese, Italian, and Polish.
- **🎨 Theme System Overhaul:** Added a System theme with automatic runtime detection, plus brand-new High Contrast and Light themes.
- **🖌️ Treemap Styling Options:** Added a View option to draw borders around treemap rectangles, new treemap styles for improved visual clarity and contrast, and dedicated fallback coloring for directory leaf blocks.
- **🗂️ "New Scan" Modal:** Added a new scan dialog, redesigned to list detected drives and mount points for quick scanning, and defaulted the scan path input to the current working directory.
- **⏹️ Scan Cancellation:** Added the ability to cancel an in-progress directory scan.
- **🖇️ Same-Filesystem Scan Option:** Added a toggle to restrict traversal to the root filesystem and avoid crossing mount points.
- **📂 File Open Operation:** Added a `FileOpen` table operation to the directory explorer.
- **↕️ Sort Scrolling:** The directory explorer now scrolls to the relevant row after sorting by a column.
- **⌨️ Delete Shortcuts:** Added `Del` and `Shift + Del` keyboard shortcuts for trash and delete operations, along with an option to bypass their confirmation dialogs.
- **🕐 "Time Format" View Option:** Added a configurable timestamp display format for the directory explorer.
- **💾 View Preferences Persistence:** View options are now persisted to disk via a new preferences model and restored on launch.
- **🌳 Refresh State Restoration:** Open/expanded tree states are now restored after refreshing a directory.
- **🧮 Deduplicator Interaction Gating:** Deduplicator row interactions are now disabled until the deduplication run has fully completed.
- **📏 Layout Sizing:** Improved the table/statistics panel split sizing and the default name column width.

#### Web App & Browser Viewer

- **🌐 Browser Snapshot Viewer:** Split the project into three crates (`edirstat-core`, `edirstat`, `edirstat-gui`) and launched `edirstat-web`, a wasm32 browser build of the GUI acting as a pure snapshot viewer.
- **🌐 Wasm Demo Snapshot:** Added a wasm-ready MGS3 demo snapshot to the website.
- **🌐 Graceful Web Degradation:** Non-viable UI elements on the web build are now shown disabled with explanatory tooltips instead of being hidden.
- **🌐 Demo & Welcome Improvements:** Improved the demo and welcome screens across the GUI and website, plus website metadata improvements.
- **🎥 Comparison Videos:** Added WinDirStat and WizTree comparison videos to the website.

#### CLI & Headless Tooling

- **💾 Snapshot Extension Handling:** The `--to` headless snapshot mode now automatically appends the `.edst` extension when not specified, with a clearer help message.

#### Platform & Filesystem

- **🐧 Linux NTFS MFT Scanning:** NTFS drives can now be scanned via the Master File Table on Linux when permissions allow (#14), including information folded in from MFT extension records (#15).

#### Developer Tooling & Testing

- **🧪 Expanded Test Suite:** Added 100+ new unit and integration tests covering the arena, snapshot format, varint codecs, scan engine, coordinator, MFT parser, CLI, and all statistics charts.
- **🚀 itch.io Deploy Job:** Initialized a CI/CD job to deploy multivers binaries and installers to itch.io on release.
- **🛠️ Multi-Target Multivers Script:** Extended the local multivers builder to support multiple compilation targets.

### Changed

#### Performance & Allocations

- **⚡ V3 Snapshot Format:** Introduced a new columnar v3 snapshot persistence format for significant memory and CPU speedups.
- **⚡ Transparent Zstd Layer:** Made the snapshot zstd compression layer transparent to loaders and added an optional uncompressed mode (`--no-compression`).
- **⚡ Frozen String Pool:** Added a frozen variant of the arena `StringPool`, reducing indexing memory.
- **⚡ Compact Timestamps:** Removed access-time support and packed timestamps as `u32` epoch seconds.
- **⚡ BitSet Explorer Cache:** Switched cached explorer matches from `Vec<bool>` to a `BitSet`.
- **⚡ MFT Metadata Records:** Avoided collecting the first 16 (metadata) NTFS records during MFT scans.

#### Platform Compatibility & Hardening

- **🛡️ Snapshot Bounds Checking:** Significantly strengthened bounds checking in the snapshot persistence loader.

#### Project Maintenance & Build Configuration

- **📦 egui v0.36:** Upgraded the UI stack to egui v0.36 and `egui-table-kit` v0.6, vendoring `egui-notify` in-tree (upstream no longer updated).
- **📦 Dependency Updates:** Bumped project dependencies throughout, including `compact_str` v0.10 and the web/npm packages.
- **⚙️ Toolchain Update:** Updated the Rust nightly compiler toolchain to `nightly-2026-07-15`.
- **⚙️ GitHub Actions Updates:** Unified all CI/CD workflow actions to their latest major versions (`checkout@v7`, `upload-artifact@v7`, `download-artifact@v8`).
- **🧹 Engine Cleanup:** Refactored the traversal engine's worker context handle and renamed the deduplicator tab module.
- **🧹 Git Ignore Rules:** `target/` directories are now ignored at any repository depth.
- **📦 crates.io README:** The `edirstat` crate now renders the repository's root `README.md` on crates.io via the `readme` manifest field, retiring the separate crate-local blurb.
- **📄 Documentation:** Linked the project website from the README.

### Fixed

#### Directory Scanner & Deduplicator

- **🏁 Traversal Termination Race:** Fixed a work-stealing termination race in the parallel scanner that could truncate scans early on deep directory trees — an idle worker could declare completion while tasks sat unpicked in local worker queues. Termination is now driven by an in-flight task counter, backed by multithreaded stress regression tests.

#### GUI & Interactive Elements

- **🖱️ Hover Occlusion:** Fixed the table row hover effect being occluded (via `egui-table-kit` bump).
- **🔺 Expand/Collapse Triangle:** Fixed the click hitbox for the expand/collapse triangle in the classic view.
- **🎨 System Theme Detection:** Fixed System theme reading on modern egui.
- **🖼️ Modal Icons:** Migrated modal icons to PNG, avoiding SVG text rendering issues.
- **📐 View Dropdown Icons:** Fixed an icon alignment inconsistency in the View dropdown.
- **🔄 Refresh Timing Cache:** Fixed directory refresh incorrectly re-using cached timing results.
- **♻️ Cache Invalidation:** Made GUI cache invalidation robust across data changes.
- **🌳 Tree Header Render:** Fixed the explorer tree header rendering on error, and a header padding clip issue.
- **🌐 wasm Gating:** Improved feature gates for wasm no-op paths, and fixed feature gating of the `egui-async` image loader call.

#### Cross-Platform & Windows

- **🪟 Windows Cancel Token:** Corrected the cancel token arguments for the Windows traversal path.

#### Localization & Web

- **🌍 Translation Fixes:** Corrected the Dutch `selection-items` string, and resolved German translation conflicts and typos.
- **🌐 Favicon Packaging:** Corrected asset packaging for the website favicon.

#### Project Maintenance

- **📦 crates.io Packaging:** Moved compile-time-embedded assets (licenses, locales, and images) into the `edirstat-gui` crate so every published crate builds standalone on crates.io, backed by a new regression test.
- **🍏 macOS Bundle Versioning:** Fixed stale hardcoded version strings in macOS app bundles; `CFBundleShortVersionString` is now generated from `Cargo.toml` in both CI/CD workflows.
- **🧪 Case-Sensitivity Test Portability:** Fixed the `$MFT` probing tests to detect filesystem case sensitivity at runtime instead of assuming case-sensitive semantics, resolving macOS CI failures.
- **🔢 Version Sync:** Synchronized version numbers across all manifests.

### 💖 Contributors & Thanks

- **[@Lej77](https://github.com/Lej77)** — for the NTFS `$MFT` fixes and improvements, including Master File Table scanning of NTFS drives on Linux (#14) and support for folding in MFT extension records (#15).
- **[@AlexanderSchuetz97](https://github.com/AlexanderSchuetz97)** — for the many feature requests, German translation review, and extensive bug testing that helped shape this release.

---

## [v2.0.1] - 2026-06-29

### Added

#### Documentation & Media

- **📽️ Video Demos:** First frame thumbnail for comparison videos previewing correctly on GitHub

### Changed

#### Project Maintenance

- **📦 Crate Metadata:** Refined the project description in `Cargo.toml` to explicitly highlight the deduplication capabilities.

### Fixed

#### GUI & Interactive Elements

- **📊 Snapshot Status Bar:** Restored total files, directories, and byte counts in the bottom status bar when loading a saved `.edst` snapshot file.

#### Build Configuration & Tooling

- **🛠️ Multi-CPU Target Builder:** Fixed the target-CPU optimized builder script (`build_local_multivers.sh`) to conditionally compile static linking flags on supported targets, cleanly purge conflicting build suffixes, and enforce the default system linker to avoid compilation failures on Linux GNU targets.

---

## [v2.0.0] - 2026-06-21

Version 2.0.0 focuses on performance optimizations, architectural hardening, and broader platform compatibility. Key updates include the integration of a Windows-native NTFS Master File Table (MFT) driver for faster scanning, a classic WinDirStat-style layout option, and headless command-line snapshot capabilities. Additionally, this release refactors several critical paths—such as transitioning snapshot persistence to a safe, Zstd-compressed format and replacing unsafe operations with compiler-optimized safe Rust—to improve reliability and resource efficiency across all supported operating systems.

The project also has a new website at: [edirstat.com](https://edirstat.com)!

### 📦 Distribution Model & Project Support

eDirStat is, and will always remain, entirely open-source and free. However, to support continuous development, engineering, and maintenance costs, **official pre-compiled binaries for Windows, macOS, and Linux are distributed exclusively as a paid download via [Itch.io - xangelix.itch.io/edirstat](https://xangelix.itch.io/edirstat)**.

- **100% Open Source:** The complete codebase is free of charge and publicly available under the permissive **MIT License**. If you prefer to build the tool yourself, you are welcome to compile the source code on Windows, Linux, or macOS at no cost. Instructions are provided in the `README.md`.
- **Package Managers:** Package managers and their repository maintainers are welcome to bundle and distribute eDirStat through their respective channels at no cost, provided that it is distributed un-paid, "unofficial", and under the MIT License.

Purchasing precompiled packages directly funds the engineering efforts required to keep eDirStat fast, secure, and compatible with the latest operating systems.

### Added

#### Windows & NTFS Integration

- **🪟 Windows NTFS MFT Driver:** Integrated a new Windows-native NTFS driver utilizing the Master File Table (MFT) for near-instantaneous drive scanning on Windows.
- **🪟 Administrator Restart Modal:** Added a prompt to restart the application as administrator to allow Master File Table (MFT) access on Windows.
- **🪟 Windows UNC Path Stripping:** Automatically strip UNC volume prefixes (such as `\\?\`) from directory paths in all visual components.

#### GUI Enhancements & Features

- **🗂️ WinDirStat Layout Mode:** Added a brand new WinDirStat-style layout mode for a classic, familiar disk usage visualization.
- **☑️ Multi-Select & Multi-Operations:** Added support for selecting multiple items and performing bulk operations in the GUI.
- **❓ "How it Works" Modal:** Added an informative explanation modal to help users understand the deduplication process.
- **🌳 Default Root Selection:** Automatically selects the root directory node upon loading, improving the initial navigation flow.
- **⏱️ Scan Stats Persistence:** Retained the elapsed scan time and scanning speed in the status bar even after the scan has finished.
- **⏱️ Millisecond Scan Timing:** Switched the GUI elapsed scan time to display millisecond-level precision.
- **📋 Clipboard Tools:** Added "Copy Name" and "Copy Path" tools to easily copy file metadata from the directory tree.
- **🔆 Scan Button Highlight:** Added a subtle glow/highlight effect to the "Scan Directory" button when no scan has been run or directory data is empty.
- **📊 New Table Implementation:** Switched to a robust, feature-rich table view powered by `egui-table-kit`, supporting horizontal scrolling and cleaner row selection/operations traits.
- **📅 Created Time Column:** Added a "Created" column in the directory tree/explorer view (positioned before "Modified") to view file creation dates.
- **🔄 Root Refresh Support:** Restored the ability to refresh/rescan the root node directly in the directory explorer.
- **ℹ️ Update Checker:** Added an update checker to the "About" modal to easily check for new versions of the application.
- **📊 Item and Directory Count Sorting:** Added full support for sorting the directory explorer view by item and subdirectory counts.
- **🛡️ Restricted Permissions Handling:** Added detection and safe handling of folders with restricted access/permissions, marking them as "No Permission" in the UI rather than failing the scan.
- **🔔 Toast Notifications:** Integrated interactive toast notifications (success, warning, error, info) for file manager operations, terminal launching, and deduplication hardlinking/softlinking.
- **🎨 Squared Logo Asset:** Created a new subtext-less squared SVG logo variant for keyart and promotional purposes.
- **⚖️ Dependency Licenses Modal:** Added an open source license viewing modal inside the About dialog to display packaged licenses for dependencies.

#### CLI & Headless Tooling

- **🚀 CLI / Positional Arguments:** Added support for passing a directory path as a positional command-line argument to automatically start scanning on startup.
- **🚀 Headless Snapshot Mode:** Added a headless command-line mode to run directory scans and automatically save results to a `.edst` snapshot file, bypassing the GUI.

#### Project Infrastructure & Packaging

- **🍏 macOS Sandboxing Entitlements:** Packaged the macOS app with proper sandboxing entitlements for improved security and smoother OS integration.
- **📦 Windows Installer (Inno Setup):** Added support for building proper Windows setup/installer binaries (`.exe` via Inno Setup) in the CI/CD pipeline on every commit.
- **⚙️ EditorConfig:** Added an `.editorconfig` file to establish and enforce consistent coding styles across different editors.
- **⚖️ MIT License:** Formally initialized the repository with the MIT License.
- **💖 GitHub Sponsors:** Added sponsor/funding configuration pointing to `xangelix` to support the project.
- **📄 Changelog:** Created a clean, practical, and exciting changelog documenting all previous releases.
- **📊 Benchmark & Demo Media:** Added comparison benchmarks against QDirStat and WinDirStat in the README, along with high-quality demonstration videos (drag race, deduplicator demo, and WinDirStat layout mode).
- **🛡️ Forbid Unsafe Code Directive:** Added `#![forbid(unsafe_code)]` at both binary and library crate entry points to ensure compilation-level code safety.
- **📦 Multi-Target CPU Windows Runner:** Integrated `cargo-multivers` builder to compile distinct, target-CPU optimized Windows binaries and package them inside a single wrapper crate (`runner/`) using `multivers-runner`.
- **⚖️ Packaged License Generation:** Integrated `cargo-about` to generate target-specific open source license lists (`assets/licenses/<target>.md`) and packaged them into the binary using `include_packed`.

#### Web App & Simulator

- **🌐 eDirStat Web V2:** Redesigned and launched the official product website (`/web`) featuring an interactive, live simulator for the cryptographic deduplication pipeline and treemap dashboard.
- **🌐 Responsive Displays & Ratios:** Expanded CSS layout grids and flexbox viewport definitions to support a wide range of mobile and ultra-wide monitor aspect ratios without clipping.
- **🌐 Deduplicator Simulator:** Built a fully client-side animated simulator of the eDirStat deduplication process, showing steps like size partitioning, header hashing, midpoint/suffix matching, and BLAKE3 checking.
- **🌐 Speed Multipliers:** Added scanning speed multipliers comparison to the website simulator charts.
- **🌐 Favicon Integration:** Added a website favicon referenced from `assets/img/`.

#### Developer Tooling & Testing

- **🛠️ Tracy Profiling:** Integrated basic support for Tracy-based profiling to assist in performance diagnostics.
- **🧪 Typos Check CI/CD:** Initialized automated spelling/typos check workflow in the CI/CD pipeline.
- **🧪 Criterion MFT Benchmarks:** Added a benchmarking suite using `criterion` to measure and track Master File Table (MFT) parsing performance.
- **🧪 Comprehensive Test Suite:** Added 55 new unit and integration tests across the codebase, significantly expanding test coverage for charts, model arena, persistence, and directory traversal.

### Changed

#### Performance & Allocations

- **⚡ Memory & Speed Optimizations:** Dramatically reduced allocations across the board by adopting `CompactString` in engine models, precalculating directory counts, and reducing deduplicator allocations.
- **⚡ Parallel MFT Parsing & Sharded Pool:** Parallelized MFT record processing and introduced a sharded `StringPool` to maximize multi-core CPU utilization, yielding a ~49% speedup.
- **⚡ MFT Flat Representation:** Implemented a flat memory representation for hierarchy resolution in the MFT driver, reducing traversal times by ~9.6%.
- **⚡ Sector-Aligned DMA Reads:** Implemented unbuffered, sector-aligned DMA read operations in the MFT driver to bypass OS caching and accelerate disk reading by ~6.9%.
- **⚡ Manual AVX2 UTF-16 Decoder:** Implemented a manual AVX2-optimized UTF-16 decoder for rapid MFT filename parsing, improving speed by ~4.6%.
- **⚡ MFT Allocation Reduction:** Replaced standard vectors with stack-allocated `SmallVec` in parsed MFT models, reducing heap allocations and boosting speed by ~3.7%.
- **⚡ MFT String Interning & Cache Scaling:** Switched to CRC32-based string interning and scaled up the L1 cache size in the MFT engine for faster path/name resolutions.
- **⚡ SIMD UTF-16 Decoder SSE Fallback:** Added SSE fallback support to the SIMD-accelerated UTF-16 filename decoder in the MFT engine.
- **⚡ Bottom-Up Size Propagation:** Optimized size calculations in the engine coordinator by adopting a bottom-up propagation strategy.
- **⚡ Zero-Allocation & Branchless Search:** Implemented allocation-free, branchless case-insensitive search algorithms in the directory explorer for faster filtering.
- **⚡ Fast Traversal & SIMD:** Integrated SIMD loop optimizations and switched to stack-allocated `SmallVec` for tracking ancestors in deep directories.
- **⚡ Alloc-Free Treemap Rendering:** Optimized the interactive treemap with allocation-free file extension mapping.
- **⚡ Treemap Render Optimizations:** Dramatically optimized interactive treemap rendering by switching to zero-allocation draw calls, discarding sub-pixel draws earlier, and adjusting the pixel precision limit to `0.2` for crisper rendering.
- **⚡ Extension List Caching:** Implemented a cached rendering strategy for file extension statistics, reducing draw calls by up to 50%.
- **⚡ Performance Polish:** Switched key hashing tables to `ahash`'s `RandomState` and replaced the extension map string types with `CompactString` to further minimize heap allocations, and reduced load on the current snapshot mutex lock.
- **⚡ Internment Migration:** Switched file name storage in the arena to use `xgx_intern` for faster lookup and reduced heap allocations.
- **⚡ Linux Explorer Performance:** Cached file permissions details and read owner UIDs/GIDs natively through `libc` to boost explorer rendering speed on Linux.
- **⚡ Extension Color Caching:** Cached file extension color calculations in the GUI theme to reduce CPU load during rendering.
- **⚡ Mimalloc Integration:** Configured `mimalloc` v3 as the default global allocator to accelerate memory allocations across multi-core systems.
- **⚡ Deduplicator Optimizations:** Deferred path string allocations in the deduplicator to minimize heap usage during search.
- **⚡ Zero-Copy Node Storage:** Restored true zero-copy node storage in the file arena to reduce latency and memory overhead.
- **⚡ Lowercase Extension Helper:** Switched to a shared, stack-allocated `with_lowercase_ext` helper for file extension mappings to avoid heap allocations.
- **⚡ Zstd Compression for Persisted Snapshots:** Migrated the snapshot persistence format to a compressed Zstd payload, replacing unsafe copy-on-write memory mapping (`memmap2`) with safe heap allocations during decompression to improve file safety and reliability.
- **⚡ Safe UTF-16 Decoder Optimization:** Replaced unsafe SIMD AVX2/SSE implementations with safe Rust iterators and boundary checks that optimize down to SIMD via LLVM.

#### UI, UX & Visual Polish

- **🎨 Rebranded Visuals:** Replaced existing logos and icons across the GUI and documentation with modern, high-quality SVG/raster variants, and added a utility script for icon generation.
- **🎨 UI & Layout Refinements:** Polished the user interface by organizing bottom panel controls, improving center panel padding, adjusting column spacing, and enhancing modal layouts with higher contrast.
- **🧹 Deduplicator Cleanup:** Disabled the file extensions panel while viewing the Deduplicator tab to reduce visual clutter, and cleaned up tab headings.
- **♻️ Recycle Icon:** Swapped the toolbar recycle icon with a larger, more visible version.
- **✏️ Explorer Column Renaming:** Renamed the "Last Modified" column to "Modified" for a cleaner and more compact header layout.
- **🎨 UI Enhancements:** Improved the table operations buttons styling (removing gray backgrounds for a cleaner look) and styled warning/detail modals with stronger, higher-contrast text.
- **🎨 Treemap Coloring:** Switched to gamma multiplication for interactive treemap color scaling, yielding smoother visual gradients.
- **📄 Zero-Copy Documentation:** Updated website templates, README, and GUI about modals to reflect the migration from zero-copy memory mapping to safe Zstd-compressed snapshots.
- **🌐 Web Disk Simulator Polish:** Cleaned up disk names and models in the website simulator.

#### Platform Compatibility & Hardening

- **🛡️ Persistence Hardening:** Improved overflow protection for memory-mapped persistence files.
- **🗂️ Platform-Native Path Slashes:** Improved cross-platform path slash handling to ensure correct directory traversal and representation.
- **🛡️ Treemap Bounds Protection:** Added robust bounds checks and protections when selecting the root node in the interactive treemap.
- **🛡️ NTFS MFT Driver Hardening:** Significantly hardened the Windows-native NTFS MFT driver with extra safety checks and bounds validation.
- **🛡️ MFT Cross-Platform Separation:** Separated Windows-specific MFT APIs from non-Windows target modules to keep the core engine portable and clean.
- **🪟 MFT Ingestion Refactoring:** Consolidated the Master File Table (MFT) ingestion loop into a dedicated helper function for cleaner, more maintainable code.
- **🧹 Walk Context Refactoring:** Refactored the recursive directory scanner context into a unified `WalkCtx` struct for cleaner and safer code.
- **🐧 Unix Root Scanning Improvements:** Enhanced traversal of the system root on Unix/Linux by automatically filtering out virtual/system directories (such as `/proc`, `/sys`, `/dev`, etc.) and allowing local partition crossing.
- **🛡️ Safe User & Group Resolution:** Replaced unsafe `libc` calls for retrieving Unix usernames and group names with safe APIs from the `uzers` crate.
- **🛡️ Windows Subsystem Execution:** Adopted the `cli-or-gui` crate to dynamically check privileges and run commands, replacing custom platform-specific elevation blocks.
- **🛡️ Elevation Helpers Refactoring:** Cleaned up custom Windows elevation and admin restart APIs by adopting a unified implementation from the `cli-or-gui` crate.
- **🛡️ Safe MFT Chunk Ingestion:** Refactored the MFT ingestion Rayon thread boundaries to split mutable slices safely, completely avoiding unsafe raw pointer calls to `from_raw_parts_mut`.
- **🛡️ Safe Windows FS Type Check:** Replaced unsafe Win32 calls to `GetVolumeInformationW` with safe cross-platform drive inspection APIs from the `sysinfo` crate.

#### Project Maintenance & Build Configuration

- **📦 Dependency Updates:** Bumped project dependencies to their latest versions for improved security and stability.
- **💾 Persistence Format V2:** Bumped the snapshot persistence header to `v2` to support new metadata features.
- **📦 Dependency Upgrades:** Upgraded underlying project dependencies, including updating `egui-table-kit` to version `0.2.1` and integrating `egui-notify`.
- **🪟 Console-Free Windows Executable:** Configured the application so that the console command window does not spawn when launching the GUI on Windows.
- **📦 Release Build Optimizations:** Enabled Link-Time Optimization (LTO thin), symbol stripping, and restricted codegen-units to 1 for release binaries to minimize executable size and improve performance.
- **📄 Documentation & Benchmarks:** Updated the README to list speedup results for WinDirStat, include new context menu options, document positional arguments, refresh performance comparison benchmarks, clean up key feature wording, and link the license locally.
- **⚙️ Toolchain Update:** Updated the local Rust compiler toolchain to `nightly-2026-06-13`.
- **⚙️ Safe Exit Flow:** Avoided calls to `std::process::exit()` in the binary to ensure standard library destructors run cleanly on exit.
- **📄 README Visual Updates:** Combined primary logo images, added a treemap screenshot under the logo, switched to a cleaner treemap-b variant logo, and replaced the drag race video with a first-frame thumbnailed version to speed up page load.
- **⚙️ Toolchain Update:** Updated Rust nightly compiler toolchain to `nightly-2026-06-15`.
- **📦 Dependency Cleanup:** Removed the direct dependency on `libc` as it is no longer required.
- **📦 Dependency Upgrades:** Upgraded underlying project dependencies, including bumping `bytes` and `wayland-protocols` to their latest versions.
- **📦 Dependency Upgrades:** Upgraded `arrayvec` to `0.7.7`, `sysinfo` to `0.39.4`, and `log` to `0.4.33`.
- **📄 README Benchmarks Update:** Updated README benchmarks with program versions, a WizTree drag race video comparison, and updated WinDirStat speed multipliers.
- **📄 Benchmark Disclaimers:** Added program versions and benchmark methodology disclaimers to the README and website.
- **📦 Minimum Cargo Package Files:** Add `include` to `Cargo.toml` to include only necessary files in the package.

### Fixed

#### GUI & Interactive Elements

- **🖱️ Selection Behavior:** Restored the ability to de-select nodes in both the directory explorer and the interactive treemap.
- **🔘 Dropdown Interactivity:** Fixed the click hitbox for dropdown buttons in the directory explorer.
- **🖱️ Treemap & Scatter Plot Bugfixes:** Truncated treemap depth past a set limit to prevent rendering overflows, resolved a block selection issue in the treemap, and improved hover text wrapping on plots.
- **♻️ Cache Clearing on Refresh:** Fixed a bug where cached GUI elements were not cleared during a directory refresh or rescan.
- **🌐 Web Layout Clipping:** Fixed layout clipping bugs on thin displays and small viewports by setting appropriate `overflow-x` properties and max-widths.
- **🌐 WizTree Speed Metrics:** Corrected the WizTree benchmark time representation on the website simulator from 4.41s to 4.38s.

#### Cross-Platform & Windows MFT

- **🪟 Windows Ancestor Traversal:** Resolved a bug in the directory traverser that caused incorrect ancestor resolution on Windows.
- **🪟 Windows Drive Resolution:** Fixed a path-resolution issue for Windows root drives in the directory allocator arena.
- **📦 Cross-Platform Compilation:** Fixed build configurations so that the `windows` crate dependency is only compiled on Windows targets and correctly supports MSVC builds.
- **🪟 Windows GUI Launching:** Fixed the entry point executable logic to properly apply the Windows GUI subsystem directive.
- **🪟 UNC Volume Paths:** Resolved an issue where UNC volume paths were not handled correctly in the Windows MFT driver.
- **🪟 UNC Visual Path Cleanups:** Fixed missing UNC path cleanups across several modals, operations, and explorer views in the GUI.
- **🧪 Test Suite Fixes:** Updated path reconstruction unit tests for Windows drive compatibility.
- **📦 Unix-only Dependencies:** Restricted the `uzers` dependency compilation to Unix/Linux targets to avoid MSVC/Windows compile failures.

#### Directory Scanner & Deduplicator

- **🧹 Deduplication Group Validation:** Fixed a bug in the deduplicator where error-prone groups were not disqualified properly.
- **🌀 Directory Cycling Prevention:** Fixed a potential infinite loop or dir cycling bug during recursive `walk_dir` scanning.
- **⏱️ Epoch Formatting Overflow Protection:** Added checks to prevent year overflow in Unix timestamp date/time conversion.

#### Documentation & Build Pipelines

- **🍏 macOS Packaging:** Fixed incorrect icon paths in the macOS CI/CD release configuration.
- **🧪 CI/CD Toolchain Pinning:** Updated CI/CD test and release workflows to compile with `nightly-2026-06-13` to match the local toolchain.
- **📝 Clippy Warnings in Docs:** Fixed documentation formatting and naming to eliminate Clippy lints.
- **🧪 Clippy Lints:** Resolved new miscellaneous Clippy warnings introduced by the latest Rust nightly compiler.
- **🧪 Safe UID Checks in Tests:** Replaced unsafe direct `libc::getuid()` calls in unit tests with a safe command execution of `id -u`.
- **🧪 Clippy Lints:** Resolved clippy warnings around thread-local constant initializers on Windows targets.
- **🌐 Web Deployment fixes:** Switched the automated deployment of the website to native GitHub Pages actions and ensured logo assets are properly retained in the build directories.
- **🧪 CI/CD License Tests:** Removed non-deterministic `cargo-about` license checks from CI/CD pipeline tests.
- **🧪 CI/CD Safe Directory:** Configured Git safe directory settings in CI/CD runner to resolve checkout permission errors.
- **📝 Typos Whitelist:** Added the assets directory to the typos configuration whitelist.

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
