use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct CssModules {
    modules: BTreeMap<String, BTreeMap<String, String>>,
}

impl CssModules {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let parsed: BTreeMap<String, BTreeMap<String, String>> =
            serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(Self { modules: parsed })
    }

    pub fn insert(&mut self, stem: &str, class: &str, hashed: &str) {
        self.modules
            .entry(stem.to_string())
            .or_default()
            .insert(class.to_string(), hashed.to_string());
    }

    pub fn module(&self, stem: &str) -> Option<&BTreeMap<String, String>> {
        self.modules.get(stem)
    }

    pub fn stems(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(String::as_str)
    }

    pub fn class(&self, stem: &str, name: &str) -> Option<&str> {
        let module = self.modules.get(stem)?;
        module
            .get(name)
            .or_else(|| module.get(&kebab(name)))
            .map(String::as_str)
    }
}

pub fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub fn camel(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in name.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}
