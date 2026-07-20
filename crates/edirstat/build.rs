fn main() -> std::io::Result<()> {
    // Check if the target platform we are building FOR is Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();

        // Path to your icon file relative to the project root
        res.set_icon("assets/img/icon.ico");

        // This compiles the resource file and tells cargo to link it
        res.compile()?;
    }

    Ok(())
}
