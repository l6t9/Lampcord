use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub target: String,

    pub sha256: String,
    pub size: u64,

    pub urls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub schema: u32,
    pub channel: String,

    pub serial: u64,
    pub version: String,

    pub released_at: u64,

    pub expires_at: u64,

    #[serde(default)]
    pub revoked: Vec<String>,
    #[serde(default)]
    pub notes_url: Option<String>,

    #[serde(default)]
    pub sources: Vec<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone)]
pub struct Policy<'a> {
    pub current: &'a Version,
    pub channel: &'a str,
    pub target: &'a str,

    pub now: u64,

    pub last_serial: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision<'a> {
    UpToDate,

    Update {
        version: Version,
        artifact: &'a Artifact,
    },
}

impl UpdateManifest {
    pub fn check<'a>(&'a self, policy: &Policy) -> Result<Decision<'a>> {
        if self.schema != SCHEMA {
            return Err(Error::UnsupportedSchema(self.schema));
        }
        if self.channel != policy.channel {
            return Err(Error::WrongChannel {
                expected: policy.channel.to_string(),
                found: self.channel.clone(),
            });
        }
        if self.serial < policy.last_serial {
            return Err(Error::OlderThanSeen {
                found: self.serial,
                seen: policy.last_serial,
            });
        }
        if policy.now >= self.expires_at {
            return Err(Error::Expired(self.expires_at));
        }
        let version = Version::parse(&self.version)
            .map_err(|e| Error::Malformed(format!("version `{}`: {e}", self.version)))?;
        if self.revoked.contains(&self.version) {
            return Err(Error::Revoked(self.version.clone()));
        }
        if version <= *policy.current {
            return Ok(Decision::UpToDate);
        }
        let artifact = self
            .artifacts
            .iter()
            .find(|a| a.target == policy.target)
            .ok_or_else(|| Error::NoArtifact(policy.target.to_string()))?;
        artifact.validate()?;
        Ok(Decision::Update { version, artifact })
    }
}

impl Artifact {
    fn validate(&self) -> Result<()> {
        if self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(Error::BadArtifact(
                "sha256 is not 64 lowercase hex digits".into(),
            ));
        }
        if self.size == 0 {
            return Err(Error::BadArtifact("size is zero".into()));
        }
        if self.urls.is_empty()
            || self
                .urls
                .iter()
                .any(|u| !(u.starts_with("https://") || u.starts_with("http://")))
        {
            return Err(Error::BadArtifact(
                "urls must be a non-empty list of http(s) URLs".into(),
            ));
        }
        Ok(())
    }
}
