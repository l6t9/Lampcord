use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub fn process_crate(manifest_dir: &str, out_dir: &str) {
    let src_dir = Path::new(manifest_dir).join("src");
    let mut module_files = Vec::new();
    let mut global_files = Vec::new();
    collect_css(&src_dir, &mut module_files, &mut global_files);
    module_files.sort();
    global_files.sort();

    let mut bundled = String::new();

    let mut modules: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    let mut seen_hashes: HashMap<String, (String, String)> = HashMap::new();

    for path in &global_files {
        let rel_path = rel_path_str(path, manifest_dir);
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("requiem-css: failed to read {}: {e}", path.display()));
        bundled.push_str("/* ");
        bundled.push_str(&rel_path);
        bundled.push_str(" (global) */\n");
        bundled.push_str(&text);
        bundled.push('\n');
    }

    for path in &module_files {
        let rel_path = rel_path_str(path, manifest_dir);
        let file_name = path.file_name().unwrap().to_string_lossy();
        let stem = file_name
            .strip_suffix(".module.css")
            .expect("collect_css only classifies *.module.css files as modules")
            .to_string();

        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("requiem-css: failed to read {}: {e}", path.display()));

        let class_map = modules.entry(stem.clone()).or_default();
        let rewritten = rewrite_stylesheet(&text, &rel_path, &stem, class_map, &mut seen_hashes);

        bundled.push_str("/* ");
        bundled.push_str(&rel_path);
        bundled.push_str(" */\n");
        bundled.push_str(&rewritten);
        bundled.push('\n');
    }

    let out_dir = Path::new(out_dir);
    fs::write(out_dir.join("bundled.css"), bundled)
        .expect("requiem-css: failed to write bundled.css");
    fs::write(out_dir.join("css_classes.rs"), generate_rust(&modules))
        .expect("requiem-css: failed to write css_classes.rs");
    fs::write(out_dir.join("css_modules.json"), generate_json(&modules))
        .expect("requiem-css: failed to write css_modules.json");
}

fn generate_json(modules: &BTreeMap<String, BTreeMap<String, String>>) -> String {
    let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    let stems: Vec<String> = modules
        .iter()
        .map(|(stem, classes)| {
            let entries: Vec<String> = classes
                .iter()
                .map(|(local, hashed)| format!("{}:{}", quote(local), quote(hashed)))
                .collect();
            format!("{}:{{{}}}", quote(stem), entries.join(","))
        })
        .collect();
    format!("{{{}}}", stems.join(","))
}

