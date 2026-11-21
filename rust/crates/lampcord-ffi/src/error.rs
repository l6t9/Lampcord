use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiErrorKind {
    InvalidInput,
    NotConnected,
    Unauthorized,
    Transport,
    Protocol,
    Storage,
    Internal,
    LockPoisoned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfiError {
    pub kind: FfiErrorKind,
    pub message: String,
}

pub type FfiResult<T> = Result<T, FfiError>;

impl FfiError {
    pub fn new(kind: FfiErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(FfiErrorKind::InvalidInput, message)
    }

    pub fn lock_poisoned() -> Self {
        Self::new(FfiErrorKind::LockPoisoned, "internal state lock poisoned")
    }
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for FfiError {}
