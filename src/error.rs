use thiserror::Error;

#[derive(Error, Debug)]
pub enum EdirstatError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File too small to contain header")]
    HeaderTooSmall,

    #[error("Invalid magic bytes in snapshot header")]
    InvalidMagic,

    #[error("Unsupported snapshot version: {0}")]
    UnsupportedVersion(u16),

    #[error("Truncated snapshot file; nodes missing")]
    TruncatedNodes,

    #[error("Truncated snapshot file; string pool missing")]
    TruncatedStringPool,
}
