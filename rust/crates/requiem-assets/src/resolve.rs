use crate::index::ParamDefault;
use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};

use crate::Error;
use crate::index::{ExportTarget, FunctionInfo, Module, index_chunk};
use crate::manifest::{BuildRef, ChunkRef, ClientInfo, Entry, EntryKind, Manifest, SCHEMA};
use crate::svg;
use std::collections;
use std::collections::BTreeSet;
use std::mem;
use std::str;

#[derive(Debug, Clone)]
pub enum Recipe {
    SvgComponent {
        name: String,
        export: String,
    },

    SvgByPath {
        name: String,
        d_prefix: String,
    },

    SvgByMarkup {
        name: String,
        needles: Vec<String>,
    },

    ContextFile {
        name: String,
        file: String,
    },

    AssetFile {
        name: String,
        file: String,
    },

    DefaultedAsset {
        name: String,
        key: String,
    },

    ContentUrl {
        name: String,
        hash_prefix: String,
    },

    PathLiteral {
        name: String,
        d_prefix: String,
    },

    LottieJson {
        name: String,
        nm: String,
    },

    FontFace {
        name: String,
        family: String,
        weight: u16,
        italic: bool,
    },
}

impl Recipe {
    pub fn name(&self) -> &str {
        match self {
            Recipe::SvgComponent { name, .. }
            | Recipe::SvgByPath { name, .. }
            | Recipe::SvgByMarkup { name, .. }
            | Recipe::ContextFile { name, .. }
            | Recipe::AssetFile { name, .. }
            | Recipe::DefaultedAsset { name, .. }
            | Recipe::PathLiteral { name, .. }
            | Recipe::ContentUrl { name, .. }
            | Recipe::LottieJson { name, .. }
            | Recipe::FontFace { name, .. } => name,
        }
    }
}

struct IndexedChunk {
    url: String,
    sha256: String,
    size: u64,
}

struct FontRule {
    family: String,
    weight: String,
    italic: bool,

    file: String,
}

#[derive(Default, Debug)]
pub struct Inventory {
    pub icons: Vec<String>,

    pub icon_failures: Vec<(String, String)>,

    pub lottie: Vec<String>,

    pub fonts: collections::BTreeMap<String, Vec<String>>,

    pub files: collections::BTreeMap<String, Vec<(Option<String>, String)>>,
}

#[derive(Default)]
pub struct Build {
    chunks: Vec<IndexedChunk>,

    modules: HashMap<String, (usize, Module)>,
    fonts: Vec<FontRule>,

    referenced: HashSet<String>,

    content_urls: HashSet<String>,
}

impl Build {
    pub fn add_chunk(&mut self, url: &str, bytes: &[u8]) -> Result<(), Error> {
        let source = str::from_utf8(bytes).map_err(|e| Error::Parse(e.to_string()))?;
        let index = index_chunk(source)?;
        let chunk = self.register(url, bytes);
        self.referenced.extend(scan_asset_refs(source));
        self.content_urls.extend(scan_content_urls(source));
        for module in index.modules {
            if let Some(file) = &module.asset_file {
                self.referenced.insert(file.clone());
            }
            self.modules.insert(module.id.clone(), (chunk, module));
        }
        Ok(())
    }

    pub fn add_css(&mut self, url: &str, bytes: &[u8]) -> Result<(), Error> {
        let source = str::from_utf8(bytes).map_err(|e| Error::Parse(e.to_string()))?;
        self.register(url, bytes);
        self.referenced.extend(scan_asset_refs(source));
        self.fonts.extend(parse_font_faces(source));
        Ok(())
    }

