fn main() -> std::io::Result<()> {
    // Check if the target platform we are building FOR is Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();

        let icon_path = if std::path::Path::new("icon.ico").exists() {
            "icon.ico"
        } else {
            "../assets/img/icon.ico"
        };
        res.set_icon(icon_path);

        // This compiles the resource file and tells cargo to link it
        res.compile()?;
    }

    Ok(())
}
