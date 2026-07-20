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
// Docs
#![doc = include_str!("../README.md")]
// --- Feature Gates ---
#![cfg_attr(windows, feature(windows_by_handle))]

pub mod engine;

pub use edirstat_core::{EdirstatError, arena, snapshot, time_utils};
pub use edirstat_gui as gui;
pub use engine::{coordinator, traversal};
pub use gui::colors;

pub mod model {
    pub use edirstat_core::{arena, time_utils, varint};

    pub mod persistence {
        pub use edirstat_core::snapshot;
        pub use edirstat_gui::preferences;
    }
}
