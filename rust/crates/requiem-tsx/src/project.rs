use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::diag::Diagnostic;
use crate::lower::{Module, Options, lower_file, scan_file};

#[derive(Clone, Debug)]
pub struct Source {
    pub path: String,
    pub text: String,
}

impl Source {
    pub fn new(path: &str, text: &str) -> Self {
        Self {
            path: path.to_string(),
            text: text.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileDiagnostic {
    pub path: String,
    pub diagnostic: Diagnostic,
}

#[derive(Clone, Debug)]
pub struct Exported {
    pub key: Rc<str>,
    pub props: Vec<String>,
}

#[derive(Default, Debug)]
pub struct FileIndex {
    pub exports: HashMap<String, Exported>,
    pub types: HashSet<String>,
}

#[derive(Default, Debug)]
pub struct ProjectIndex {
    pub files: HashMap<String, FileIndex>,
}

pub fn normalize(path: &str) -> String {
    let path = path.replace('\\', "/");
    let path = path.strip_prefix("./").unwrap_or(&path);
    for extension in [".tsx", ".ts"] {
        if let Some(stripped) = path.strip_suffix(extension) {
            return stripped.to_string();
        }
    }
    path.to_string()
}

pub fn resolve(from: &str, specifier: &str) -> String {
    let mut parts: Vec<String> = from.split('/').map(str::to_string).collect();
    parts.pop();
    let target = normalize(specifier);
    for segment in target.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_string()),
        }
    }
    parts.join("/")
}

pub fn compile_project(
    sources: &[Source],
    options: &Options,
) -> Result<Module, Vec<FileDiagnostic>> {
    let mut diagnostics = Vec::new();
    let mut index = ProjectIndex::default();
    for source in sources {
        let path = normalize(&source.path);
        match scan_file(&source.text, &path) {
            Ok(file) => {
                index.files.insert(path, file);
            }
            Err(errors) => diagnostics.extend(errors.into_iter().map(|diagnostic| FileDiagnostic {
                path: source.path.clone(),
                diagnostic,
            })),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut components = Vec::new();
    for source in sources {
        let path = normalize(&source.path);
        match lower_file(&source.text, &path, &index, options) {
            Ok(found) => components.extend(found),
            Err(errors) => diagnostics.extend(errors.into_iter().map(|diagnostic| FileDiagnostic {
                path: source.path.clone(),
                diagnostic,
            })),
        }
    }
    if diagnostics.is_empty() {
        Ok(Module {
            components: Rc::new(components),
        })
    } else {
        Err(diagnostics)
    }
}
