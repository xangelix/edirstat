#![windows_subsystem = "windows"]
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

use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use edirstat::{coordinator::SharedState, gui::GuiApp, traversal::TraversalEngine};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan or snapshot file to load
    path: Option<PathBuf>,

    /// Run in headless benchmark mode to measure scan time on the target directory and exit
    #[arg(long)]
    benchmark: bool,
}

fn run_benchmark(path_opt: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let path = path_opt.ok_or("Error: A path must be provided for benchmarking.")?;
    if !path.exists() {
        return Err(format!("Error: Path does not exist: {}", path.display()).into());
    }
    let path = std::fs::canonicalize(&path)?;
    if !path.is_dir() {
        return Err(format!("Error: Path is not a directory: {}", path.display()).into());
    }

    println!("Running edirstat benchmark on: {}", path.display());

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

fn main() -> eframe::Result {
    let args = Args::parse();

    if args.benchmark {
        if let Err(e) = run_benchmark(args.path) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        std::process::exit(0);
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
    )
}
