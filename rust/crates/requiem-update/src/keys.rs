use std::str;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::{Error, KIND_ROTATION, Result, Signed};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEntry {
    pub id: String,

    pub public: String,
}

impl KeyEntry {
    pub fn new(id: &str, key: &VerifyingKey) -> Self {
        Self {
            id: id.to_string(),
            public: STANDARD.encode(key.as_bytes()),
        }
    }

    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        let bytes: [u8; 32] = STANDARD
            .decode(&self.public)
            .ok()
            .and_then(|b| b.try_into().ok())
            .ok_or_else(|| {
                Error::Malformed(format!("key `{}` is not 32 bytes of base64", self.id))
            })?;
        VerifyingKey::from_bytes(&bytes)
            .map_err(|e| Error::Malformed(format!("key `{}`: {e}", self.id)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeysDocument {
    pub schema: u32,
    pub keys: Vec<KeyEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationStatement {
    pub new_key: KeyEntry,
    #[serde(default)]
    pub retire: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct KeySet {
    keys: Vec<(String, VerifyingKey)>,
}

const OFFICIAL: &[(&str, &str)] = &[
    (
        "primary-2026",
        "CPjunbp7mkjcC6hZE4bdyiWSFdWnBLdvnGIhtj5Kj7k=",
    ),
    ("spare-2026", "mWZJj10TuR5f+k86C+A5yplrBL/i4OBA4ddVycZJLNU="),
];

pub fn official_keys() -> KeySet {
    KeySet::from_entries(OFFICIAL.iter().map(|(id, public)| KeyEntry {
        id: id.to_string(),
        public: public.to_string(),
    }))
    .expect("the compiled-in keys are valid")
}

impl KeySet {
    pub fn from_entries(entries: impl IntoIterator<Item = KeyEntry>) -> Result<Self> {
        let mut set = Self::default();
        for entry in entries {
            set.insert(&entry)?;
        }
        Ok(set)
    }

    pub fn insert(&mut self, entry: &KeyEntry) -> Result<()> {
        let key = entry.verifying_key()?;
        self.keys.retain(|(id, _)| *id != entry.id);
        self.keys.push((entry.id.clone(), key));
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&VerifyingKey> {
        self.keys.iter().find(|(k, _)| k == id).map(|(_, key)| key)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.keys.iter().map(|(id, _)| id.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn apply_rotation(&mut self, statement: &Signed) -> Result<()> {
        let rotation: RotationStatement = statement.open(self, KIND_ROTATION)?;
        rotation.new_key.verifying_key()?;
        self.keys.retain(|(id, _)| !rotation.retire.contains(id));
        self.insert(&rotation.new_key)
    }
}

pub fn fingerprint(key: &VerifyingKey) -> String {
    let hex = crate::sha256_hex(key.as_bytes());
    hex.as_bytes()
        .chunks(4)
        .map(|c| str::from_utf8(c).expect("hex is ascii"))
        .collect::<Vec<_>>()
        .join(" ")
}
