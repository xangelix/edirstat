# eDirStat

![eDirStat Logo](assets/img/logo.svg)

[![Crates.io](https://img.shields.io/crates/v/edirstat)](https://crates.io/crates/edirstat)
[![Docs.rs](https://docs.rs/edirstat/badge.svg)](https://docs.rs/edirstat)
[![License](https://img.shields.io/crates/l/edirstat)](https://spdx.org/licenses/MIT)

**eDirStat** is a modern, high-performance, cross-platform disk usage analyzer written in Rust. Inspired by legacy utilities like [WinDirStat](https://windirstat.net/), it leverages an immediate-mode graphical interface [`egui`](https://egui.rs/) to provide a real-time, interactive treemap visualization of your filesystem.

Unlike traditional analyzers that crawl sequentially, **eDirStat** is built from the ground up for modern multi-core systems. It couples a blazing-fast, work-stealing multithreaded directory walker with a zero-copy arena data structure. This allows you to scan millions of files seamlessly, visualize space hogs instantly via adaptive HSL gradients, and save/load system snapshots in milliseconds using memory-mapped files.

[**26.9x speedup** vs `WinDirStat`](#benchmarks)

[**3.6x to 7.5x speedup** vs `QDirStat`](#benchmarks)

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

## Benchmarks

### Vs `WinDirStat`

### Vs `QDirStat`

To evaluate traversal performance, `edirstat` includes a custom comparison benchmark target comparing it against `qdirstat-cache-writer` (the official headless command-line crawler shipped with `QDirStat` for background scanning).

#### How It Works & Why It's Fair

1. **End-to-End Subprocess Spawning:** Both tools are launched as independent external subprocesses (running the optimized release binary for `edirstat` and the `perl` script execution for `QDirStat`). This captures full end-to-end CLI execution time, including binary loading, runtime initialization (Perl interpreter boot vs. Rust startup), option parsing, and traversal startup.
2. **Warm Cache Inode Priming:** The benchmark performs 2 warm-up runs for each target directory to prime the OS directory entry page caches. This eliminates disk I/O bottlenecks and isolates CPU/algorithm execution efficiency (multi-threaded, work-stealing Rust vs. single-threaded Perl).
3. **Statistical Averaging:** Measurements are collected across 5 consecutive sample runs to compute a robust median and average traversal duration.

#### Results

Across a diverse suite of storage devices and directory layouts, **eDirStat** consistently exceeds the performance of `QDirStat` in duplicate, delivering a consistent **3.6x to 7.5x speedup** in scan time!

Whether crawling highly nested code repositories on high-speed `NVMe` drives, game installations on SATA SSDs, or deep directory trees on enterprise HDDs, `edirstat`'s parallel, work-stealing multithreading model, allows it to remain the fastest disk usage analyzer available.

Key Highlights:

- Up to **7.54x faster** than the `QDirStat` backend writer, especially on SSDs.
- Achieves a **3.60x speedup** on mechanical HDDs even when processing massive, deeply nested directory paths.
- Smoothly scales directory traversal workload across all available CPU threads.

#### System Details

AMD Ryzen 9 9950X3D (32)

Linux 6.18.34-1-cachyos-lts

#### `/home/tux/Documents/git`

`Samsung 990 Pro NVMe SSD (Gen 4) [btrfs]`

Dense, an enormous amount of small files and directories.

```text
Running benches/compare.rs (target/release/deps/compare-f758f81e7b6c6d90)
==================================================
          eDirStat vs QDirStat Benchmark
==================================================
Target Directory : /home/tux/Documents/git
CPU Cores Available: 32
==================================================
Performing 2 warm-up runs...
Performing 5 sample runs...
Run 1/5... edirstat: 1.23s, qdirstat: 6.18s
Run 2/5... edirstat: 1.23s, qdirstat: 6.19s
Run 3/5... edirstat: 1.23s, qdirstat: 6.17s
Run 4/5... edirstat: 1.24s, qdirstat: 6.17s
Run 5/5... edirstat: 1.23s, qdirstat: 6.16s

================ RESULTS SUMMARY ================
eDirStat (Rust, parallel):
  Min   : 1.23s
  Max   : 1.24s
  Median: 1.23s
  Mean  : 1.23s
QDirStat (Perl writer):
  Min   : 6.16s
  Max   : 6.19s
  Median: 6.17s
  Mean  : 6.17s
Speedup (QDirStat / eDirStat): 5.00x
==================================================
```

#### `/run/media/tux/F1/Games/PC/SteamLibrary/steamapps/common`

`Samsung SSD 870 QVO 8TB [btrfs]`

Game files, a mix of large and small files on a SATA SSD.

```text
Running benches/compare.rs (target/release/deps/compare-f758f81e7b6c6d90)
==================================================
eDirStat vs QDirStat Benchmark
==================================================
Target Directory : /run/media/tux/F1/Games/PC/SteamLibrary/steamapps/common
CPU Cores Available: 32
==================================================
Performing 2 warm-up runs...
Performing 5 sample runs...
Run 1/5... edirstat: 570.87ms, qdirstat: 4.02s
Run 2/5... edirstat: 562.79ms, qdirstat: 4.03s
Run 3/5... edirstat: 566.17ms, qdirstat: 4.06s
Run 4/5... edirstat: 557.54ms, qdirstat: 4.07s
Run 5/5... edirstat: 563.63ms, qdirstat: 4.06s

================ RESULTS SUMMARY ================
eDirStat (Rust, parallel):
Min : 557.54ms
Max : 570.87ms
Median: 563.63ms
Mean : 564.20ms
QDirStat (Perl writer):
Min : 4.02s
Max : 4.07s
Median: 4.06s
Mean : 4.05s
Speedup (QDirStat / eDirStat): 7.20x
==================================================
```

#### `/run/media/tux/D1`

`Seagate Exos X18 18TB HDD [btrfs]`

Large files, but fewer, on an HDD. Less directory nesting.

```text
Running benches/compare.rs (target/release/deps/compare-f758f81e7b6c6d90)
==================================================
eDirStat vs QDirStat Benchmark
==================================================
Target Directory : /run/media/tux/D1
CPU Cores Available: 32
==================================================
Performing 2 warm-up runs...
Performing 5 sample runs...
Run 1/5... edirstat: 4.36ms, qdirstat: 30.76ms
Run 2/5... edirstat: 4.15ms, qdirstat: 31.57ms
Run 3/5... edirstat: 4.17ms, qdirstat: 31.31ms
Run 4/5... edirstat: 3.83ms, qdirstat: 30.92ms
Run 5/5... edirstat: 4.03ms, qdirstat: 31.40ms

================ RESULTS SUMMARY ================
eDirStat (Rust, parallel):
Min : 3.83ms
Max : 4.36ms
Median: 4.15ms
Mean : 4.11ms
QDirStat (Perl writer):
Min : 30.76ms
Max : 31.57ms
Median: 31.31ms
Mean : 31.19ms
Speedup (QDirStat / eDirStat): 7.54x
==================================================
```

#### `/run/media/tux/B4`

`Toshiba MG09SACA16EA 16TB HDD [btrfs]`

An enormous amount of tiny files with deep directory nesting, on an HDD.

```text
Running benches/compare.rs (target/release/deps/compare-f758f81e7b6c6d90)
==================================================
eDirStat vs QDirStat Benchmark
==================================================
Target Directory : /run/media/tux/B4
CPU Cores Available: 32
==================================================
Performing 2 warm-up runs...
Performing 5 sample runs...
Run 1/5... edirstat: 976.23ms, qdirstat: 3.40s
Run 2/5... edirstat: 953.45ms, qdirstat: 3.42s
Run 3/5... edirstat: 938.84ms, qdirstat: 3.38s
Run 4/5... edirstat: 945.62ms, qdirstat: 3.41s
Run 5/5... edirstat: 931.06ms, qdirstat: 3.46s

================ RESULTS SUMMARY ================
eDirStat (Rust, parallel):
Min : 931.06ms
Max : 976.23ms
Median: 945.62ms
Mean : 949.04ms
QDirStat (Perl writer):
Min : 3.38s
Max : 3.46s
Median: 3.41s
Mean : 3.41s
Speedup (QDirStat / eDirStat): 3.60x
==================================================
```

---

## 📝 License

This project is licensed under the [MIT License](https://spdx.org/licenses/MIT).
