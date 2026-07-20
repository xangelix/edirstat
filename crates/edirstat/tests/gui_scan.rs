// -- Clippy Denies --
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, atomic::Ordering};

use edirstat::{
    coordinator::SharedState, engine::scanner::EngineScanController, gui::GuiApp,
    traversal::TraversalEngine,
};

/// Test a `GuiApp` given an initial directory path drives the native `ScanController`
/// (traversal engine + coordinator) and receives the published snapshot.
#[test]
fn test_gui_app_scans_initial_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::current_dir()?
        .join("target")
        .join("test_gui_app_initial_path");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    let test_file = temp_dir.join("test.txt");
    std::fs::write(&test_file, b"hello world")?;

    let shared_state = Arc::new(SharedState::new());
    let engine = Arc::new(TraversalEngine::new(shared_state.scan_stats.clone()));
    let scanner = Arc::new(EngineScanController::new(engine, shared_state.clone()));

    // Test scanning a directory
    let mut app = GuiApp::new(
        shared_state.clone(),
        Some(scanner),
        Some(temp_dir.clone()),
        false,
    );
    app.process_pending_initial_path();

    // Wait for the background scan to start
    let mut attempts = 0;
    while !shared_state.is_scanning.load(Ordering::SeqCst) && attempts < 200 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        attempts += 1;
    }

    // Wait for the background scan to complete
    attempts = 0;
    while shared_state.is_scanning.load(Ordering::SeqCst) && attempts < 200 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        attempts += 1;
    }

    let snapshot = shared_state.current_snapshot.load();
    assert!(!snapshot.nodes.is_empty());
    assert_eq!(app.current_scan_path(), Some(temp_dir.as_path()));

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}
