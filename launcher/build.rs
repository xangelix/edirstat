//! The Windows launcher carries eDirStat's icon (Explorer, the taskbar and
//! the shortcuts show it). seaworthy replaces the version resource with the
//! release's when it packages the launcher.

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=../assets/img/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../assets/img/icon.ico");
        res.compile()?;
    }
    Ok(())
}
