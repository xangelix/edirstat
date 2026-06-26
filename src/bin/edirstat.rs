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

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use edirstat::{coordinator::SharedState, gui::GuiApp, traversal::TraversalEngine};

#[global_allocator]
static GLOBAL: mimalloc_rspack::MiMalloc = mimalloc_rspack::MiMalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan or snapshot file to load
    path: Option<PathBuf>,

    /// Run in headless benchmark mode to measure scan time on the target directory and exit
    #[arg(long)]
    benchmark: bool,

    /// Destination path or directory to save the scanned snapshot file (no-GUI/headless)
    #[arg(long)]
    to: Option<PathBuf>,

    /// Disable Zstd compression for the output snapshot file (saves as uncompressed .edst)
    #[arg(long)]
    no_compression: bool,
}

fn run_benchmark(path_opt: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let path = path_opt.ok_or("Error: A path must be provided for benchmarking.")?;
    if !path.exists() {
        return Err(format!("Error: Path does not exist: {}", path.display()).into());
    }
    let path = std::fs::canonicalize(&path)?;

    let is_mft = path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("$mft"));

    if !is_mft && !path.is_dir() {
        return Err(format!("Error: Path is not a directory: {}", path.display()).into());
    }

    println!(
        "Running edirstat benchmark on {}: {}",
        if is_mft { "mft" } else { "dir" },
        path.display()
    );

    let shared_state = Arc::new(SharedState::new());
    let traversal_engine = Arc::new(TraversalEngine::new());
    let (tx, rx) = crossbeam::channel::unbounded();

    let start = std::time::Instant::now();
    let handle = traversal_engine.start_traversal(path.clone(), tx)?;

    let mut coordinator = edirstat::coordinator::Coordinator::new(rx, shared_state);
    coordinator.run_coordinator_loop(&path.to_string_lossy());

    let _ = handle.join();
    let duration = start.elapsed();

    let stats = traversal_engine.stats();
    let files = stats
        .files_scanned
        .load(std::sync::atomic::Ordering::SeqCst);
    let dirs = stats.dirs_scanned.load(std::sync::atomic::Ordering::SeqCst);
    let bytes = stats
        .bytes_scanned
        .load(std::sync::atomic::Ordering::SeqCst);

    println!("----------------------------------------");
    println!("Time elapsed: {duration:?}");
    println!("Directories scanned: {dirs}");
    println!("Files scanned: {files}");
    println!("Total bytes: {bytes}");
    println!("----------------------------------------");
    Ok(())
}

fn run_headless_scan_and_save(
    scan_path: &Path,
    mut to_path: PathBuf,
    no_compression: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !scan_path.exists() {
        return Err(format!("Error: Scan path does not exist: {}", scan_path.display()).into());
    }
    let scan_path = std::fs::canonicalize(scan_path)?;

    let is_mft = scan_path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("$mft"));

    if !is_mft && !scan_path.is_dir() {
        return Err(format!(
            "Error: Scan path is not a directory: {}",
            scan_path.display()
        )
        .into());
    }

    let ext = if no_compression { "edst" } else { "edst.zst" };
    if to_path.extension().is_none_or(|s| s != ext) {
        to_path = to_path.with_added_extension(ext);
    }

    println!("Headless scanning started for: {}", scan_path.display());

    let shared_state = Arc::new(SharedState::new());
    let traversal_engine = Arc::new(TraversalEngine::new());
    let (tx, rx) = crossbeam::channel::unbounded();

    let handle = traversal_engine.start_traversal(scan_path.clone(), tx)?;

    let mut coordinator = edirstat::coordinator::Coordinator::new(rx, shared_state.clone());
    coordinator.run_coordinator_loop(&scan_path.to_string_lossy());

    let _ = handle.join();

    let snapshot = shared_state.current_snapshot.load();
    if snapshot.nodes.is_empty() {
        return Err("Error: The completed scan resulted in an empty snapshot.".into());
    }

    let mut dest_path = to_path;
    if dest_path.is_dir() {
        let folder_name = scan_path
            .file_name()
            .map_or_else(|| "root".to_string(), |s| s.to_string_lossy().into_owned());
        dest_path.push(format!("{folder_name}.{ext}"));
    }

    println!("Saving snapshot to: {}", dest_path.display());
    edirstat::persistence::snapshot::save_snapshot(
        &snapshot.nodes,
        &snapshot.string_pool,
        &dest_path,
        !no_compression,
    )?;
    println!("Snapshot saved successfully.");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "profile-tracy")]
    let _client = tracy_client::Client::start();

    if !cli_or_gui::is_launched_from_terminal() {
        cli_or_gui::hide_console_window();
    }

    let args = Args::parse();

    if args.benchmark {
        run_benchmark(args.path).map_err(|e| anyhow::anyhow!("{e}"))?;

        return Ok(());
    }

    if let Some(to_path) = args.to {
        let scan_path = args.path.unwrap_or_else(|| {
            eprintln!("Error: A path to scan must be provided when utilizing the --to option.");
            std::process::exit(1);
        });

        run_headless_scan_and_save(&scan_path, to_path, args.no_compression)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        return Ok(());
    }

    // Create system states
    let shared_state = Arc::new(SharedState::new());
    let traversal_engine = Arc::new(TraversalEngine::new());

    // Native boot options for eframe
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("eDirStat - Cross-Platform Disk Usage Analyzer")
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../../assets/img/icon_512x.png")[..],
                )
                .map_err(|e| {
                    eprintln!("Failed to load icon: {e}");
                    eframe::Error::AppCreation(Box::new(e))
                })?,
            )
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let initial_path = args.path;

    eframe::run_native(
        "edirstat",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(GuiApp::new(
                shared_state,
                traversal_engine,
                initial_path,
            )))
        }),
    )?;

    Ok(())
}
