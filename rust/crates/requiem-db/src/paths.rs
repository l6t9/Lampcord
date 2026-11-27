use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::OnceLock;

pub const DATA_DIR_ENV: &str = "REQUIEM_DATA_DIR";

struct Root {
    dir: Option<PathBuf>,

    portable: bool,
}

fn root() -> &'static Root {
    static ROOT: OnceLock<Root> = OnceLock::new();
    ROOT.get_or_init(resolve)
}

pub fn data_root() -> Option<PathBuf> {
    root().dir.clone()
}

pub fn is_portable() -> bool {
    root().portable
}

pub fn system_cache_root() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|h| h.join(".cache")))
    };
    base.map(|b| b.join(APP_DIR))
}

pub fn cache_root() -> Option<PathBuf> {
    if is_portable() {
        data_root().map(|d| d.join("cache"))
    } else {
        system_cache_root()
    }
}

const APP_DIR: &str = "requiem";

fn resolve() -> Root {
    if let Some(dir) = env::var_os(DATA_DIR_ENV).filter(|v| !v.is_empty()) {
        return Root {
            dir: Some(PathBuf::from(dir)),
            portable: true,
        };
    }
    if cfg!(windows) {
        let beside_exe = env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("data")));
        if let Some(dir) = beside_exe {
            if writable(&dir) {
                return Root {
                    dir: Some(dir),
                    portable: true,
                };
            }
            log::warn!(
                "{} is not writable; keeping data in %APPDATA% instead",
                dir.display()
            );
        }
    }
    Root {
        dir: system_config_root(),
        portable: false,
    }
}

fn system_config_root() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        env::var_os("APPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|h| h.join(".config")))
    };
    base.map(|b| b.join(APP_DIR))
}

fn home() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(".write-test-{}", process::id()));
    let ok = fs::write(&probe, b"").is_ok();
    let _ = fs::remove_file(probe);
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_folder_is_created_and_probed() {
        let dir = env::temp_dir()
            .join(format!("requiem-db-paths-{}", process::id()))
            .join("data");
        assert!(writable(&dir));
        assert!(
            fs::read_dir(&dir).unwrap().next().is_none(),
            "the probe file is removed"
        );
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn a_folder_under_a_file_is_not_writable() {
        let file = env::temp_dir().join(format!("requiem-db-paths-file-{}", process::id()));
        fs::write(&file, b"x").unwrap();
        assert!(!writable(&file.join("data")));
        let _ = fs::remove_file(file);
    }
}
