//! The command line and the GUI start-up: everything the `edirstat` binary
//! does, as a library function ([`main`]).

#![forbid(unsafe_code)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::AtomicBool,
};

use crate::{
    coordinator::SharedState,
    engine::scanner::EngineScanController,
    extensions::{EXT_DOCKER_INVENTORY, EXT_DOCKER_INVENTORY_VERSION, ExtensionStore},
    gui::{GuiApp, docker::DockerCollector},
    traversal::TraversalEngine,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)] // each bool is an independent CLI flag
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

    /// Skip Docker inventory collection
    #[arg(long)]
    no_docker: bool,

    /// Restrict directory traversal to the same filesystem/device boundary
    #[arg(long, short = 'x', alias = "one-file-system")]
    same_filesystem: bool,
}

fn run_benchmark(
    path_opt: Option<PathBuf>,
    same_filesystem: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let traversal_engine = Arc::new(TraversalEngine::new(shared_state.scan_stats.clone()));
    let (tx, rx) = crossbeam::channel::unbounded();

    let start = std::time::Instant::now();
    let handle = traversal_engine.start_traversal(
        path.clone(),
        same_filesystem,
        shared_state.scan_cancel.clone(),
        tx,
    )?;

    let mut coordinator = crate::coordinator::Coordinator::new(rx, shared_state);
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
    no_docker: bool,
    same_filesystem: bool,
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
    let traversal_engine = Arc::new(TraversalEngine::new(shared_state.scan_stats.clone()));
    let (tx, rx) = crossbeam::channel::unbounded();

    let handle = traversal_engine.start_traversal(
        scan_path.clone(),
        same_filesystem,
        shared_state.scan_cancel.clone(),
        tx,
    )?;

    let mut coordinator = crate::coordinator::Coordinator::new(rx, shared_state.clone());
    coordinator.run_coordinator_loop(&scan_path.to_string_lossy());

    let _ = handle.join();

    let snapshot = shared_state.current_snapshot.load();
    if snapshot.nodes.is_empty() {
        return Err("Error: The completed scan resulted in an empty snapshot.".into());
    }

    // Best-effort Docker inventory: attached to the snapshot as the
    // EXT_DOCKER_INVENTORY extension (never fails the save).
    let mut extensions = snapshot.extensions.clone();
    if !no_docker {
        collect_docker_to_extensions(&shared_state.scan_cancel, &mut extensions);
    }

    let mut dest_path = to_path;
    if dest_path.is_dir() {
        let folder_name = scan_path
            .file_name()
            .map_or_else(|| "root".to_string(), |s| s.to_string_lossy().into_owned());
        dest_path.push(format!("{folder_name}.{ext}"));
    }

    println!("Saving snapshot to: {}", dest_path.display());
    crate::snapshot::save_snapshot_ext(
        &snapshot.nodes,
        &snapshot.string_pool,
        &dest_path,
        !no_compression,
        &extensions,
    )?;
    println!("Snapshot saved successfully.");

    Ok(())
}

/// Collect the Docker inventory for the headless `--to` flow and insert it
/// into `extensions`. Skipped silently when no Docker presence is detected;
/// collection errors go to stderr and the plain snapshot is still saved.
fn collect_docker_to_extensions(cancel: &AtomicBool, extensions: &mut ExtensionStore) {
    let collector = crate::engine::docker::NativeDockerCollector::new();
    if !collector.detect_environment().has_docker() {
        return;
    }
    match collector.collect_preferred(cancel) {
        Ok(inventory) => match inventory.to_extension_payload() {
            Ok(payload) => {
                println!("Docker inventory collected ({:?}).", inventory.source);
                extensions.insert(
                    EXT_DOCKER_INVENTORY,
                    EXT_DOCKER_INVENTORY_VERSION,
                    Arc::from(payload.as_slice()),
                );
            }
            Err(e) => eprintln!("Warning: failed to encode the Docker inventory: {e}"),
        },
        Err(e) => eprintln!("Warning: Docker inventory collection failed: {e}"),
    }
}

/// The application's entry point: parses the command line, then runs a
/// headless benchmark or scan, or opens the GUI.
///
/// The `edirstat` binary (`cargo run`, `cargo install edirstat`) and the
/// per-CPU-level cores the release packages ship (`edirstat_app`, loaded by
/// the `edirstat` launcher) both call this, so every build behaves the same.
pub fn main() -> anyhow::Result<()> {
    #[cfg(feature = "profile-tracy")]
    let _client = tracy_client::Client::start();

    if !cli_or_gui::is_launched_from_terminal() {
        cli_or_gui::hide_console_window();
    }

    let args = Args::parse();

    if args.benchmark {
        run_benchmark(args.path, args.same_filesystem).map_err(|e| anyhow::anyhow!("{e}"))?;

        return Ok(());
    }

    if let Some(to_path) = args.to {
        let scan_path = args.path.unwrap_or_else(|| {
            eprintln!("Error: A path to scan must be provided when utilizing the --to option.");
            std::process::exit(1);
        });

        run_headless_scan_and_save(
            &scan_path,
            to_path,
            args.no_compression,
            args.no_docker,
            args.same_filesystem,
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;

        return Ok(());
    }

    // Create system states
    let shared_state = Arc::new(SharedState::new());
    let traversal_engine = Arc::new(TraversalEngine::new(shared_state.scan_stats.clone()));
    let scanner = Arc::new(EngineScanController::new(
        traversal_engine,
        shared_state.clone(),
    ));

    // Native boot options for eframe
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("eDirStat - Cross-Platform Disk Usage Analyzer")
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/img/icon_512x.png")[..],
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
                Some(scanner),
                initial_path,
                args.same_filesystem,
            )))
        }),
    )?;

    Ok(())
}
