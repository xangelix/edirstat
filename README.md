# eDirStat

![eDirStat Logo](assets/img/logo.svg)

[![Crates.io](https://img.shields.io/crates/v/edirstat)](https://crates.io/crates/edirstat)
[![Docs.rs](https://docs.rs/edirstat/badge.svg)](https://docs.rs/edirstat)
[![License](https://img.shields.io/crates/l/edirstat)](https://spdx.org/licenses/MIT)

**eDirStat** is a modern, high-performance, cross-platform disk usage analyzer written in Rust. Inspired by legacy utilities like [WinDirStat](https://windirstat.net/), it leverages an immediate-mode graphical interface [`egui`](https://egui.rs/) to provide a real-time, interactive treemap visualization of your filesystem.

Unlike traditional analyzers that crawl sequentially, **eDirStat** is built from the ground up for modern multi-core systems. It couples a blazing-fast, work-stealing multithreaded directory walker with a zero-copy arena data structure. This allows you to scan millions of files seamlessly, visualize space hogs instantly via adaptive HSL gradients, and save/load system snapshots in milliseconds using memory-mapped files.

---

## 📽️ Demo Video

<https://github.com/user-attachments/assets/cad3056f-1d2c-45da-9827-3d0a90b8066a>

---

## 📸 Screenshots

![eDirStat Main Interface - Directory Tree and Interactive Treemap](docs/screenshots/main_interface.jpg)

![eDirStat Deduplicator - Deduplication File Scan](docs/screenshots/deduplicator.png)

![eDirStat Plots - File Size Distribution](docs/screenshots/file_size_distribution.jpg)

![eDirStat Plots - File Age vs File Size](docs/screenshots/file_age_vs_file_size.jpg)

![eDirStat Plots - Directory Composition](docs/screenshots/directory_composition.jpg)

![eDirStat Plots - File Sizes by Extension](docs/screenshots/file_sizes_by_extension.jpg)

![eDirStat Plots - Linked Temporal Timelines](docs/screenshots/linked_temporal_timelines.jpg)

![eDirStat Plots - Duplicate Waste by Extension](docs/screenshots/duplicate_waste_by_extension.png)

---

## 🚀 Key Features

- ⚡ **Work-Stealing Multi-threading:** Powered by a lock-free task injector queue for optimal utilization of all CPU cores during directory traversal.
- 👥 **Multi-Stage Deduplication Engine:** Detects byte-for-byte identical files using an optimized 7-stage hashing pipeline with full hardlink awareness.
- 📦 **Zero-Copy Serialization:** Uses binary snapshot layouts that can be instantly mapped into memory via `memmap2` and cast via `bytemuck`, bypassing traditional parsing overhead.
- 📊 **Dynamic Treemap Visualization:** An interactive, responsive layout canvas that slices and dices data, using stable extension hashing for color-coded grouping.
- 🔀 **Lazy Tree View:** Fluid interface navigation with automatic sibling-size sorting and recursive guidance lines.
- 🛡️ **Safe & Native:** Built completely in safe, pure Rust with immediate-mode UI rendering and cross-platform path handling.

---

## 🚀 Installation & Build

### Prerequisites

Ensure you have the latest stable Rust toolchain installed.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/xangelix/edirstat.git
cd edirstat

# Build the release executable
cargo build --release
```

The compiled binary will be located at `target/release/edirstat`.

> **Note:** When building on **Windows** you must use the nightly compiler, as `edirstat` requires the nightly feature `windows_by_handle`.

---

## 📖 Usage Guide

Run the application from the command line:

```bash
./target/release/edirstat
```

### Navigating the User Interface

1. **Scan a Directory:**
   Click the **📁 Scan Directory** button in the top menu bar to open a folder picker. Select the target drive or folder to initiate the scan.
2. **Explore the Tree:**
   The left-hand panel displays a hierarchical directory explorer. You can expand/collapse folders using the `[+]`/`[-]` toggles. Use the **🔍 Filter** input bar to narrow down the view to matching folders or files.
3. **Interact with the Treemap:**
   The central panel displays a visual representation of your disk space. Larger rectangles correspond to larger files or directories.
   - **Hovering:** Move your cursor over a block to view its full path and size in a tooltip.
   - **Clicking:** Click on a block to automatically select it in the directory tree on the left.
4. **Inspect File Extensions:**
   The right panel displays a sorted list of file extensions detected during the scan, complete with color-coded markers.
5. **Deduplicate Your Drive:**
   Switch to the **👥 Deduplicator** tab to search for duplicate files on your scanned filesystem. Custom selection helpers allow you to automatically select duplicates while preserving the oldest, newest, or shortest-path file.
6. **Context Actions:**
   Right-click any item in the left-hand explorer to open a context menu.
   - **Open in File Manager:** Launches your operating system's default file browser (Explorer, Finder, or Files) at the selected path.
   - **Delete (Permanent):** Opens a safety dialog to permanently delete the target path from your disk.

---

## 💾 Saving & Loading Snapshots

If you need to analyze a server or remote environment:

1. Scan the directory and click **💾 Save Snapshot** to write the structured tree to an `.edst` file.
2. Transfer the file to another machine.
3. Launch `edirstat` and click **📖 Load Snapshot** to open and navigate the tree with full interactivity, requiring no active filesystem connection.

---

## 🛠 Architectural Design & Internals

### 1. Parallel Work-Stealing Walker (`src/traversal.rs`)

The traversal engine avoids the performance bottlenecks of standard recursive single-threaded walkers. It utilizes `crossbeam-deque` for task scheduling:

- **Workers & Stealers:** Each parallel thread operates on a local thread-safe FIFO task queue. When a thread runs out of directories to scan, it attempts to steal tasks from a global injector or peer worker queues.
- **Cycle Detection:** Avoids infinite directory loops (caused by recursive symbolic links) by checking filesystem identity descriptors (`dev`/`ino` on Unix, and `volume_serial_number`/`file_index` on Windows) against an inherited stack of ancestors.
- **Ignore Matching:** Evaluates file pathways against globally defined structures and localized directory-level `.gitignore` files using compiled `globset` configurations.

### 2. Lock-Free Snapshot Commit Loop (`src/coordinator.rs`)

To prevent traversal worker threads from blocking the UI rendering cycle, `edirstat` decouples directory scanning from interface updates through an event-driven coordinator model:

- **The Coordinator:** Worker threads stream compressed structural events (`ScanEvent`) over a lock-free channel to a dedicated background Coordinator thread.
- **Dynamic ID Map:** The Coordinator translates worker-local task identifiers to global array indexes in $O(1)$ amortized time.
- **Atomic Snapshot Publishing:** Instead of locking a mutable tree, the GUI accesses an immutable `FileArenaSnapshot` read-only copy via `arc_swap`. The Coordinator issues updated snapshots to the GUI every 100 milliseconds during an active scan.

### 3. Cache-Friendly Arena Representation (`src/arena.rs`)

To conserve system memory and minimize pointers, the scanned directory hierarchy is flattened into a single contiguous array (arena):

```text
[ Root Node ] ---> [ Child A ] ---> [ Child B ] ---> [ Child C ]
                        |
                        v
                 [ Sub-child 1 ]
```

- **No Pointer Chasing:** Individual `FileNode` blocks reference their parents, first-born children, and next siblings through raw `u32` indices rather than heap-allocated pointers (`Box` or `Rc`).
- **Plain Old Data (POD):** The `FileNode` struct is annotated with `bytemuck::Pod` and `bytemuck::Zeroable` and is strictly aligned to 8 bytes to prevent uninitialized memory gaps.
- **Compact String Pool:** Names of files and folders are deduplicated and written into a contiguous byte sequence (`StringPool`). Nodes keep a simple index wrapper (`StringId`), minimizing allocations for duplicate names like `node_modules` or `.git`.

### 4. Zero-Copy Snapshot Persistence (`src/persistence.rs`)

The `.edst` snapshot file layout matches the structure of the in-memory arena:

```text
+------------------------------------------------------------+
|  Header (32 Bytes)                                         |
|  - Magic: "EDST"                                           |
|  - Version: u16                                            |
|  - Node Count: u64                                         |
|  - String Pool Offset & Length                             |
+------------------------------------------------------------+
|  Array of FileNode Structs (Flat Binary Segment)           |
+------------------------------------------------------------+
|  String Pool Data (Serialized Offsets + Packed UTF-8 Bytes)|
+------------------------------------------------------------+
```

- **Memory-Mapped Loading:** Loading a snapshot uses copy-on-write virtual memory maps (`memmap2`).
- **Zero Parsing Overhead:** Because `FileNode` is a POD structure, the loaded byte buffer is safely cast directly to a slice of `&[FileNode]`. This yields instant loading, even for files tracking millions of objects.

### 5. Multi-Stage Deduplication Engine (`src/stats/deduplicator.rs`)

The deduplication module detects byte-for-byte identical files with minimal disk I/O. Candidate duplicate groups are identified and isolated through a 7-stage pipeline:

1. **Size Partitioning:** Scanned files are grouped by identical byte counts. Singleton sizes are discarded immediately.
2. **Prefix Hashing:** Worker threads read and hash the first 4KB of files to filter out non-matching formats.
3. **Midpoint Hashing:** Computes a hash around the center of the remaining files to detect differences inside similar files.
4. **Suffix Hashing:** Hashes the last 4KB of file data, which often contains unique trailing metadata.
5. **Multi-Range Hashing:** Performs periodic block sampling (every 100MB) across large files to ensure long-distance uniformity without scanning entire gigabyte-scale structures.
6. **Full Cryptographic Hashing:** Executes a full BLAKE3 cryptographic hash only over candidates that successfully cleared the previous five stages.
7. **Real-time Validation:** Performs timestamp checking on disk immediately before grouping and action triggers to protect you against modifying files changed since snapshot generation.

The engine remains hardlink-aware, allowing it to accurately differentiate between physical duplicate copies and single-inode hardlinks, which consume no additional storage.

---

## 📝 License

This project is licensed under the [MIT License](https://spdx.org/licenses/MIT).