fn rel_path_str(path: &Path, manifest_dir: &str) -> String {
    path.strip_prefix(manifest_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn collect_css(dir: &Path, modules: &mut Vec<PathBuf>, globals: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_css(&path, modules, globals);
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.ends_with(".module.css") {
            modules.push(path);
        } else if name.ends_with(".css") {
            globals.push(path);
        }
    }
}

fn rewrite_stylesheet(
    css: &str,
    rel_path: &str,
    stem: &str,
    class_map: &mut BTreeMap<String, String>,
    seen_hashes: &mut HashMap<String, (String, String)>,
) -> String {
    let chars: Vec<char> = css.chars().collect();
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    let mut prelude_start = 0;

    while i < chars.len() {
        if let Some(end) = skip_comment_or_string(&chars, i) {
            i = end;
            continue;
        }
        if chars[i] == '{' {
            let prelude: String = chars[prelude_start..i].iter().collect();
            out.push_str(&rewrite_prelude(
                &prelude,
                rel_path,
                stem,
                class_map,
                seen_hashes,
            ));
            let block_start = i;
            let mut depth = 1;
            i += 1;
            while i < chars.len() && depth > 0 {
                match chars[i] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            out.push_str(&chars[block_start..i].iter().collect::<String>());
            prelude_start = i;
            continue;
        }
        i += 1;
    }
    out.push_str(&chars[prelude_start..].iter().collect::<String>());
    out
}

fn skip_comment_or_string(chars: &[char], i: usize) -> Option<usize> {
    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
        let mut j = i + 2;
        while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == '/') {
            j += 1;
        }
        return Some((j + 2).min(chars.len()));
    }
    if chars[i] == '"' || chars[i] == '\'' {
        let quote = chars[i];
        let mut j = i + 1;
        while j < chars.len() && chars[j] != quote {
            if chars[j] == '\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(chars.len()));
    }
    None
}

fn rewrite_prelude(
    prelude: &str,
    rel_path: &str,
    stem: &str,
    class_map: &mut BTreeMap<String, String>,
    seen_hashes: &mut HashMap<String, (String, String)>,
) -> String {
    let chars: Vec<char> = prelude.chars().collect();
    let mut out = String::with_capacity(prelude.len());
    let mut i = 0;

    while i < chars.len() {
        let is_class_start = chars[i] == '.'
            && chars
                .get(i + 1)
                .is_some_and(|c| c.is_alphabetic() || *c == '_');
        if let Some(end) = skip_comment_or_string(&chars, i) {
            out.push_str(&chars[i..end].iter().collect::<String>());
            i = end;
        } else if is_class_start {
            let start = i + 1;
            let mut j = start;
            while j < chars.len()
                && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '-')
            {
                j += 1;
            }
            let local_name: String = chars[start..j].iter().collect();
            let hashed = hashed_name_for(&local_name, rel_path, stem, class_map, seen_hashes);
            out.push('.');
            out.push_str(&hashed);
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn hashed_name_for(
    local_name: &str,
    rel_path: &str,
    stem: &str,
    class_map: &mut BTreeMap<String, String>,
    seen_hashes: &mut HashMap<String, (String, String)>,
) -> String {
    if let Some(existing) = class_map.get(local_name) {
        return existing.clone();
    }
    let hash_input = format!("{rel_path}::{local_name}");
    let hashed = format!("{stem}-{local_name}_{}", base36_6(fnv1a64(&hash_input)));

    if let Some((prev_path, prev_local)) = seen_hashes.get(&hashed) {
        if prev_path != rel_path || prev_local != local_name {
            panic!(
                "requiem-css: hash collision on generated class '{hashed}' between \
                 {prev_path}::{prev_local} and {rel_path}::{local_name} — rename one of the source classes"
            );
        }
    } else {
        seen_hashes.insert(
            hashed.clone(),
            (rel_path.to_string(), local_name.to_string()),
        );
    }

    class_map.insert(local_name.to_string(), hashed.clone());
    hashed
}

fn fnv1a64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn base36_6(hash: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    let mut n = hash & 0x7FFF_FFFF;
    let mut out = [0u8; 6];
    for slot in out.iter_mut().rev() {
        *slot = DIGITS[(n % 36) as usize];
        n /= 36;
    }
    String::from_utf8(out.to_vec()).unwrap()
}

fn generate_rust(modules: &BTreeMap<String, BTreeMap<String, String>>) -> String {
    let mut out = String::new();
    out.push_str("// @generated by requiem-css build.rs integration — do not edit by hand.\n");
    for (stem, classes) in modules {
        out.push_str(&format!("pub mod {} {{\n", sanitize_ident(stem)));
        for (local, hashed) in classes {
            out.push_str(&format!(
                "    pub const {}: &str = \"{}\";\n",
                scream_snake(local),
                hashed
            ));
        }
        out.push_str("}\n");
    }
    out
}

fn sanitize_ident(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn scream_snake(s: &str) -> String {
    sanitize_ident(s).to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_hashes_identically() {
        assert_eq!(base36_6(fnv1a64("a::b")), base36_6(fnv1a64("a::b")));
    }

    #[test]
    fn different_path_changes_hash() {
        assert_ne!(
            base36_6(fnv1a64("a.module.css::root")),
            base36_6(fnv1a64("b.module.css::root"))
        );
    }

    #[test]
    fn rewrites_only_selector_classes_not_declarations() {
        let mut class_map = BTreeMap::new();
        let mut seen = HashMap::new();
        let css =
            ".root, .root:hover { opacity: 0.5; background: rgba(0,0,0,0.24); --x: .not-a-class; }";
        let out = rewrite_stylesheet(css, "src/x.module.css", "x", &mut class_map, &mut seen);

        assert!(out.contains("opacity: 0.5; background: rgba(0,0,0,0.24); --x: .not-a-class;"));

        let hashed = class_map.get("root").unwrap();
        assert_eq!(out.matches(hashed.as_str()).count(), 2);
    }

    #[test]
    fn comment_mentioning_a_class_name_is_not_treated_as_a_selector() {
        let mut class_map = BTreeMap::new();
        let mut seen = HashMap::new();
        let css = "/* see .modeSelected and .module.css elsewhere */\n.real-btn { color: red; }";
        let out = rewrite_stylesheet(css, "src/x.module.css", "x", &mut class_map, &mut seen);

        assert_eq!(class_map.keys().collect::<Vec<_>>(), vec!["real-btn"]);
        assert!(out.contains("see .modeSelected and .module.css elsewhere"));
    }

    #[test]
    fn compound_and_comma_selectors_all_rewritten() {
        let mut class_map = BTreeMap::new();
        let mut seen = HashMap::new();
        let css = ".a-btn, .b-btn { color: red; }\n.a-btn-selected:hover, .a-btn-selected:active { color: blue; }";
        let out = rewrite_stylesheet(css, "src/x.module.css", "x", &mut class_map, &mut seen);
        assert_eq!(class_map.len(), 3);
        for hashed in class_map.values() {
            assert!(out.contains(hashed.as_str()));
        }
    }
}
