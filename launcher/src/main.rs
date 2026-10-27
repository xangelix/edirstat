//! The `edirstat` launcher: built for the architecture's baseline (plain
//! x86-64, `apple-m1` on Apple Silicon), it loads the first core in this
//! list the machine fully supports, most preferred first. The list must be
//! seaworthy.toml's `targets` (and `macos-targets`), in the same order:
//! `seaworthy verify` refuses a launcher whose list differs. The order is
//! the one cargo-multivers chose (the most enabled features first), so each
//! machine runs the build it ran before.

#![forbid(unsafe_code)]

#[cfg(not(target_os = "macos"))]
seaworthy_launcher::launcher_main!(
    core = "edirstat_app",
    cpus = [
        "znver5",
        "znver4",
        "alderlake",
        "znver3",
        "skylake",
        "x86-64-v4",
        "x86-64-v3",
        "x86-64-v2",
    ],
);

#[cfg(target_os = "macos")]
seaworthy_launcher::launcher_main!(core = "edirstat_app", cpus = ["apple-m1"]);
