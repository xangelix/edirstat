fn main() -> std::io::Result<()> {
    // Pack assets (licenses subdirectory only).
    include_packed::Config::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/licenses"
    ))
    .level(15)
    .build()
    .map_err(std::io::Error::other)?;

    fluent_zero_build::generate_static_cache(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/locales"
    ));

    Ok(())
}
