#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("malformed document: {0}")]
    Malformed(String),
    #[error("signed by key `{0}`, which is not trusted")]
    UnknownKey(String),
    #[error("the signature does not match the document")]
    BadSignature,
    #[error("the document is a `{found}`, not a `{expected}`")]
    WrongKind { expected: String, found: String },
    #[error("unsupported schema version {0}")]
    UnsupportedSchema(u32),
    #[error("the feed is for channel `{found}`, not `{expected}`")]
    WrongChannel { expected: String, found: String },
    #[error("the feed expired at {0} (unix seconds)")]
    Expired(u64),
    #[error("the feed's serial {found} is older than the {seen} already seen")]
    OlderThanSeen { found: u64, seen: u64 },
    #[error("version {0} has been revoked")]
    Revoked(String),
    #[error("no download for target `{0}`")]
    NoArtifact(String),
    #[error("artifact is invalid: {0}")]
    BadArtifact(String),
    #[error("the download is larger than the {expected} bytes the feed promised")]
    TooLarge { expected: u64 },
    #[error("the download has {found} bytes, the feed promised {expected}")]
    WrongSize { expected: u64, found: u64 },
    #[error("the download's SHA-256 does not match the feed")]
    HashMismatch,
    #[error("the archive is not acceptable: {0}")]
    BadArchive(String),
    #[error("i/o error: {0}")]
    Io(String),
}
