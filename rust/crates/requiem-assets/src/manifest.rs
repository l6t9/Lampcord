use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::collections;

pub const SCHEMA: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,

    pub client: ClientInfo,

    pub asset_base: String,

    pub builds: Vec<BuildRef>,

    pub chunks: Vec<ChunkRef>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientInfo {
    pub build_number: Option<u64>,

    pub build_id: String,

    pub release_channel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildRef {
    pub build_id: String,
    pub build_number: Option<u64>,
}

impl BuildRef {
    pub fn new(build_id: &str, build_number: Option<u64>) -> Self {
        Self {
            build_id: build_id.to_string(),
            build_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkRef {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,

    pub build: u32,
    #[serde(flatten)]
    pub kind: EntryKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntryKind {
    File {
        url: String,
        sha256: Option<String>,
    },

    SvgComponent {
        chunk: u32,
        module_id: String,
        export: Option<String>,
        start: u32,
        end: u32,
    },

    Text {
        chunk: u32,
        start: u32,
        end: u32,
    },
}

impl Manifest {
    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn compose(base: &Manifest, overlay: &Manifest) -> Manifest {
        let mut out = Manifest {
            schema: SCHEMA,
            client: overlay.client.clone(),
            asset_base: overlay.asset_base.clone(),
            builds: Vec::new(),
            chunks: Vec::new(),
            entries: Vec::new(),
        };
        let mut builds: HashMap<String, u32> = HashMap::new();
        let mut chunks: HashMap<String, u32> = HashMap::new();
        let overridden: collections::HashSet<&str> =
            overlay.entries.iter().map(|e| e.name.as_str()).collect();

        let mut add = |source: &Manifest, entry: &Entry, out: &mut Manifest| {
            let build = &source.builds[entry.build as usize];
            let build_index = *builds.entry(build.build_id.clone()).or_insert_with(|| {
                out.builds.push(build.clone());
                (out.builds.len() - 1) as u32
            });
            let mut kind = entry.kind.clone();
            if let EntryKind::SvgComponent { chunk, .. } | EntryKind::Text { chunk, .. } = &mut kind
            {
                let source_chunk = &source.chunks[*chunk as usize];
                *chunk = *chunks.entry(source_chunk.url.clone()).or_insert_with(|| {
                    out.chunks.push(source_chunk.clone());
                    (out.chunks.len() - 1) as u32
                });
            }
            out.entries.push(Entry {
                name: entry.name.clone(),
                build: build_index,
                kind,
            });
        };
        for entry in base
            .entries
            .iter()
            .filter(|e| !overridden.contains(e.name.as_str()))
        {
            add(base, entry, &mut out);
        }
        for entry in &overlay.entries {
            add(overlay, entry, &mut out);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(build: &str, entries: &[(&str, u32)]) -> Manifest {
        Manifest {
            schema: SCHEMA,
            client: ClientInfo {
                build_number: Some(1),
                build_id: build.into(),
                release_channel: "stable".into(),
            },
            asset_base: "https://d/assets/".into(),
            builds: vec![BuildRef::new(build, Some(1))],
            chunks: vec![
                ChunkRef {
                    url: format!("https://d/assets/{build}-a.js"),
                    sha256: "a".into(),
                    size: 1,
                },
                ChunkRef {
                    url: "https://d/assets/shared.js".into(),
                    sha256: "s".into(),
                    size: 1,
                },
            ],
            entries: entries
                .iter()
                .map(|(n, chunk)| Entry {
                    name: (*n).into(),
                    build: 0,
                    kind: EntryKind::Text {
                        chunk: *chunk,
                        start: 0,
                        end: 1,
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn compose_keeps_old_entries_and_remaps_indices() {
        let old = manifest("old", &[("a", 0), ("b", 1)]);
        let new = manifest("new", &[("b", 0), ("c", 1)]);
        let merged = Manifest::compose(&old, &new);

        assert_eq!(merged.client.build_id, "new");
        assert_eq!(
            merged
                .builds
                .iter()
                .map(|b| b.build_id.as_str())
                .collect::<Vec<_>>(),
            ["old", "new"]
        );
        let names: Vec<_> = merged.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);

        let a = merged.entry("a").unwrap();
        assert_eq!(merged.builds[a.build as usize].build_id, "old");

        assert_eq!(
            merged.builds[merged.entry("b").unwrap().build as usize].build_id,
            "new"
        );

        assert_eq!(
            merged
                .chunks
                .iter()
                .filter(|c| c.url.ends_with("shared.js"))
                .count(),
            1
        );
        let chunk_of = |n: &str| match merged.entry(n).unwrap().kind {
            EntryKind::Text { chunk, .. } => merged.chunks[chunk as usize].url.clone(),
            _ => unreachable!(),
        };
        assert_eq!(chunk_of("c"), "https://d/assets/shared.js");
        assert_eq!(chunk_of("a"), "https://d/assets/old-a.js");
    }
}
