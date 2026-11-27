use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct ExtractRules {
    pub required_file: String,

    pub forbidden_top: Vec<String>,
    pub max_files: usize,

    pub max_total: u64,
}

fn io(e: io::Error) -> Error {
    Error::Io(e.to_string())
}

fn bad(msg: impl Into<String>) -> Error {
    Error::BadArchive(msg.into())
}

fn is_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit())
}

fn safe_components(path: &Path) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for component in path.components() {
        let Component::Normal(part) = component else {
            return None;
        };
        let part = part.to_str()?;
        if part.is_empty()
            || part.contains([':', '\\', '<', '>', '"', '|', '?', '*'])
            || part.ends_with(['.', ' '])
            || is_reserved(part)
        {
            return None;
        }
        out.push(part.to_string());
    }
    Some(out)
}

struct Entry {
    index: usize,
    parts: Vec<String>,
    is_dir: bool,
}

pub fn extract_zip(zip_path: &Path, dest: &Path, rules: &ExtractRules) -> Result<Vec<PathBuf>> {
    if dest.exists() {
        return Err(bad(format!("{} exists already", dest.display())));
    }
    let mut archive =
        zip::ZipArchive::new(File::open(zip_path).map_err(io)?).map_err(|e| bad(e.to_string()))?;
    if archive.len() > rules.max_files {
        return Err(bad(format!(
            "{} entries (limit {})",
            archive.len(),
            rules.max_files
        )));
    }

    let mut entries = Vec::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|e| bad(e.to_string()))?;
        if entry.is_symlink() {
            return Err(bad(format!("`{}` is a symlink", entry.name())));
        }
        let name = entry
            .enclosed_name()
            .ok_or_else(|| bad(format!("`{}` escapes the archive", entry.name())))?;
        let Some(parts) = safe_components(&name) else {
            return Err(bad(format!("`{}` is not a safe file name", entry.name())));
        };
        total = total.saturating_add(entry.size());
        if total > rules.max_total {
            return Err(bad(format!(
                "more than {} bytes when unpacked",
                rules.max_total
            )));
        }
        entries.push(Entry {
            index,
            parts,
            is_dir: entry.is_dir(),
        });
    }

    let strip = match entries.first().and_then(|e| e.parts.first()) {
        Some(first)
            if entries
                .iter()
                .all(|e| e.parts.first() == Some(first) && (e.parts.len() > 1 || e.is_dir)) =>
        {
            1
        }
        _ => 0,
    };

    let partial = dest.with_extension("partial");
    let _ = fs::remove_dir_all(&partial);
    fs::create_dir_all(&partial).map_err(io)?;
    let result = (|| {
        let mut files = Vec::new();
        for entry in &entries {
            let rel = &entry.parts[strip.min(entry.parts.len())..];
            if rel.is_empty() {
                continue;
            }
            if rules
                .forbidden_top
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&rel[0]))
            {
                return Err(bad(format!("`{}` writes into `{}`", rel.join("/"), rel[0])));
            }
            let target = rel.iter().fold(partial.clone(), |p, part| p.join(part));
            if entry.is_dir {
                fs::create_dir_all(&target).map_err(io)?;
                continue;
            }
            if let Some(dir) = target.parent() {
                fs::create_dir_all(dir).map_err(io)?;
            }
            let mut file = archive
                .by_index(entry.index)
                .map_err(|e| bad(e.to_string()))?;
            let mut out = File::create(&target).map_err(io)?;

            let declared = file.size();
            let copied = io::copy(&mut (&mut file).take(declared + 1), &mut out).map_err(io)?;
            if copied != declared {
                return Err(bad(format!(
                    "`{}` is larger than it declares",
                    rel.join("/")
                )));
            }
            out.flush().map_err(io)?;
            files.push(rel.iter().collect::<PathBuf>());
        }
        let required: PathBuf = rules.required_file.split('/').collect();
        if !files.contains(&required) {
            return Err(bad(format!("`{}` is missing", rules.required_file)));
        }
        Ok(files)
    })();
    match result {
        Ok(files) => {
            fs::rename(&partial, dest).map_err(io)?;
            Ok(files)
        }
        Err(err) => {
            let _ = fs::remove_dir_all(&partial);
            Err(err)
        }
    }
}
