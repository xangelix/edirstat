#![forbid(unsafe_code)]
// -- Clippy Denies --
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// --- Clippy Lint Groups & Specific Warnings ---
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::needless_return)]
// --- Allowed Lints (Overrides) ---
#![allow(clippy::mod_module_files)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::panic)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::future_not_send)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::crate_in_macro_def)]
#![allow(clippy::too_many_lines)]

use std::path::PathBuf;

pub mod gui;
pub mod preferences;
pub mod stats;

pub use edirstat_core::{EdirstatError, arena, file_id, snapshot, state, time_utils};
pub use gui::theme as colors;

pub use gui::GuiApp;

// Generated fluent-zero message cache. The generated code uses unwrap/expect
// and unseparated literals by design; the `t!` macro reaches it via `crate::CACHE`.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::unreadable_literal)]
mod static_cache {
    include!(concat!(env!("OUT_DIR"), "/static_cache.rs"));
}

pub use static_cache::*;

/// Gates UI for functionality that needs a live local filesystem or OS integration.
pub(crate) const IS_NATIVE: bool = !cfg!(target_family = "wasm");

/// When true, UI elements that are non-viable or native-only are completely hidden in WASM/web.
/// When false (default), non-viable UI elements remain visible but disabled with hover text.
pub(crate) const HIDE_NA_UI: bool = false;

/// Backend capable of starting directory scans. Implemented by the native
/// crate on top of the traversal engine; the wasm frontend passes `None` and
/// acts as a pure snapshot viewer.
pub trait ScanController: Send + Sync {
    /// Start an asynchronous scan of `path`, publishing progress and the final
    /// snapshot into the shared state given at construction.
    fn start_scan(&self, path: PathBuf, same_filesystem: bool);

    /// Number of worker threads a scan will use (displayed in the UI).
    fn num_threads(&self) -> usize;
}
