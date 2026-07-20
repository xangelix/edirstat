use std::{path::PathBuf, sync::Arc};

use edirstat_core::state::SharedState;
use edirstat_gui::ScanController;

use super::{coordinator::Coordinator, traversal::TraversalEngine};

/// Native scanner backend: runs directory scans on background threads via the
/// traversal engine and coordinator, publishing snapshots and progress into
/// the shared state for the GUI to render.
pub struct EngineScanController {
    traversal_engine: Arc<TraversalEngine>,
    shared_state: Arc<SharedState>,
}

impl EngineScanController {
    pub const fn new(
        traversal_engine: Arc<TraversalEngine>,
        shared_state: Arc<SharedState>,
    ) -> Self {
        Self {
            traversal_engine,
            shared_state,
        }
    }
}

impl ScanController for EngineScanController {
    fn start_scan(&self, path: PathBuf, same_filesystem: bool) {
        // Start traversal and coordinator
        let (tx, rx) = crossbeam::channel::unbounded();

        // Launch Traversal Engine in background
        match self
            .traversal_engine
            .start_traversal(path.clone(), same_filesystem, tx)
        {
            Ok(_) => {
                // Launch Coordinator in background
                let mut coordinator = Coordinator::new(rx, self.shared_state.clone());
                std::thread::spawn(move || {
                    coordinator.run_coordinator_loop(&path.to_string_lossy());
                });
            }
            Err(e) => {
                println!("Failed to start traversal: {e}");
            }
        }
    }

    fn num_threads(&self) -> usize {
        self.traversal_engine.num_threads()
    }
}
