use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Error, KeySet, Result};

const DOMAIN: &[u8] = b"requiem/signed/v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signed {
    pub kind: String,
    pub payload: String,

    pub sig: String,
    pub key_id: String,
}

fn message(kind: &str, payload: &str) -> Vec<u8> {
    let mut msg = Vec::with_capacity(DOMAIN.len() + kind.len() + 1 + payload.len());
    msg.extend_from_slice(DOMAIN);
    msg.extend_from_slice(kind.as_bytes());
    msg.push(0);
    msg.extend_from_slice(payload.as_bytes());
    msg
}

pub fn sign_key_from_seed(seed: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(seed)
}

impl Signed {
    pub fn sign<T: Serialize>(
        kind: &str,
        value: &T,
        key_id: &str,
        key: &SigningKey,
    ) -> Result<Self> {
        let payload = serde_json::to_string(value).map_err(|e| Error::Malformed(e.to_string()))?;
        Ok(Self::sign_text(kind, payload, key_id, key))
    }

    pub fn sign_text(kind: &str, payload: String, key_id: &str, key: &SigningKey) -> Self {
        let sig = key.sign(&message(kind, &payload));
        Self {
            kind: kind.to_string(),
            payload,
            sig: STANDARD.encode(sig.to_bytes()),
            key_id: key_id.to_string(),
        }
    }

    pub fn parse(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|e| Error::Malformed(e.to_string()))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("an envelope serializes")
    }

    pub fn verify(&self, keys: &KeySet, kind: &str) -> Result<&str> {
        if self.kind != kind {
            return Err(Error::WrongKind {
                expected: kind.to_string(),
                found: self.kind.clone(),
            });
        }
        let key = keys
            .get(&self.key_id)
            .ok_or_else(|| Error::UnknownKey(self.key_id.clone()))?;
        let bytes: [u8; 64] = STANDARD
            .decode(&self.sig)
            .ok()
            .and_then(|b| b.try_into().ok())
            .ok_or(Error::BadSignature)?;
        key.verify(
            &message(&self.kind, &self.payload),
            &Signature::from_bytes(&bytes),
        )
        .map_err(|_| Error::BadSignature)?;
        Ok(&self.payload)
    }

    pub fn open<T: DeserializeOwned>(&self, keys: &KeySet, kind: &str) -> Result<T> {
        serde_json::from_str(self.verify(keys, kind)?).map_err(|e| Error::Malformed(e.to_string()))
    }
}
