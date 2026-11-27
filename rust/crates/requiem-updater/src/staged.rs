use std::path::{Path, PathBuf};

use semver::Version;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::env;
use std::fs;
use std::io;
#[cfg(test)]
use std::process;

const MARKER: &str = "ready.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staged {
    pub version: String,

    pub dir: PathBuf,
    pub notes_url: Option<String>,

    pub source: String,
}

impl Staged {
    pub fn version(&self) -> Option<Version> {
        Version::parse(&self.version).ok()
    }
}

pub fn save(root: &Path, staged: &Staged) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let tmp = root.join(format!("{MARKER}.tmp"));
    fs::write(
        &tmp,
        serde_json::to_vec_pretty(staged).expect("staged serializes"),
    )?;
    fs::rename(tmp, root.join(MARKER))
}

pub fn load(root: &Path, current: &Version) -> Option<Staged> {
    let staged: Option<Staged> = fs::read(root.join(MARKER))
        .ok()
        .and_then(|raw| serde_json::from_slice(&raw).ok());
    let usable = staged.filter(|s| s.dir.is_dir() && s.version().is_some_and(|v| v > *current));
    match &usable {
        Some(staged) => clean(root, Some(&staged.version)),
        None => discard(root),
    }
    usable
}

pub fn clean(root: &Path, keep: Option<&str>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_dir() {
            if keep != Some(name.as_str()) {
                let _ = fs::remove_dir_all(&path);
            }
        } else if name.ends_with(".zip") {
            let _ = fs::remove_file(&path);
        }
    }
}

pub fn discard(root: &Path) {
    let _ = fs::remove_file(root.join(MARKER));
    clean(root, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf {
        let d = env::temp_dir().join(format!("staged-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn cleaning_leaves_the_update_helpers_records_alone() {
        let root = root("records");
        for file in [
            "helper.log",
            "pending.json",
            "healthy",
            "rolled-back.json",
            "2.0.0.zip.part",
            "1.0.0.zip",
        ] {
            fs::write(root.join(file), b"x").unwrap();
        }
        fs::create_dir_all(root.join("1.0.0/sub")).unwrap();
        fs::create_dir_all(root.join("2.0.0")).unwrap();

        clean(&root, Some("2.0.0"));

        for kept in [
            "helper.log",
            "pending.json",
            "healthy",
            "rolled-back.json",
            "2.0.0.zip.part",
            "2.0.0",
        ] {
            assert!(root.join(kept).exists(), "{kept} must stay");
        }
        for gone in ["1.0.0", "1.0.0.zip"] {
            assert!(!root.join(gone).exists(), "{gone} must go");
        }
    }

    #[test]
    fn a_staged_release_that_is_not_newer_is_dropped_with_its_marker() {
        let root = root("stale");
        let dir = root.join("1.0.0");
        fs::create_dir_all(&dir).unwrap();
        save(
            &root,
            &Staged {
                version: "1.0.0".into(),
                dir: dir.clone(),
                notes_url: None,
                source: "s".into(),
            },
        )
        .unwrap();
        fs::write(root.join("helper.log"), b"log").unwrap();

        assert!(load(&root, &Version::new(1, 0, 0)).is_none());
        assert!(!dir.exists() && !root.join("ready.json").exists());
        assert!(root.join("helper.log").exists());

        let newer = root.join("1.1.0");
        fs::create_dir_all(&newer).unwrap();
        save(
            &root,
            &Staged {
                version: "1.1.0".into(),
                dir: newer,
                notes_url: None,
                source: "s".into(),
            },
        )
        .unwrap();
        assert_eq!(
            load(&root, &Version::new(1, 0, 0)).unwrap().version,
            "1.1.0"
        );
    }
}
