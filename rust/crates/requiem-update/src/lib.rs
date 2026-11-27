use std::result;

mod artifact;
mod envelope;
mod error;
mod keys;
mod manifest;
mod stage;

pub use artifact::{ArtifactCheck, sha256_hex};
pub use envelope::{Signed, sign_key_from_seed};
pub use error::Error;
pub use keys::{KeyEntry, KeySet, KeysDocument, RotationStatement, fingerprint, official_keys};
pub use manifest::{Artifact, Decision, Policy, UpdateManifest};
pub use stage::{ExtractRules, extract_zip};

pub use ed25519_dalek::{SigningKey, VerifyingKey};

pub const KIND_UPDATE: &str = "update-manifest";

pub const KIND_ROTATION: &str = "key-rotation";

pub const KIND_IDENTITY: &str = "desktop-identity";

pub type Result<T> = result::Result<T, Error>;
