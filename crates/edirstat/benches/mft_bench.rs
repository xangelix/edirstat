use std::{path::Path, sync::Arc, sync::atomic::AtomicUsize, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use edirstat::{
    engine::mft::try_scan_mft,
    engine::traversal::{ScanEvent, TraversalStats},
};

fn bench_mft_parsing(c: &mut Criterion) {
    // Check both potential locations: a literal file at '/workspace' or inside '/workspace/$MFT'
    let path = Path::new("workspace/$MFT");

    let mft_path = if path.is_file() {
        path
    } else {
        eprintln!("WARNING: MFT file not found at 'workspace/$MFT'. Skipping benchmark.");
        return;
    };

    let mut group = c.benchmark_group("MFT Parser Performance");

    // MFT scans are quite heavy. We reduce the sample size and increase
    // measurement time to allow stable, representative iterations.
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(150));

    group.bench_function("parse_raw_mft", |b| {
        b.iter(|| {
            // Set up a channel to receive the parsed events
            let (tx, rx) = crossbeam::channel::unbounded::<Vec<ScanEvent>>();

            // Spawn a background thread to drain the channel instantly
            // to avoid backing up memory or blocking worker channels.
            let drain_handle = std::thread::spawn(move || while rx.recv().is_ok() {});

            let stats = TraversalStats {
                files_scanned: Arc::new(AtomicUsize::new(0)),
                dirs_scanned: Arc::new(AtomicUsize::new(0)),
                bytes_scanned: Arc::new(AtomicUsize::new(0)),
            };

            // Run the MFT parser
            let result = try_scan_mft(mft_path, &tx, &stats);
            assert!(result.is_ok(), "MFT parsing failed: {:?}", result.err());

            // Signal the background drain thread to stop by dropping the sender
            drop(tx);
            let _ = drain_handle.join();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mft_parsing);
criterion_main!(benches);
