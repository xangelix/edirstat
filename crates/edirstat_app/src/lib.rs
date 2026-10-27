//! eDirStat's per-CPU-level core: the whole application ([`edirstat::app`]),
//! built once per CPU target as a shared library. The `edirstat` launcher
//! (`launcher/`) reads each core's requirement record, loads the best one
//! this machine runs, and calls it in its own process, so arguments, the
//! console and the GUI behave as in the plain `edirstat` binary.

#![forbid(unsafe_code)]

#[global_allocator]
static GLOBAL: mimalloc_rspack::MiMalloc = mimalloc_rspack::MiMalloc;

fn main() -> anyhow::Result<()> {
    edirstat::app::main()
}

seaworthy_launcher::core_main!(main);