    fn register(&mut self, url: &str, bytes: &[u8]) -> usize {
        self.chunks.push(IndexedChunk {
            url: url.to_string(),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        });
        self.chunks.len() - 1
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn find_svg(&self, d_prefix: &str) -> Vec<FoundSvg> {
        let wanted = path_tokens(d_prefix);
        let mut found = Vec::new();
        if wanted.is_empty() {
            return found;
        }
        for (_, module) in self.modules.values() {
            for (function, info) in &module.functions {
                let Some(Ok(icon)) = &info.icon else { continue };
                if icon
                    .paths
                    .iter()
                    .any(|p| path_tokens(&p.d).starts_with(&wanted))
                {
                    found.push(FoundSvg {
                        module_id: module.id.clone(),
                        function: function.clone(),
                        icon: icon.clone(),
                    });
                }
            }
        }
        found.sort_by(|a, b| (&a.module_id, &a.function).cmp(&(&b.module_id, &b.function)));
        found
    }

    pub fn export_names(&self) -> impl Iterator<Item = &str> {
        self.modules
            .values()
            .flat_map(|(_, m)| m.exports.keys().map(String::as_str))
    }

    pub fn icon_exports(&self) -> Vec<(String, Result<svg::SvgIcon, String>)> {
        let mut out = Vec::new();
        for (_, module) in self.modules.values() {
            for (export, target) in &module.exports {
                let Some((owner, _, function)) = self.follow(module, target, 0) else {
                    continue;
                };
                if let Some(icon) = &owner.functions[&function].icon {
                    out.push((export.clone(), icon.clone()));
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    pub fn icon_svg(&self, export: &str) -> Result<String, String> {
        let (_, icon) = self
            .icon_exports()
            .into_iter()
            .find(|(n, _)| n == export)
            .ok_or_else(|| format!("no icon component exported as `{export}`"))?;
        icon?.to_svg().map_err(|e| e.to_string())
    }

    pub fn inventory(&self) -> Inventory {
        let mut inv = Inventory::default();
        let mut seen = BTreeSet::new();
        for (name, icon) in self.icon_exports() {
            let Ok(icon) = icon else { continue };
            if !seen.insert(name.clone()) {
                continue;
            }
            match icon.to_svg() {
                Ok(_) => inv.icons.push(name.clone()),
                Err(why) => inv.icon_failures.push((name.clone(), why.to_string())),
            }
        }
        for font in &self.fonts {
            let styles = inv.fonts.entry(font.family.clone()).or_default();
            let style = format!(
                "{}{}",
                font.weight,
                if font.italic { " italic" } else { "" }
            );
            if !styles.contains(&style) {
                styles.push(style);
            }
        }

        let mut context_names: HashMap<&str, &str> = HashMap::new();
        for (_, module) in self.modules.values() {
            for (key, id) in module.context.iter().flatten() {
                context_names.insert(id.as_str(), key.trim_start_matches("./"));
            }
        }
        for (_, module) in self.modules.values() {
            if let Some(file) = &module.asset_file {
                let ext = file.rsplit('.').next().unwrap_or("").to_string();
                let name = context_names.get(module.id.as_str()).map(|n| n.to_string());
                inv.files.entry(ext).or_default().push((name, file.clone()));
            }
            if let Some(json) = &module.json {
                let header = json.head.split("\"layers\"").next().unwrap_or(&json.head);
                if let Some(nm) = header
                    .split("\"nm\":\"")
                    .nth(1)
                    .and_then(|r| r.split('"').next())
                {
                    inv.lottie.push(nm.to_string());
                }
            }
        }
        inv.icons.sort();
        inv.lottie.sort();
        for files in inv.files.values_mut() {
            files.sort();
            files.dedup();
        }
        inv
    }

    fn module(&self, id: &str) -> Option<&(usize, Module)> {
        self.modules.get(id)
    }

    fn follow(
        &self,
        module: &Module,
        target: &ExportTarget,
        depth: u8,
    ) -> Option<(&Module, usize, String)> {
        if depth > 8 {
            return None;
        }
        match target {
            ExportTarget::Local(name) => {
                let (chunk, here) = self.module(&module.id)?;
                here.functions
                    .contains_key(name)
                    .then(|| (here, *chunk, name.clone()))
            }
            ExportTarget::Member { binding, property } => {
                let (_, next) = self.module(module.requires.get(binding)?)?;
                self.follow(next, next.exports.get(property)?, depth + 1)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FoundSvg {
    pub module_id: String,
    pub function: String,
    pub icon: svg::SvgIcon,
}

#[derive(Debug)]
pub struct Resolution {
    pub manifest: Manifest,

    pub failures: Vec<(String, String)>,
}

pub fn resolve(
    build: &Build,
    recipes: &[Recipe],
    source: &BuildRef,
    release_channel: &str,
    asset_base: &str,
) -> Resolution {
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    for recipe in recipes {
        let result = match recipe {
            Recipe::SvgComponent { name, export } => resolve_svg(build, name, export),
            Recipe::SvgByPath { name, d_prefix } => resolve_svg_by_path(build, name, d_prefix),
            Recipe::SvgByMarkup { name, needles } => resolve_svg_by_markup(build, name, needles),
            Recipe::ContextFile { name, file } => {
                resolve_context_file(build, name, file, asset_base)
            }
            Recipe::AssetFile { name, file } => resolve_pinned_file(build, name, file, asset_base),
            Recipe::DefaultedAsset { name, key } => {
                resolve_defaulted_asset(build, name, key, asset_base)
            }
            Recipe::PathLiteral { name, d_prefix } => resolve_path_literal(build, name, d_prefix),
            Recipe::ContentUrl { name, hash_prefix } => {
                resolve_content_url(build, name, hash_prefix)
            }
            Recipe::LottieJson { name, nm } => resolve_lottie(build, name, nm),
            Recipe::FontFace {
                name,
                family,
                weight,
                italic,
            } => resolve_font(build, name, family, *weight, *italic, asset_base),
        };
        match result {
            Ok(entry) => entries.push(entry),
            Err(reason) => failures.push((recipe.name().to_string(), reason)),
        }
    }

    let mut remap: HashMap<u32, u32> = HashMap::new();
    let mut chunks = Vec::new();
    for entry in &mut entries {
        let slot = match &mut entry.kind {
            EntryKind::SvgComponent { chunk, .. } | EntryKind::Text { chunk, .. } => chunk,
            EntryKind::File { .. } => continue,
        };
        let new = *remap.entry(*slot).or_insert_with(|| {
            let c = &build.chunks[*slot as usize];
            chunks.push(ChunkRef {
                url: c.url.clone(),
                sha256: c.sha256.clone(),
                size: c.size,
            });
            (chunks.len() - 1) as u32
        });
        *slot = new;
    }
    Resolution {
        manifest: Manifest {
            schema: SCHEMA,
            client: ClientInfo {
                build_number: source.build_number,
                build_id: source.build_id.clone(),
                release_channel: release_channel.to_string(),
            },
            asset_base: asset_base.to_string(),
            builds: vec![source.clone()],
            chunks,
            entries,
        },
        failures,
    }
}

fn svg_entry(
    name: &str,
    owner: &Module,
    chunk: usize,
    info: &FunctionInfo,
    export: Option<&str>,
) -> Result<Entry, String> {
    let label = export.unwrap_or("component");
    let icon = match &info.icon {
        Some(Ok(icon)) => icon,
        Some(Err(msg)) => return Err(format!("`{label}`: {msg}")),
        None => {
            return Err(format!(
                "`{label}` resolves to a function that renders no <svg>"
            ));
        }
    };

    if let Err(why) = icon.to_svg() {
        return Err(format!("`{label}`: {why}"));
    }
    Ok(Entry {
        name: name.to_string(),
        build: 0,
        kind: EntryKind::SvgComponent {
            chunk: chunk as u32,
            module_id: owner.id.clone(),
            export: export.map(str::to_string),
            start: info.start,
            end: info.end,
        },
    })
}

fn resolve_svg(build: &Build, name: &str, export: &str) -> Result<Entry, String> {
    let mut last_reason = format!("no module exports `{export}`");
    for (_, module) in build.modules.values() {
        let Some(target) = module.exports.get(export) else {
            continue;
        };
        let Some((owner, chunk, function)) = build.follow(module, target, 0) else {
            last_reason = format!(
                "`{export}` exported from module {} but its target could not be followed",
                module.id
            );
            continue;
        };
        match svg_entry(
            name,
            owner,
            chunk,
            &owner.functions[&function],
            Some(export),
        ) {
            Ok(entry) => return Ok(entry),
            Err(reason) => last_reason = reason,
        }
    }
    Err(last_reason)
}

fn same_paths(a: &FunctionInfo, b: &FunctionInfo) -> bool {
    match (&a.icon, &b.icon) {
        (Some(Ok(a)), Some(Ok(b))) => a.view_box == b.view_box && a.paths == b.paths,
        _ => false,
    }
}

fn resolve_svg_by_path(build: &Build, name: &str, d_prefix: &str) -> Result<Entry, String> {
    let wanted_groups: Vec<Vec<String>> = d_prefix
        .split(" && ")
        .map(path_tokens)
        .filter(|tokens| !tokens.is_empty())
        .collect();
    if wanted_groups.is_empty() {
        return Err("empty path prefix".into());
    }
    let mut found = Vec::new();
    for (chunk, module) in build.modules.values() {
        for (fn_name, info) in &module.functions {
            let Some(Ok(icon)) = &info.icon else { continue };
            let has = |tokens: &[String]| {
                icon.paths.iter().any(|path| {
                    path_tokens(&path.d).starts_with(tokens)
                        || path_tokens(&path.d)
                            .windows(tokens.len())
                            .any(|window| window == tokens)
                })
            };
            if wanted_groups.iter().all(|tokens| has(tokens)) {
                found.push((module, *chunk, fn_name, info));
            }
        }
    }

    match found.as_slice() {
        [] => Err("no component has a path starting with that data".into()),
        [(module, chunk, _, info)] => svg_entry(name, module, *chunk, info, None),
        [(module, chunk, _, info), rest @ ..]
            if rest.iter().all(|(_, _, _, other)| same_paths(info, other)) =>
        {
            svg_entry(name, module, *chunk, info, None)
        }
        many => Err(format!(
            "{} components match that path prefix (modules {})",
            many.len(),
            many.iter()
                .map(|m| m.0.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn resolve_svg_by_markup(build: &Build, name: &str, needles: &[String]) -> Result<Entry, String> {
    if needles.is_empty() {
        return Err("no markup to look for".into());
    }
    let mut found = Vec::new();
    for (chunk, module) in build.modules.values() {
        for (fn_name, info) in &module.functions {
            let Some(Ok(icon)) = &info.icon else { continue };
            let Ok(svg) = icon.to_svg() else { continue };
            if needles.iter().all(|n| svg.contains(n.as_str())) {
                found.push((module, *chunk, fn_name, info));
            }
        }
    }
    match found.as_slice() {
        [] => Err("no component renders markup containing all of those pieces".into()),
        [(module, chunk, _, info)] => svg_entry(name, module, *chunk, info, None),
        many => Err(format!(
            "{} components match that markup (modules {})",
            many.len(),
            many.iter()
                .map(|m| m.0.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn resolve_context_file(
    build: &Build,
    name: &str,
    file: &str,
    asset_base: &str,
) -> Result<Entry, String> {
    let key = format!("./{file}");
    for (_, module) in build.modules.values() {
        let Some(table) = &module.context else {
            continue;
        };
        let Some((_, id)) = table.iter().find(|(k, _)| *k == key) else {
            continue;
        };
        let (_, asset) = build.module(id).ok_or_else(|| {
            format!("`{file}` maps to module {id}, which is not in any fetched chunk")
        })?;
        let hashed = asset
            .asset_file
            .as_ref()
            .ok_or_else(|| format!("module {id} for `{file}` is not an asset module"))?;
        return Ok(Entry {
            name: name.to_string(),
            build: 0,
            kind: EntryKind::File {
                url: format!("{asset_base}{hashed}"),
                sha256: None,
            },
        });
    }
    Err(format!("no require.context table lists `{file}`"))
}

fn resolve_pinned_file(
    build: &Build,
    name: &str,
    file: &str,
    asset_base: &str,
) -> Result<Entry, String> {
    if build.referenced.contains(file) {
        Ok(Entry {
            name: name.to_string(),
            build: 0,
            kind: EntryKind::File {
                url: format!("{asset_base}{file}"),
                sha256: None,
            },
        })
    } else {
        Err(format!("`{file}` is no longer referenced by this build"))
    }
}

fn resolve_content_url(build: &Build, name: &str, hash_prefix: &str) -> Result<Entry, String> {
    const BASE: &str = "https://cdn.discordapp.com/assets/content/";
    let mut found: Vec<&String> = build
        .content_urls
        .iter()
        .filter(|u| {
            u.strip_prefix(BASE)
                .is_some_and(|rest| rest.starts_with(hash_prefix))
        })
        .collect();
    found.sort();
    match found.as_slice() {
        [url] => Ok(Entry {
            name: name.to_string(),
            build: 0,
            kind: EntryKind::File {
                url: (*url).clone(),
                sha256: None,
            },
        }),
        [] => Err(format!(
            "no content URL starting with `{hash_prefix}` in this build"
        )),
        many => Err(format!(
            "{} content URLs start with `{hash_prefix}`",
            many.len()
        )),
    }
}

fn scan_content_urls(source: &str) -> Vec<String> {
    const BASE: &str = "https://cdn.discordapp.com/assets/content/";
    let mut out = Vec::new();
    for part in source.split(BASE).skip(1) {
        let hash_len = part.bytes().take_while(|b| b.is_ascii_hexdigit()).count();
        let Some(rest) = part[hash_len..].strip_prefix('.') else {
            continue;
        };
        let ext_len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric())
            .count();
        if hash_len >= 32 && ext_len > 0 {
            out.push(format!("{BASE}{}.{}", &part[..hash_len], &rest[..ext_len]));
        }
    }
    out
}

fn resolve_path_literal(build: &Build, name: &str, d_prefix: &str) -> Result<Entry, String> {
    let wanted = path_tokens(d_prefix);
    if wanted.is_empty() {
        return Err("empty path prefix".into());
    }
    let mut matches = Vec::new();
    for (chunk, module) in build.modules.values() {
        for literal in &module.path_literals {
            if literal.head.len() >= wanted.len() && literal.head[..wanted.len()] == wanted[..] {
                matches.push((*chunk, literal));
            }
        }
    }
    let Some(&(chunk, first)) = matches.first() else {
        return Err("no string literal holds path data starting with that".into());
    };

    if matches.iter().any(|(_, l)| l.value != first.value) {
        return Err(format!(
            "{} different string literals start with that path data",
            matches.len()
        ));
    }
    Ok(Entry {
        name: name.to_string(),
        build: 0,
        kind: EntryKind::Text {
            chunk: chunk as u32,
            start: first.start,
            end: first.end,
        },
    })
}

fn resolve_defaulted_asset(
    build: &Build,
    name: &str,
    key: &str,
    asset_base: &str,
) -> Result<Entry, String> {
    let mut files = BTreeSet::new();
    for (_, module) in build.modules.values() {
        for (k, default) in &module.param_defaults {
            if k != key {
                continue;
            }
            match default {
                ParamDefault::File(file) => {
                    files.insert(file.clone());
                }
                ParamDefault::Binding(binding) => {
                    if let Some(file) = module.constants.get(binding) {
                        files.insert(file.clone());
                    }
                }
            }
        }
    }
    let mut files = files.into_iter();
    match (files.next(), files.next()) {
        (None, _) => Err(format!(
            "no component has a parameter `{key}` defaulting to an asset"
        )),
        (Some(file), None) => Ok(Entry {
            name: name.to_string(),
            build: 0,
            kind: EntryKind::File {
                url: format!("{asset_base}{file}"),
                sha256: None,
            },
        }),
        (Some(a), Some(b)) => Err(format!(
            "parameter `{key}` defaults to more than one asset ({a}, {b}, ...)"
        )),
    }
}

fn resolve_lottie(build: &Build, name: &str, nm: &str) -> Result<Entry, String> {
    let needle = format!("\"nm\":\"{nm}\"");
    let mut found = Vec::new();
    for (chunk, module) in build.modules.values() {
        let Some(json) = &module.json else { continue };
        let header = json.head.split("\"layers\"").next().unwrap_or(&json.head);
        if header.contains(&needle) {
            found.push((*chunk, json));
        }
    }
    match found.as_slice() {
        [] => Err(format!("no embedded JSON animation named `{nm}`")),
        [(chunk, json)] => Ok(Entry {
            name: name.to_string(),
            build: 0,
            kind: EntryKind::Text {
                chunk: *chunk as u32,
                start: json.start,
                end: json.end,
            },
        }),
        many => Err(format!("{} animations named `{nm}`", many.len())),
    }
}

fn resolve_font(
    build: &Build,
    name: &str,
    family: &str,
    weight: u16,
    italic: bool,
    asset_base: &str,
) -> Result<Entry, String> {
    let weight = weight.to_string();
    let mut matches = build
        .fonts
        .iter()
        .filter(|f| f.family == family && f.weight == weight && f.italic == italic);
    let first = matches.next().ok_or_else(|| {
        format!(
            "no @font-face for {family} {weight}{}",
            if italic { " italic" } else { "" }
        )
    })?;
    if matches.any(|f| f.file != first.file) {
        return Err(format!(
            "{family} {weight} is declared with more than one file"
        ));
    }
    Ok(Entry {
        name: name.to_string(),
        build: 0,
        kind: EntryKind::File {
            url: format!("{asset_base}{}", first.file),
            sha256: None,
        },
    })
}

fn scan_asset_refs(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in source.split("/assets/").skip(1) {
        let hash_len = part.bytes().take_while(|b| b.is_ascii_hexdigit()).count();
        if !(16..=32).contains(&hash_len) {
            continue;
        }
        let rest = &part[hash_len..];
        let Some(rest) = rest.strip_prefix('.') else {
            continue;
        };
        let ext_len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric())
            .count();
        if ext_len > 0 {
            out.push(format!("{}.{}", &part[..hash_len], &rest[..ext_len]));
        }
    }
    out
}

fn parse_font_faces(css: &str) -> Vec<FontRule> {
    let mut out = Vec::new();
    for rule in css.split("@font-face").skip(1) {
        let Some(body) = rule
            .trim_start()
            .strip_prefix('{')
            .and_then(|r| r.split('}').next())
        else {
            continue;
        };
        let (mut family, mut weight, mut italic, mut file) = (None, None, false, None);
        for decl in body.split(';') {
            let Some((prop, value)) = decl.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match prop.trim() {
                "font-family" => family = Some(value.trim_matches('"').to_string()),
                "font-weight" => weight = Some(value.to_string()),
                "font-style" => italic = value == "italic",
                "src" => {
                    file = value
                        .split_once("url(")
                        .and_then(|(_, r)| r.split(')').next())
                        .and_then(|u| u.trim_matches('"').rsplit('/').next().map(str::to_string));
                }
                _ => {}
            }
        }
        if let (Some(family), Some(weight), Some(file)) = (family, weight, file) {
            out.push(FontRule {
                family,
                weight,
                italic,
                file,
            });
        }
    }
    out
}

pub(crate) fn path_tokens(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut seen_dot = false;
    let flush = |tokens: &mut Vec<String>, current: &mut String, seen_dot: &mut bool| {
        if !current.is_empty() {
            tokens.push(mem::take(current));
        }
        *seen_dot = false;
    };
    for c in d.chars() {
        match c {
            '0'..='9' => current.push(c),
            '.' => {
                if seen_dot {
                    flush(&mut tokens, &mut current, &mut seen_dot);
                }
                seen_dot = true;
                current.push(c);
            }
            '-' => {
                flush(&mut tokens, &mut current, &mut seen_dot);
                current.push(c);
            }
            c if c.is_ascii_alphabetic() => {
                flush(&mut tokens, &mut current, &mut seen_dot);
                tokens.push(c.to_string());
            }
            _ => flush(&mut tokens, &mut current, &mut seen_dot),
        }
    }
    flush(&mut tokens, &mut current, &mut seen_dot);
    tokens
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
