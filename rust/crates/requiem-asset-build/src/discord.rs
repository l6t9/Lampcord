use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkFile {
    pub name: String,
}

impl ChunkFile {
    pub fn is_js(&self) -> bool {
        self.name.ends_with(".js")
    }
    pub fn is_css(&self) -> bool {
        self.name.ends_with(".css")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_id: String,
    pub build_number: Option<u64>,
    pub channel: String,

    pub entry_script: String,

    pub chunks: Vec<ChunkFile>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("page has no GLOBAL_ENV `{0}`")]
    MissingEnv(&'static str),
    #[error("page links no entry script (web.<hash>.js)")]
    NoEntryScript,
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub build_id: String,
    pub build_number: Option<u64>,
    pub channel: String,
    pub entry_script: String,

    pub linked: Vec<ChunkFile>,
}

pub fn parse_page(html: &str) -> Result<PageInfo, ParseError> {
    let build_id =
        string_after(html, "\"VERSION_HASH\":\"").ok_or(ParseError::MissingEnv("VERSION_HASH"))?;
    let build_number = string_after(html, "\"BUILD_NUMBER\":\"").and_then(|s| s.parse().ok());
    let channel = string_after(html, "\"RELEASE_CHANNEL\":\"").unwrap_or_else(|| "stable".into());
    let linked = asset_names(html, &["js", "css"]);
    let entry_script = linked
        .iter()
        .find(|c| c.name.starts_with("web.") && c.is_js())
        .map(|c| c.name.clone())
        .ok_or(ParseError::NoEntryScript)?;
    Ok(PageInfo {
        build_id,
        build_number,
        channel,
        entry_script,
        linked,
    })
}

pub fn build_info(page: PageInfo, entry_script_source: &str) -> BuildInfo {
    let mut chunks = page.linked;
    chunks.extend(lazy_chunks(entry_script_source));
    let mut seen = HashSet::new();
    chunks.retain(|c| seen.insert(c.name.clone()));
    BuildInfo {
        build_id: page.build_id,
        build_number: page.build_number,
        channel: page.channel,
        entry_script: page.entry_script,
        chunks,
    }
}

pub fn lazy_chunks(entry_script: &str) -> Vec<ChunkFile> {
    let mut out = Vec::new();
    for ext in ["js", "css"] {
        out.extend(explicit_branches(entry_script, ext));
        out.extend(fallback_map(entry_script, ext));
    }
    out
}

fn explicit_branches(source: &str, ext: &str) -> Vec<ChunkFile> {
    let mut out = Vec::new();
    let suffix = format!(".{ext}\"");
    let mut from = 0;
    while let Some(at) = source[from..].find("\"===e?") {
        let at = from + at;
        from = at + 6;

        let digits: String = source[..at]
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if digits.is_empty() {
            continue;
        }
        let rest = &source[from..];
        let literal_end = if let Some(r) = rest.strip_prefix("\"\"+e+\"") {
            r.find('"').map(|n| &r[..n + 1])
        } else if let Some(r) = rest.strip_prefix('"') {
            r.find('"').map(|n| &r[..n + 1])
        } else {
            None
        };
        let Some(lit) = literal_end else { continue };
        if !lit.ends_with(&suffix) {
            continue;
        }
        let body = lit.trim_end_matches('"');

        let name = if body.starts_with('.') {
            format!("{digits}{body}")
        } else {
            body.to_string()
        };
        out.push(ChunkFile { name });
    }
    out
}

fn fallback_map(source: &str, ext: &str) -> Vec<ChunkFile> {
    let mut out = Vec::new();
    let close = format!("}})[e]+\".{ext}\"");
    let mut from = 0;
    while let Some(at) = source[from..].find("\"\"+({") {
        let start = from + at + 4;
        from = start;
        let Some(end) = matching_brace(source, start) else {
            continue;
        };
        if !source[end..].starts_with(&close) {
            continue;
        }
        for entry in source[start + 1..end].split(',') {
            let Some((_, value)) = entry.split_once(':') else {
                continue;
            };
            let hash = value.trim().trim_matches('"');
            if (8..=64).contains(&hash.len()) && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                out.push(ChunkFile {
                    name: format!("{hash}.{ext}"),
                });
            }
        }
    }
    out
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in source.bytes().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn string_after(haystack: &str, marker: &str) -> Option<String> {
    let start = haystack.find(marker)? + marker.len();
    let end = haystack[start..].find('"')?;
    Some(haystack[start..start + end].to_string())
}

fn asset_names(text: &str, exts: &[&str]) -> Vec<ChunkFile> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for part in text.split("/assets/").skip(1) {
        let len = part
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
            .count();
        let name = &part[..len];
        if exts.iter().any(|e| name.ends_with(&format!(".{e}"))) && seen.insert(name.to_string()) {
            out.push(ChunkFile {
                name: name.to_string(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    #[test]
    fn explicit_and_fallback_forms() {
        let src = r#"T.u=e=>"1"===e?""+e+".aaaaaaaaaaaaaaaa.js":"2"===e?"2.bbbbbbbbbbbbbbbb.js":""+({10:"cccccccccccccccc",x:"dddddddddddddddd"})[e]+".js",T.g=1"#;
        let names: Vec<_> = lazy_chunks(src).into_iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            [
                "1.aaaaaaaaaaaaaaaa.js",
                "2.bbbbbbbbbbbbbbbb.js",
                "cccccccccccccccc.js",
                "dddddddddddddddd.js"
            ]
        );
    }

    #[test]
    fn page_env() {
        let html = r#"<script src="/assets/web.ea428454da038edc.js"></script>GLOBAL_ENV = {"BUILD_NUMBER":"626571","RELEASE_CHANNEL":"stable","VERSION_HASH":"222f262f"}"#;
        let page = parse_page(html).unwrap();
        assert_eq!(page.build_number, Some(626571));
        assert_eq!(page.build_id, "222f262f");
        assert_eq!(page.entry_script, "web.ea428454da038edc.js");
    }

    #[test]
    fn real_build_chunk_counts() {
        let Ok(dir) = env::var("ASSET_SERVICE_FIXTURES") else {
            return;
        };
        let html = fs::read_to_string(format!("{dir}/app.html")).unwrap();
        let web = fs::read_to_string(format!("{dir}/web.js")).unwrap();
        let info = build_info(parse_page(&html).unwrap(), &web);
        let js = info.chunks.iter().filter(|c| c.is_js()).count();
        let css = info.chunks.iter().filter(|c| c.is_css()).count();
        eprintln!(
            "build {} #{:?}: {js} js + {css} css",
            info.build_id, info.build_number
        );
        assert!(js > 5000 && css > 1000);
    }
}
