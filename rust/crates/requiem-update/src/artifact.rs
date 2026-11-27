use sha2::{Digest, Sha256};

use crate::{Artifact, Error, Result};

pub struct ArtifactCheck {
    expected_size: u64,
    expected_sha256: String,
    hasher: Sha256,
    seen: u64,
}

impl ArtifactCheck {
    pub fn new(artifact: &Artifact) -> Self {
        Self {
            expected_size: artifact.size,
            expected_sha256: artifact.sha256.clone(),
            hasher: Sha256::new(),
            seen: 0,
        }
    }

    pub fn update(&mut self, chunk: &[u8]) -> Result<()> {
        self.seen += chunk.len() as u64;
        if self.seen > self.expected_size {
            return Err(Error::TooLarge {
                expected: self.expected_size,
            });
        }
        self.hasher.update(chunk);
        Ok(())
    }

    pub fn seen(&self) -> u64 {
        self.seen
    }

    pub fn finish(self) -> Result<()> {
        if self.seen != self.expected_size {
            return Err(Error::WrongSize {
                expected: self.expected_size,
                found: self.seen,
            });
        }
        // The hasher already holds the digest; hex-encode it rather than hashing it again.
        let digest: String = self
            .hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if digest != self.expected_sha256 {
            return Err(Error::HashMismatch);
        }
        Ok(())
    }
}

pub use requiem_assets::sha256_hex;
