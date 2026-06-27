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

    /// A snapshot integer field was too large for the host's address space or
    /// target integer width (e.g. an offset or count that overflows `usize`).
    #[error("Snapshot value out of range: {0}")]
    OutOfRange(&'static str),

    /// The snapshot's metadata is internally inconsistent — for example
    /// non-monotonic string-pool offsets, or a column shorter than the declared
    /// node count. The file is not merely truncated; its declared sizes
    /// contradict each other.
    #[error("Corrupt snapshot file: {0}")]
    Corrupt(&'static str),

    /// The byte stream could not be decoded — e.g. an overlong or unterminated
    /// varint. Reported separately from [`Self::TruncatedNodes`] so callers can
    /// distinguish "ran out of bytes" from "bytes are malformed".
    #[error("Malformed snapshot data: {0}")]
    Decode(&'static str),

    /// The string pool contained bytes that are not valid UTF-8.
    #[error("Snapshot string pool is not valid UTF-8")]
    InvalidUtf8,

    /// A Zstandard decompression failure on the snapshot container or payload.
    /// Kept separate from generic I/O so a truncated/corrupt *container* is
    /// distinguishable from a failing disk read.
    #[error("Zstd decompression error: {0}")]
    Zstd(String),
}
