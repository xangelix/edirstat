# eDirStat

A fast, cross-platform disk usage analyzer and deduplicator—with work-stealing
multithreading, zero-copy snapshots, and an interactive treemap GUI.

This crate is the native application: the work-stealing traversal engine
(including the Windows NTFS `$MFT` fast path), the CLI, and the desktop GUI
binary. See the [repository](https://github.com/xangelix/edirstat) for the full
documentation, screenshots, and the companion crates:

- `edirstat-core` — data model and zero-copy snapshot format (wasm-compatible)
- `edirstat-gui` — the egui frontend (native and browser/wasm)
