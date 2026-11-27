use std::cmp;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct BuildEnv {
    pub msvc: Option<Msvc>,
    pub sdk: Option<WindowsSdk>,
    pub ffmpeg: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Msvc {
    pub version: String,

    pub root: PathBuf,
}

#[derive(Debug)]
pub struct WindowsSdk {
    pub version: String,
    pub include: PathBuf,
    pub lib: PathBuf,
}

impl BuildEnv {
    pub fn discover() -> Self {
        if !cfg!(windows) {
            return Self::default();
        }
        Self {
            msvc: find_msvc(),
            sdk: find_sdk(),
            ffmpeg: find_ffmpeg(),
        }
    }

    pub fn vars(&self) -> Vec<(&'static str, OsString)> {
        let mut vars = Vec::new();
        let mut path_front: Vec<PathBuf> = Vec::new();

        if let Some(msvc) = &self.msvc {
            path_front.push(msvc.root.join("bin").join("Hostx64").join("x64"));
        }
        if let Some(ffmpeg) = &self.ffmpeg {
            path_front.push(ffmpeg.join("bin"));
            vars.push(("FFMPEG_DIR", ffmpeg.clone().into_os_string()));
        }
        if !path_front.is_empty() {
            let mut all = path_front;
            all.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
            if let Ok(joined) = env::join_paths(all) {
                vars.push(("PATH", joined));
            }
        }

        let mut include: Vec<PathBuf> = Vec::new();
        let mut lib: Vec<PathBuf> = Vec::new();
        if let Some(msvc) = &self.msvc {
            include.push(msvc.root.join("include"));
            lib.push(msvc.root.join("lib").join("x64"));
        }
        if let Some(sdk) = &self.sdk {
            for part in ["ucrt", "um", "shared", "winrt", "cppwinrt"] {
                let dir = sdk.include.join(part);
                if dir.is_dir() {
                    include.push(dir);
                }
            }
            for part in ["ucrt", "um"] {
                let dir = sdk.lib.join(part).join("x64");
                if dir.is_dir() {
                    lib.push(dir);
                }
            }
        }
        if !include.is_empty() {
            if let Ok(joined) = env::join_paths(include) {
                vars.push(("INCLUDE", joined));
            }
        }
        if !lib.is_empty() {
            if let Ok(joined) = env::join_paths(lib) {
                vars.push(("LIB", joined));
            }
        }
        vars
    }
}

fn version_dirs(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut found: Vec<(String, PathBuf)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
        .collect();
    found.sort_by_key(|(name, _)| cmp::Reverse(version_key(name)));
    found
}

fn version_key(name: &str) -> Vec<u32> {
    name.split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

fn find_msvc() -> Option<Msvc> {
    for program_files in ["C:\\Program Files", "C:\\Program Files (x86)"] {
        for year_dir in version_dirs(&Path::new(program_files).join("Microsoft Visual Studio")) {
            for edition in ["Community", "Professional", "Enterprise", "BuildTools"] {
                let tools = year_dir
                    .1
                    .join(edition)
                    .join("VC")
                    .join("Tools")
                    .join("MSVC");
                if let Some((version, root)) = version_dirs(&tools).into_iter().find(|(_, root)| {
                    root.join("bin")
                        .join("Hostx64")
                        .join("x64")
                        .join("link.exe")
                        .is_file()
                }) {
                    return Some(Msvc { version, root });
                }
            }
        }
    }
    None
}

fn find_sdk() -> Option<WindowsSdk> {
    let kits = Path::new("C:\\Program Files (x86)\\Windows Kits\\10");
    let (version, include) = version_dirs(&kits.join("Include"))
        .into_iter()
        .find(|(_, dir)| dir.join("ucrt").is_dir())?;
    let lib = kits.join("Lib").join(&version);
    lib.is_dir().then_some(WindowsSdk {
        version,
        include,
        lib,
    })
}

fn find_ffmpeg() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("FFMPEG_DIR").map(PathBuf::from) {
        if dir.join("bin").is_dir() {
            return Some(dir);
        }
    }
    let packages = PathBuf::from(env::var_os("LOCALAPPDATA")?)
        .join("Microsoft")
        .join("WinGet")
        .join("Packages");
    for package in fs::read_dir(packages).ok()?.flatten() {
        if !package
            .file_name()
            .to_string_lossy()
            .starts_with("Gyan.FFmpeg.Shared")
        {
            continue;
        }
        if let Some((_, dir)) = version_dirs(&package.path())
            .into_iter()
            .find(|(name, dir)| {
                name.starts_with("ffmpeg-")
                    && dir.join("bin").is_dir()
                    && dir.join("include").is_dir()
            })
        {
            return Some(dir);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_sort_numerically_not_lexically() {
        assert!(version_key("14.44.35207") > version_key("14.9.1"));
        assert!(version_key("10.0.26100.0") > version_key("10.0.22621.0"));
    }

    #[test]
    fn an_empty_environment_sets_nothing() {
        assert!(BuildEnv::default().vars().is_empty());
    }
}
