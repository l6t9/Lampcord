use serde::{Deserialize, Serialize};

use super::id::{Id, marker::AttachmentMarker};
fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Id<AttachmentMarker>,
    pub filename: String,
    pub size: u64,
    #[serde(default)]
    pub content_type: Option<String>,

    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    pub url: String,
    #[serde(default)]
    pub proxy_url: Option<String>,

    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub placeholder_version: Option<u32>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub flags: u32,

    #[serde(default)]
    pub description: Option<String>,
}

impl Attachment {
    pub fn is_spoiler(&self) -> bool {
        self.flags & 8 != 0 || self.filename.starts_with("SPOILER_")
    }

    pub fn is_image(&self) -> bool {
        if let Some(ct) = &self.content_type {
            return ct.starts_with("image/");
        }
        let lower = self.filename.to_ascii_lowercase();
        [".png", ".jpg", ".jpeg", ".gif", ".webp"]
            .iter()
            .any(|ext| lower.ends_with(ext))
    }

    pub fn is_video(&self) -> bool {
        if let Some(ct) = &self.content_type {
            return ct.starts_with("video/");
        }
        let lower = self.filename.to_ascii_lowercase();
        [".mp4", ".webm", ".mov", ".mkv", ".m4v"]
            .iter()
            .any(|ext| lower.ends_with(ext))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmbedMedia {
    pub url: String,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,

    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub placeholder_version: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmbedAuthor {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmbedProvider {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Embed {
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,

    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub footer: Option<EmbedFooter>,
    #[serde(default)]
    pub image: Option<EmbedMedia>,
    #[serde(default)]
    pub thumbnail: Option<EmbedMedia>,
    #[serde(default)]
    pub video: Option<EmbedMedia>,
    #[serde(default)]
    pub author: Option<EmbedAuthor>,
    #[serde(default)]
    pub provider: Option<EmbedProvider>,
    #[serde(default)]
    pub fields: Option<Vec<EmbedField>>,
}

#[cfg(test)]
mod spacebar_tests {
    use super::*;

    #[test]
    fn an_attachment_with_null_flags_and_placeholders_decodes() {
        let json = r#"{"id":"1","filename":"a.png","size":5,"height":8,"width":9,"content_type":"image/png","message_id":"2","channel_id":"3","description":null,"flags":null,"content_scan_version":null,"placeholder_version":null,"placeholder":null,"duration_secs":null,"waveform":null,"title":null,"clip_created_at":null,"url":"https://x/a.png","proxy_url":"https://x/a.png"}"#;
        let attachment: Attachment = serde_json::from_str(json).expect("null flags decode");
        assert_eq!(attachment.flags, 0);
    }
}
