#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not parse JavaScript: {0}")]
    Parse(String),
    #[error("could not resolve: {0}")]
    Unresolved(String),
    #[error(
        "chunk hash mismatch (expected {expected}, got {actual}); the build changed under this release"
    )]
    HashMismatch { expected: String, actual: String },
    #[error("entry span {start}..{end} is outside the chunk ({len} bytes)")]
    BadSpan { start: u32, end: u32, len: usize },
}
