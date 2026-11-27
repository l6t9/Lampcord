use serde::{Deserialize, Serialize};

pub mod component_type {
    pub const ACTION_ROW: u32 = 1;
    pub const BUTTON: u32 = 2;
    pub const STRING_SELECT: u32 = 3;
    pub const SECTION: u32 = 9;
    pub const TEXT_DISPLAY: u32 = 10;
    pub const THUMBNAIL: u32 = 11;
    pub const MEDIA_GALLERY: u32 = 12;
    pub const FILE: u32 = 13;
    pub const SEPARATOR: u32 = 14;
    pub const CONTAINER: u32 = 17;
}

pub mod button_style {
    pub const PRIMARY: u32 = 1;
    pub const SUCCESS: u32 = 3;
    pub const DANGER: u32 = 4;
    pub const LINK: u32 = 5;
}

/// A message component of any type. The kinds share one shape so an unknown
/// or future component still parses; each reads only the fields it uses.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "type", default)]
    pub kind: u32,

    /// Children of a row, section, or container.
    #[serde(default)]
    pub components: Vec<Component>,

    /// A section's button or thumbnail.
    #[serde(default)]
    pub accessory: Option<Box<Component>>,

    /// Text display markdown.
    #[serde(default)]
    pub content: Option<String>,

    /// Thumbnail image.
    #[serde(default)]
    pub media: Option<ComponentMedia>,

    /// Attached file.
    #[serde(default)]
    pub file: Option<ComponentMedia>,

    #[serde(default)]
    pub items: Vec<GalleryItem>,

    #[serde(default)]
    pub divider: Option<bool>,

    /// Separator padding: 1 small, 2 large.
    #[serde(default)]
    pub spacing: Option<u32>,

    #[serde(default)]
    pub accent_color: Option<u32>,

    #[serde(default)]
    pub spoiler: bool,

    #[serde(default)]
    pub style: Option<u32>,

    #[serde(default)]
    pub label: Option<String>,

    #[serde(default)]
    pub emoji: Option<ComponentEmoji>,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub disabled: bool,

    #[serde(default)]
    pub placeholder: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentMedia {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub placeholder: Option<String>,
}

impl ComponentMedia {
    pub fn source(&self) -> &str {
        self.proxy_url.as_deref().unwrap_or(&self.url)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GalleryItem {
    #[serde(default)]
    pub media: ComponentMedia,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub spoiler: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentEmoji {
    #[serde(default, deserialize_with = "id_text")]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub animated: bool,
}

/// A snowflake that arrives as a string or as a bare number.
fn id_text<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Ok(match Option::<serde_json::Value>::deserialize(deserializer)? {
        Some(serde_json::Value::String(text)) => Some(text),
        Some(serde_json::Value::Number(number)) => Some(number.to_string()),
        _ => None,
    })
}

impl Component {
    pub fn text(&self) -> &str {
        self.content.as_deref().unwrap_or_default()
    }

    /// Text a component contributes to the message for search and copy.
    pub fn collect_text(&self, out: &mut Vec<String>) {
        if let Some(content) = self.content.as_deref().filter(|c| !c.is_empty()) {
            out.push(content.to_string());
        }
        for child in &self.components {
            child.collect_text(out);
        }
        if let Some(accessory) = &self.accessory {
            accessory.collect_text(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_v2_container_parses_with_nested_children() {
        let json = r#"{"type":17,"accent_color":5793266,"components":[
            {"type":10,"content":"hi"},
            {"type":9,"components":[{"type":10,"content":"x"}],"accessory":{"type":2,"style":1,"label":"Go","custom_id":"a"}},
            {"type":12,"items":[{"media":{"url":"https://x/y.png"}}]},
            {"type":99,"mystery":true}]}"#;
        let component: Component = serde_json::from_str(json).unwrap();
        assert_eq!(component.kind, component_type::CONTAINER);
        assert_eq!(component.components.len(), 4);
        assert_eq!(
            component.components[1].accessory.as_ref().unwrap().label.as_deref(),
            Some("Go")
        );
        let mut text = Vec::new();
        component.collect_text(&mut text);
        assert_eq!(text, ["hi", "x"]);
    }
}
