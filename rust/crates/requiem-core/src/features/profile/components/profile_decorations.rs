use serde_json::Value;

use requiem_ui::node::Node;
use requiem_ui::widget::primitives::ImageWidget;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::user_profile::{ProfileMeta, UserProfile};
use crate::query::QueryClient;
use crate::rest::endpoints::users;
use crate::rest::{HttpClient, RestError};
use crate::ui::classes::profile_popout as cls;

const COLLECTIBLE_TYPE_EFFECT: i64 = 1;
const COLLECTIBLE_TYPE_FRAME: i64 = 3;

/// Profile frames are temporarily hidden on desktop while their layering is
/// being reviewed. Effects, nameplates and avatar decorations stay visible.
const SHOW_PROFILE_FRAMES: bool = false;

fn meta_sku(meta: Option<&ProfileMeta>, kind: i64) -> Option<String> {
    let meta = meta?;
    let direct = if kind == COLLECTIBLE_TYPE_EFFECT {
        meta.profile_effect.as_ref().and_then(|effect| effect.sku_id.as_deref())
    } else {
        meta.profile_frame.as_ref().and_then(|frame| frame.sku_id.as_deref())
    };
    if let Some(sku) = direct.filter(|sku| !sku.trim().is_empty()) {
        return Some(sku.to_string());
    }
    meta.collectibles
        .as_ref()?
        .iter()
        .find(|item| item.kind == Some(kind as u8))
        .and_then(|item| item.sku_id.as_deref())
        .filter(|sku| !sku.trim().is_empty())
        .map(str::to_string)
}

/// Guild cosmetics win over the user's own, matching the mobile client.
pub fn effect_sku(profile: &UserProfile) -> Option<String> {
    meta_sku(profile.guild_member_profile.as_ref(), COLLECTIBLE_TYPE_EFFECT)
        .or_else(|| meta_sku(profile.user_profile.as_ref(), COLLECTIBLE_TYPE_EFFECT))
}

pub fn frame_sku(profile: &UserProfile) -> Option<String> {
    meta_sku(profile.guild_member_profile.as_ref(), COLLECTIBLE_TYPE_FRAME)
        .or_else(|| meta_sku(profile.user_profile.as_ref(), COLLECTIBLE_TYPE_FRAME))
}

fn load_product(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    sku: &str,
) -> Option<Value> {
    let key = format!("collectible-product:{sku}");
    if let Some(http) = http {
        let http = http.clone();
        let owned = sku.to_string();
        let result = queries.query_default::<Value, _, RestError>(&key, move || async move {
            users::get_collectible_product(&http, &owned).await
        });
        result.data.as_deref().cloned()
    } else {
        queries.peek::<Value>(&key).as_deref().cloned()
    }
}

fn as_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn as_text(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty() && *text != "null")
}

fn item_of_kind(product: &Value, kind: i64) -> Option<&Value> {
    product
        .get("items")?
        .as_array()?
        .iter()
        .find(|item| as_i64(item.get("type")) == Some(kind))
}

fn media_url(backend: &BackendConfig, raw: &str) -> String {
    let raw = raw.trim();
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else if let Some(rest) = raw.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        format!(
            "{}/{}",
            backend.cdn_base_url.trim_end_matches('/'),
            raw.trim_start_matches('/')
        )
    }
}

fn image_node(image: skia_safe::Image, class: &'static str) -> Node {
    Node::new("div", ImageWidget::new(Some(image)).with_cover_centered()).class(class)
}

/// The collectible nameplate shown behind account and row surfaces at 40% opacity.
pub fn nameplate_node(
    asset: Option<&str>,
    images: &ImageCache,
    backend: &BackendConfig,
    class: &'static str,
) -> Option<Node> {
    let asset = asset?;
    let image = images.get_or_decode_image(&backend.nameplate_url(asset))?;
    Some(image_node(image, class))
}

/// Static effect and frame layers for a profile card. The animated mobile effect
/// is intentionally rendered as its first frame on the Skia desktop client.
pub fn layers(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    profile: &UserProfile,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Vec<Node> {
    let mut out = Vec::new();

    let effect_sku = effect_sku(profile);
    if let Some(product) = effect_sku
        .as_deref()
        .and_then(|sku| load_product(queries, http, sku))
    {
        if let Some(item) = item_of_kind(&product, COLLECTIBLE_TYPE_EFFECT) {
            let source = as_text(item.get("reducedMotionSrc"))
                .or_else(|| as_text(item.get("reduced_motion_src")))
                .or_else(|| as_text(item.get("staticFrameSrc")))
                .or_else(|| as_text(item.get("static_frame_src")))
                .or_else(|| {
                    item.get("effects")?
                        .as_array()?
                        .first()?
                        .get("src")
                        .and_then(Value::as_str)
                });
            if let Some(image) = source
                .map(|url| media_url(backend, url))
                .and_then(|url| images.get_or_decode_image(&url))
            {
                out.push(image_node(image, cls::PROFILE_EFFECT));
            }
        }
    }

    if SHOW_PROFILE_FRAMES {
        if let Some(sku) = frame_sku(profile) {
            if let Some(product) = load_product(queries, http, &sku) {
                if let Some(item) = item_of_kind(&product, COLLECTIBLE_TYPE_FRAME) {
                    let mut layers = item
                        .get("layers")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|layer| as_text(layer.get("id")))
                        .take(6)
                        .filter_map(|id| {
                            let url = backend.collectible_media_url(&sku, id);
                            images.get_or_decode_image(&url)
                        });
                    out.extend(
                        layers.by_ref().map(|image| image_node(image, cls::PROFILE_FRAME_LAYER)),
                    );
                }
            }
        }
    }

    out
}
