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
    env,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine path to scan
    let scan_path = env::var("BENCH_DIR").map_or_else(
        |_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    let scan_path = std::fs::canonicalize(&scan_path)?;

    println!("==================================================");
    println!("          eDirStat vs QDirStat Benchmark          ");
    println!("==================================================");
    println!("Target Directory : {}", scan_path.display());
    println!(
        "CPU Cores Available: {}",
        std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
    );
    println!("==================================================");

    if !scan_path.exists() {
        eprintln!("Error: Path '{}' does not exist.", scan_path.display());
        std::process::exit(1);
    }
    if !scan_path.is_dir() {
        eprintln!("Error: Path '{}' is not a directory.", scan_path.display());
        std::process::exit(1);
    }

    // Check if qdirstat-cache-writer is available
    let has_qdirstat = Command::new("which")
        .arg("qdirstat-cache-writer")
        .output()
        .is_ok_and(|o| o.status.success());

    if !has_qdirstat {
        println!("WARNING: 'qdirstat-cache-writer' not found in PATH.");
        println!("QDirStat benchmark will be skipped.");
    }

    // Benchmark configuration
    let warmup_runs = 2;
    let sample_runs = 5;

    println!("Performing {warmup_runs} warm-up runs...");
    for _ in 1..=warmup_runs {
        // Run edirstat in-process
        let _ = run_edirstat_scan(&scan_path)?;
        if has_qdirstat {
            let _ = run_qdirstat_scan(&scan_path);
        }
    }

    println!("Performing {sample_runs} sample runs...");
    let mut edirstat_times = Vec::new();
    let mut qdirstat_times = Vec::new();

    for i in 1..=sample_runs {
        print!("Run {i}/{sample_runs}... ");
        std::io::stdout().flush()?;

        let edirstat_dur = run_edirstat_scan(&scan_path)?;
        edirstat_times.push(edirstat_dur);

        if has_qdirstat {
            let qdirstat_dur = run_qdirstat_scan(&scan_path);
            qdirstat_times.push(qdirstat_dur);
            println!("edirstat: {edirstat_dur:.2?}, qdirstat: {qdirstat_dur:.2?}");
        } else {
            println!("edirstat: {edirstat_dur:.2?}");
        }
    }

    println!("\n================ RESULTS SUMMARY ================");
    print_results("eDirStat (Rust, parallel)", &edirstat_times);
    if has_qdirstat {
        print_results("QDirStat (Perl writer)", &qdirstat_times);

        // Compare medians
        let edirstat_median = median(&edirstat_times);
        let qdirstat_median = median(&qdirstat_times);
        let speedup = qdirstat_median.as_secs_f64() / edirstat_median.as_secs_f64();
        println!("Speedup (QDirStat / eDirStat): {speedup:.2}x");
    }
    println!("==================================================");
    println!("Tip: Set BENCH_DIR environment variable to benchmark a custom directory.");
    println!("Example: BENCH_DIR=/var/log cargo bench --bench compare");
    Ok(())
}

fn run_edirstat_scan(path: &PathBuf) -> Result<Duration, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_edirstat"))
        .arg(path)
        .arg("--benchmark")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "edirstat returned error status: {:?}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(start.elapsed())
}

fn run_qdirstat_scan(path: &PathBuf) -> Duration {
    let start = Instant::now();
    let output = Command::new("qdirstat-cache-writer")
        .arg(path)
        .arg("/dev/null")
        .output();
    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!(
                    "Warning: qdirstat-cache-writer returned error status: {:?}",
                    out.status
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: failed to execute qdirstat-cache-writer: {e}");
        }
    }
    start.elapsed()
}

fn median(times: &[Duration]) -> Duration {
    let mut sorted = times.to_vec();
    sorted.sort();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2
    } else {
        sorted[mid]
    }
}

fn mean(times: &[Duration]) -> Duration {
    let sum: Duration = times.iter().sum();
    sum / (times.len() as u32)
}

fn print_results(label: &str, times: &[Duration]) {
    if times.is_empty() {
        return;
    }
    let min = times.iter().min().unwrap_or(&Duration::ZERO);
    let max = times.iter().max().unwrap_or(&Duration::ZERO);
    let med = median(times);
    let avg = mean(times);
    println!("{label}:");
    println!("  Min   : {min:.2?}");
    println!("  Max   : {max:.2?}");
    println!("  Median: {med:.2?}");
    println!("  Mean  : {avg:.2?}");
}
