use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
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

/// UniFFI-friendly flat error for exported API methods.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "uniffi", uniffi(flat_error))]
pub enum FfiClientError {
    #[error("{kind:?}: {message}")]
    Generic { kind: FfiErrorKind, message: String },
}

impl FfiClientError {
    pub fn new(kind: FfiErrorKind, message: impl Into<String>) -> Self {
        Self::Generic {
            kind,
            message: message.into(),
        }
    }

    pub fn not_connected() -> Self {
        Self::new(FfiErrorKind::NotConnected, "client worker is not running")
    }
}

impl From<FfiError> for FfiClientError {
    fn from(value: FfiError) -> Self {
        Self::Generic {
            kind: value.kind,
            message: value.message,
        }
    }
}
