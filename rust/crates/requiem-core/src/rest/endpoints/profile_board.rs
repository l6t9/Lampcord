use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    config::BackendConfig,
    models::{Id, id::marker::UserMarker},
    rest::{HttpClient, RequestExtras, RestError},
};

/// One game listed in a profile board widget.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardGameEntry {
    pub game_id: String,
    pub comment: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A widget from `/users/:id/profile`. Only the known board widget types are kept.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardWidget {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default)]
    pub games: Vec<BoardGameEntry>,
    pub application_id: Option<String>,
}

/// Resolved title artwork for a game id.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardGameInfo {
    pub name: String,
    pub image: Option<String>,
}

/// A single value/label pair from an application widget's bottom surface.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardStat {
    pub value: String,
    pub label: String,
}

/// Resolved identity and widget configuration for an application widget.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardApplication {
    pub name: String,
    pub icon: Option<String>,
    pub title: Option<String>,
    pub image: Option<String>,
    #[serde(default)]
    pub subtitles: Vec<String>,
    #[serde(default)]
    pub stats: Vec<BoardStat>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBoard {
    #[serde(default)]
    pub widgets: Vec<BoardWidget>,
    #[serde(default)]
    pub games: HashMap<String, BoardGameInfo>,
    #[serde(default)]
    pub applications: HashMap<String, BoardApplication>,
}

impl ProfileBoard {
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }
}

const UNKNOWN_GAME_NAME: &str = "Unknown game";
const DEFAULT_APP_NAME: &str = "Game Stats";
const GAME_BATCH: usize = 25;
const LOCATION: &str = "profile_board";
const WIDGET_TYPES: &[&str] = &[
    "favorite_games",
    "played_games",
    "current_games",
    "want_to_play_games",
    "application",
];

/// Loads and resolves the game/application widgets on a user's profile.
///
/// Failing auxiliary lookups leave the affected entries unresolved instead of failing the whole
/// board. A profile without widgets yields an empty [`ProfileBoard`].
pub async fn load(
    http: &HttpClient,
    config: &BackendConfig,
    user_id: Id<UserMarker>,
) -> Result<ProfileBoard, RestError> {
    let extras = RequestExtras::context(LOCATION);
    let path =
        format!("/users/{user_id}/profile?with_mutual_guilds=false&with_mutual_friends=false");
    let Some(profile) = get_json(http, "GET /users/:id/profile", &path, &extras).await? else {
        return Ok(ProfileBoard::default());
    };

    let widgets = parse_widgets(profile.get("widgets"));
    if widgets.is_empty() {
        return Ok(ProfileBoard::default());
    }

    let mut game_ids: Vec<String> = widgets
        .iter()
        .flat_map(|widget| widget.games.iter().map(|game| game.game_id.clone()))
        .collect();
    dedupe_strings(&mut game_ids);

    let games = resolve_games(http, config, &game_ids, &extras).await;
    let applications = resolve_applications(http, config, user_id, &widgets, &extras).await;

    Ok(ProfileBoard {
        widgets,
        games,
        applications,
    })
}

async fn get_json(
    http: &HttpClient,
    route: &str,
    path: &str,
    extras: &RequestExtras,
) -> Result<Option<Value>, RestError> {
    let response = match http
        .request_with(route, wreq::Method::GET, path, None::<&()>, extras)
        .await
    {
        Ok(response) => response,
        Err(RestError::Status { .. }) => return Ok(None),
        Err(err) => return Err(err),
    };
    let body = response.text().await?;
    Ok(serde_json::from_str(&body).ok())
}

fn dedupe_strings(items: &mut Vec<String>) {
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.clone()));
}

fn parse_widgets(element: Option<&Value>) -> Vec<BoardWidget> {
    let Some(array) = element.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut widgets = Vec::new();
    for wrapper in array {
        let Some(data) = wrapper.get("data").and_then(Value::as_object) else {
            continue;
        };
        let Some(widget_type) = text(data.get("type")) else {
            continue;
        };
        if !WIDGET_TYPES.contains(&widget_type.as_str()) {
            continue;
        }
        let games: Vec<BoardGameEntry> = data
            .get("games")
            .and_then(Value::as_array)
            .map(|games| {
                games
                    .iter()
                    .filter_map(|game| {
                        let game = game.as_object()?;
                        let game_id = text(game.get("game_id"))?;
                        let tags = game
                            .get("tags")
                            .and_then(Value::as_array)
                            .map(|tags| tags.iter().filter_map(text_or_null).collect())
                            .unwrap_or_default();
                        Some(BoardGameEntry {
                            game_id,
                            comment: text(game.get("comment")),
                            tags,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let application_id = text(data.get("application_id"));
        if games.is_empty() && application_id.is_none() {
            continue;
        }
        widgets.push(BoardWidget {
            widget_type,
            games,
            application_id,
        });
    }
    widgets
}

async fn resolve_games(
    http: &HttpClient,
    config: &BackendConfig,
    ids: &[String],
    extras: &RequestExtras,
) -> HashMap<String, BoardGameInfo> {
    let mut games = HashMap::new();

    for batch in ids.chunks(GAME_BATCH) {
        let query = batch
            .iter()
            .map(|id| format!("game_ids={id}"))
            .collect::<Vec<_>>()
            .join("&");
        let path = format!("/games?{query}");
        let Some(value) = get_json(http, "GET /games", &path, extras)
            .await
            .ok()
            .flatten()
        else {
            continue;
        };
        let array = match &value {
            Value::Array(array) => Some(array),
            Value::Object(object) => object.get("games").and_then(Value::as_array),
            _ => None,
        };
        if let Some(array) = array {
            for element in array {
                if let Some((id, info)) = parse_game(element.as_object(), config) {
                    games.insert(id, info);
                }
            }
        }
    }

    for id in ids {
        if games.contains_key(id) {
            continue;
        }
        let path = format!("/games/{id}");
        let Some(value) = get_json(http, "GET /games/:id", &path, extras)
            .await
            .ok()
            .flatten()
        else {
            continue;
        };
        if let Some((game_id, info)) = parse_game(value.as_object(), config) {
            games.insert(game_id, info);
        }
    }

    games
}

fn parse_game(
    game: Option<&Map<String, Value>>,
    config: &BackendConfig,
) -> Option<(String, BoardGameInfo)> {
    let game = game?;
    let id = text(game.get("id"))?;
    let supplemental = game
        .get("supplemental_game_data")
        .and_then(Value::as_object);
    let name = text(game.get("name"))
        .filter(|name| name != &id)
        .or_else(|| text(supplemental.and_then(|data| data.get("name"))).filter(|name| name != &id))
        .unwrap_or_else(|| UNKNOWN_GAME_NAME.to_owned());

    let artwork = supplemental
        .and_then(|data| data.get("artwork_urls"))
        .and_then(Value::as_array)
        .and_then(|urls| urls.first())
        .and_then(text_or_null)
        .and_then(|url| normalize_media_url(&url, config));
    let cover = text(supplemental.and_then(|data| data.get("cover_image_url")))
        .and_then(|url| normalize_media_url(&url, config));
    let icon = text(supplemental.and_then(|data| data.get("icon_hash")))
        .or_else(|| text(game.get("icon_hash")));

    let media_cover = game
        .get("media")
        .and_then(Value::as_object)
        .and_then(|media| media.get("cover"))
        .and_then(Value::as_object);
    let media_cover_url = match text(media_cover.and_then(|cover| cover.get("type"))).as_deref() {
        Some("url") => text(media_cover.and_then(|cover| cover.get("value")))
            .and_then(|url| normalize_media_url(&url, config)),
        Some("hash") => text(media_cover.and_then(|cover| cover.get("value")))
            .map(|hash| app_icon_url(config, &id, &hash, 256)),
        _ => None,
    };

    let image = media_cover_url
        .or(artwork)
        .or(cover)
        .or_else(|| icon.map(|hash| app_icon_url(config, &id, &hash, 256)));

    Some((id, BoardGameInfo { name, image }))
}

async fn resolve_applications(
    http: &HttpClient,
    config: &BackendConfig,
    user_id: Id<UserMarker>,
    widgets: &[BoardWidget],
    extras: &RequestExtras,
) -> HashMap<String, BoardApplication> {
    let mut ids: Vec<String> = widgets
        .iter()
        .filter_map(|widget| widget.application_id.clone())
        .collect();
    dedupe_strings(&mut ids);
    if ids.is_empty() {
        return HashMap::new();
    }

    let path = format!("/users/{user_id}/application-identities?with_profiles=true");
    let identities_value = get_json(http, "GET /users/:id/application-identities", &path, extras)
        .await
        .ok()
        .flatten()
        .unwrap_or(Value::Null);
    let identities = list_objects(Some(&identities_value), "identities")
        .into_iter()
        .filter_map(|object| text(object.get("application_id")).map(|id| (id, object)))
        .collect::<HashMap<_, _>>();

    let public_query = ids
        .iter()
        .map(|id| format!("application_ids={id}"))
        .collect::<Vec<_>>()
        .join("&");
    let public_value = get_json(
        http,
        "GET /applications/public",
        &format!("/applications/public?{public_query}"),
        extras,
    )
    .await
    .ok()
    .flatten()
    .unwrap_or(Value::Null);
    let public_apps = list_objects(Some(&public_value), "applications")
        .into_iter()
        .filter_map(|object| text(object.get("id")).map(|id| (id, object)))
        .collect::<HashMap<_, _>>();

    let mut result = HashMap::new();
    for id in ids {
        let path = format!("/applications/{id}/widget-configs");
        let configs_value = get_json(http, "GET /applications/:id/widget-configs", &path, extras)
            .await
            .ok()
            .flatten()
            .unwrap_or(Value::Null);
        let configs = list_objects(Some(&configs_value), "configs");
        let selected = select_config(&configs);
        result.insert(
            id.clone(),
            parse_application(
                &id,
                public_apps.get(&id).copied(),
                identities.get(&id).copied(),
                selected,
                config,
            ),
        );
    }

    result
}

fn list_objects<'a>(value: Option<&'a Value>, key: &str) -> Vec<&'a Map<String, Value>> {
    value
        .and_then(Value::as_object)
        .and_then(|object| object.get(key))
        .and_then(Value::as_array)
        .map(|array| array.iter().filter_map(Value::as_object).collect())
        .unwrap_or_default()
}

fn select_config<'a>(configs: &[&'a Map<String, Value>]) -> Option<&'a Map<String, Value>> {
    let mut first: Option<&Map<String, Value>> = None;
    let mut with_top: Option<&Map<String, Value>> = None;
    for config in configs {
        if first.is_none() {
            first = Some(*config);
        }
        let has_top = config
            .get("surfaces")
            .and_then(Value::as_object)
            .and_then(|surfaces| surfaces.get("widget_top"))
            .is_some();
        if has_top {
            if with_top.is_none() {
                with_top = Some(*config);
            }
            if text(config.get("status")).as_deref() == Some("published") {
                return Some(*config);
            }
        }
    }
    with_top.or(first)
}

fn parse_application(
    id: &str,
    application: Option<&Map<String, Value>>,
    identity: Option<&Map<String, Value>>,
    widget_config: Option<&Map<String, Value>>,
    config: &BackendConfig,
) -> BoardApplication {
    let app = application.cloned().or_else(|| {
        widget_config
            .and_then(|widget| widget.get("application"))
            .and_then(Value::as_object)
            .cloned()
    });
    let name = text(widget_config.and_then(|widget| widget.get("display_name")))
        .or_else(|| text(app.as_ref().and_then(|app| app.get("name"))))
        .unwrap_or_else(|| DEFAULT_APP_NAME.to_owned());
    let icon_hash = text(app.as_ref().and_then(|app| app.get("icon")))
        .or_else(|| text(app.as_ref().and_then(|app| app.get("icon_hash"))));
    let icon = icon_hash.map(|hash| app_icon_url(config, id, &hash, 64));

    let profile = identity
        .and_then(|identity| identity.get("profile"))
        .and_then(Value::as_object)
        .cloned()
        .or_else(|| {
            identity
                .and_then(|identity| identity.get("profiles"))
                .and_then(Value::as_array)
                .and_then(|profiles| profiles.first())
                .and_then(Value::as_object)
                .cloned()
        });
    let values = profile_values(profile.as_ref());

    let assets = widget_config
        .and_then(|widget| widget.get("resolved_assets"))
        .and_then(Value::as_array)
        .cloned();
    let assets = assets.as_deref();
    let surfaces = widget_config
        .and_then(|widget| widget.get("surfaces"))
        .and_then(Value::as_object);
    let top = surfaces
        .and_then(|surfaces| surfaces.get("widget_top"))
        .and_then(Value::as_object);
    let bottom = surfaces
        .and_then(|surfaces| surfaces.get("widget_bottom"))
        .and_then(Value::as_object);
    let top_components = top
        .and_then(|top| top.get("components"))
        .and_then(Value::as_object);

    let title = component_text(top_components, "title", "text", &values, assets, id, config)
        .or_else(|| text(profile.as_ref().and_then(|profile| profile.get("username"))));
    let image = component_image(top_components, "hero_image", &values, assets, id, config)
        .or_else(|| {
            component_image(
                top_components,
                "contained_image",
                &values,
                assets,
                id,
                config,
            )
        })
        .or_else(|| {
            values
                .get("featured_played_character_image")
                .and_then(|value| media_url(Some(value), config))
        });
    let subtitles = (1..=3)
        .filter_map(|index| {
            component_text(
                top_components,
                &format!("subtitle_{index}"),
                "text",
                &values,
                assets,
                id,
                config,
            )
        })
        .collect();

    let mut stats = Vec::new();
    let bottom_components = bottom
        .and_then(|bottom| bottom.get("components"))
        .and_then(Value::as_object);
    if let Some(bottom_components) = bottom_components {
        for index in 1..=20 {
            let key = format!("stat_{index}");
            let Some(component) = bottom_components.get(&key).and_then(Value::as_object) else {
                continue;
            };
            let fields = component.get("fields").and_then(Value::as_object);
            let value = resolve_field(
                fields
                    .and_then(|fields| fields.get("value"))
                    .and_then(Value::as_object),
                &values,
                assets,
                id,
                config,
                0,
            )
            .and_then(|resolved| resolved.text);
            let label = resolve_field(
                fields
                    .and_then(|fields| fields.get("label"))
                    .and_then(Value::as_object),
                &values,
                assets,
                id,
                config,
                0,
            )
            .and_then(|resolved| resolved.text);
            if let (Some(value), Some(label)) = (value, label) {
                stats.push(BoardStat { value, label });
            }
        }
    }

    BoardApplication {
        name,
        icon,
        title,
        image,
        subtitles,
        stats,
    }
}

fn component_text(
    components: Option<&Map<String, Value>>,
    name: &str,
    field: &str,
    values: &HashMap<String, Value>,
    assets: Option<&[Value]>,
    application_id: &str,
    config: &BackendConfig,
) -> Option<String> {
    let field = components
        .and_then(|components| components.get(name))
        .and_then(Value::as_object)
        .and_then(|component| component.get("fields"))
        .and_then(Value::as_object)
        .and_then(|fields| fields.get(field))
        .and_then(Value::as_object);
    resolve_field(field, values, assets, application_id, config, 0).and_then(|r| r.text)
}

fn component_image(
    components: Option<&Map<String, Value>>,
    name: &str,
    values: &HashMap<String, Value>,
    assets: Option<&[Value]>,
    application_id: &str,
    config: &BackendConfig,
) -> Option<String> {
    let field = components
        .and_then(|components| components.get(name))
        .and_then(Value::as_object)
        .and_then(|component| component.get("fields"))
        .and_then(Value::as_object)
        .and_then(|fields| fields.get("image"))
        .and_then(Value::as_object);
    resolve_field(field, values, assets, application_id, config, 0).and_then(|r| r.image)
}

#[derive(Clone, Debug, Default)]
struct Resolved {
    text: Option<String>,
    image: Option<String>,
}

fn resolve_field(
    field: Option<&Map<String, Value>>,
    values: &HashMap<String, Value>,
    assets: Option<&[Value]>,
    application_id: &str,
    config: &BackendConfig,
    depth: usize,
) -> Option<Resolved> {
    let field = field?;
    if depth > 3 {
        return None;
    }
    let key = text(field.get("value"));
    let resolved = match text(field.get("value_type")).as_deref() {
        Some("custom_string") => key.map(|text| Resolved {
            text: Some(text),
            ..Resolved::default()
        }),
        Some("data") => match key.as_ref().and_then(|key| values.get(key)) {
            Some(value @ Value::Object(_)) => media_url(Some(value), config).map(|url| Resolved {
                image: Some(url),
                ..Resolved::default()
            }),
            Some(Value::String(value)) => Some(Resolved {
                text: Some(value.clone()),
                ..Resolved::default()
            }),
            Some(Value::Number(value)) => Some(Resolved {
                text: Some(value.to_string()),
                ..Resolved::default()
            }),
            Some(Value::Bool(value)) => Some(Resolved {
                text: Some(value.to_string()),
                ..Resolved::default()
            }),
            _ => None,
        },
        Some("application_asset") => find_asset(assets, key.as_deref())
            .and_then(|asset| text(asset.get("asset_id")))
            .map(|asset_id| Resolved {
                image: Some(app_asset_url(config, application_id, &asset_id, 512)),
                ..Resolved::default()
            }),
        _ => None,
    };

    resolved.or_else(|| {
        resolve_field(
            field.get("fallback").and_then(Value::as_object),
            values,
            assets,
            application_id,
            config,
            depth + 1,
        )
    })
}

fn find_asset<'a>(
    assets: Option<&'a [Value]>,
    key: Option<&str>,
) -> Option<&'a Map<String, Value>> {
    let key = key?;
    assets?
        .iter()
        .filter_map(Value::as_object)
        .find(|asset| text(asset.get("key")).as_deref() == Some(key))
}

fn profile_values(profile: Option<&Map<String, Value>>) -> HashMap<String, Value> {
    let mut values = HashMap::new();
    let Some(profile) = profile else {
        return values;
    };
    if let Some(username) = text(profile.get("username")) {
        values.insert("username".to_owned(), Value::String(username));
    }
    let data = profile.get("data").and_then(Value::as_object);
    if let Some(primary) = data
        .and_then(|data| data.get("primary"))
        .and_then(Value::as_object)
    {
        for (key, value) in primary {
            if is_profile_value(value) {
                values.insert(key.clone(), value.clone());
            }
        }
    }
    if let Some(dynamic) = data
        .and_then(|data| data.get("dynamic"))
        .and_then(Value::as_array)
    {
        for field in dynamic {
            let Some(field) = field.as_object() else {
                continue;
            };
            let Some(name) = text(field.get("name")) else {
                continue;
            };
            let Some(value) = field.get("value") else {
                continue;
            };
            if is_profile_value(value) {
                values.insert(name, value.clone());
            }
        }
    }
    values
}

fn is_profile_value(value: &Value) -> bool {
    matches!(
        value,
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Object(_)
    )
}

fn media_url(value: Option<&Value>, config: &BackendConfig) -> Option<String> {
    let media = value?.as_object()?;
    let proxy = text(media.get("proxy_url")).and_then(|url| normalize_media_url(&url, config));
    proxy.or_else(|| text(media.get("url")).and_then(|url| normalize_media_url(&url, config)))
}

/// Accepts absolute HTTPS URLs and resolves relative ones against the configured CDN, as the
/// collectible asset parser does.
fn normalize_media_url(raw: &str, config: &BackendConfig) -> Option<String> {
    let raw = raw.trim();
    let url = if raw.starts_with("//") {
        format!("https:{raw}")
    } else if raw.starts_with('/') {
        format!("{}{}", config.cdn_base_url.trim_end_matches('/'), raw)
    } else {
        raw.to_owned()
    };
    url.starts_with("https://").then_some(url)
}

fn app_icon_url(config: &BackendConfig, application_id: &str, hash: &str, size: u32) -> String {
    format!(
        "{}/app-icons/{application_id}/{hash}.png?size={size}",
        config.cdn_base_url.trim_end_matches('/')
    )
}

fn app_asset_url(
    config: &BackendConfig,
    application_id: &str,
    asset_id: &str,
    size: u32,
) -> String {
    format!(
        "{}/app-assets/{application_id}/{asset_id}.webp?size={size}",
        config.cdn_base_url.trim_end_matches('/')
    )
}

fn text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.trim().is_empty() && value != "null" => Some(value.clone()),
        _ => None,
    }
}

fn text_or_null(value: &Value) -> Option<String> {
    text(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn config() -> BackendConfig {
        BackendConfig {
            rest_base_url: "https://example.invalid/api/v9".to_owned(),
            gateway_url: "wss://example.invalid/gateway".to_owned(),
            cdn_base_url: "https://cdn.example.test/".to_owned(),
        }
    }

    #[test]
    fn widgets_keep_known_types_and_skip_malformed_entries() {
        let profile = json!({
            "widgets": [
                { "data": { "type": "favorite_games", "games": [
                    { "game_id": "1", "comment": "nice", "tags": ["co-op", 7, "null", ""] },
                    { "comment": "no id" },
                    "not an object"
                ] } },
                { "data": { "type": "application", "application_id": "99" } },
                { "data": { "type": "unknown", "games": [{ "game_id": "2" }] } },
                { "data": { "type": "played_games", "games": [] } },
                { "data": { "type": "current_games", "games": [{ "game_id": "3" }] } },
                { "games": [{ "game_id": "4" }] }
            ]
        });

        let widgets = parse_widgets(profile.get("widgets"));
        assert_eq!(widgets.len(), 3);
        assert_eq!(widgets[0].widget_type, "favorite_games");
        assert_eq!(widgets[0].games.len(), 1);
        assert_eq!(widgets[0].games[0].game_id, "1");
        assert_eq!(widgets[0].games[0].comment.as_deref(), Some("nice"));
        assert_eq!(widgets[0].games[0].tags, vec!["co-op".to_owned()]);
        assert_eq!(widgets[1].application_id.as_deref(), Some("99"));
        assert!(widgets[1].games.is_empty());
        assert_eq!(widgets[2].widget_type, "current_games");
    }

    #[test]
    fn public_board_structures_round_trip_through_serde() {
        let board = ProfileBoard {
            widgets: vec![BoardWidget {
                widget_type: "favorite_games".to_owned(),
                games: vec![BoardGameEntry {
                    game_id: "1".to_owned(),
                    comment: None,
                    tags: vec!["cozy".to_owned()],
                }],
                application_id: None,
            }],
            games: HashMap::from([(
                "1".to_owned(),
                BoardGameInfo {
                    name: "A Game".to_owned(),
                    image: None,
                },
            )]),
            applications: HashMap::new(),
        };

        let json = serde_json::to_value(&board).unwrap();
        assert_eq!(json["widgets"][0]["type"], "favorite_games");
        let decoded: ProfileBoard = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, board);
        assert!(!decoded.is_empty());
        assert!(ProfileBoard::default().is_empty());
    }

    #[test]
    fn game_artwork_uses_configured_cdn_base() {
        let config = config();
        let game = json!({ "id": "42", "name": "42", "icon_hash": "abc" });
        let (_, info) = parse_game(game.as_object(), &config).unwrap();
        assert_eq!(info.name, "Unknown game");
        assert_eq!(
            info.image.as_deref(),
            Some("https://cdn.example.test/app-icons/42/abc.png?size=256")
        );

        let game = json!({
            "id": "7",
            "name": "Seven",
            "supplemental_game_data": {
                "artwork_urls": ["/art/7.png"],
                "name": "7"
            }
        });
        let (_, info) = parse_game(game.as_object(), &config).unwrap();
        assert_eq!(info.name, "Seven");
        assert_eq!(
            info.image.as_deref(),
            Some("https://cdn.example.test/art/7.png")
        );

        let game = json!({
            "id": "8",
            "media": { "cover": { "type": "hash", "value": "coverhash" } }
        });
        let (_, info) = parse_game(game.as_object(), &config).unwrap();
        assert_eq!(
            info.image.as_deref(),
            Some("https://cdn.example.test/app-icons/8/coverhash.png?size=256")
        );
    }

    #[test]
    fn widget_components_and_assets_resolve_against_configured_cdn() {
        let config = config();
        let app = json!({ "id": "99", "name": "Public App", "icon": "iconhash" });
        let identity = json!({
            "application_id": "99",
            "profile": {
                "username": "player",
                "data": {
                    "primary": {
                        "featured_played_character_image": { "url": "//media.example.test/hero.png" }
                    },
                    "dynamic": [{ "name": "score", "value": 1234 }]
                }
            }
        });
        let widget = json!({
            "status": "published",
            "display_name": "Widget Name",
            "resolved_assets": [{ "key": "hero", "asset_id": "asset1" }],
            "surfaces": {
                "widget_top": { "components": {
                    "title": { "fields": { "text": { "value_type": "data", "value": "username" } } },
                    "hero_image": { "fields": { "image": { "value_type": "application_asset", "value": "hero" } } },
                    "subtitle_1": { "fields": { "text": { "value_type": "custom_string", "value": "Live" } } }
                } },
                "widget_bottom": { "components": {
                    "stat_1": { "fields": {
                        "value": { "value_type": "data", "value": "score" },
                        "label": { "value_type": "custom_string", "value": "Score" }
                    } }
                } }
            }
        });

        let resolved = parse_application(
            "99",
            app.as_object(),
            identity.as_object(),
            widget.as_object(),
            &config,
        );
        assert_eq!(resolved.name, "Widget Name");
        assert_eq!(
            resolved.icon.as_deref(),
            Some("https://cdn.example.test/app-icons/99/iconhash.png?size=64")
        );
        assert_eq!(resolved.title.as_deref(), Some("player"));
        assert_eq!(
            resolved.image.as_deref(),
            Some("https://cdn.example.test/app-assets/99/asset1.webp?size=512")
        );
        assert_eq!(resolved.subtitles, vec!["Live".to_owned()]);
        assert_eq!(resolved.stats.len(), 1);
        assert_eq!(resolved.stats[0].value, "1234");
        assert_eq!(resolved.stats[0].label, "Score");

        let media = json!({ "proxy_url": "//media.example.test/clip.png" });
        assert_eq!(
            media_url(Some(&media), &config).as_deref(),
            Some("https://media.example.test/clip.png")
        );
        let media = json!({ "url": "/media/clip.png" });
        assert_eq!(
            media_url(Some(&media), &config).as_deref(),
            Some("https://cdn.example.test/media/clip.png")
        );
    }

    #[test]
    fn config_selection_prefers_a_published_top_surface() {
        let draft = json!({ "status": "draft" });
        let top = json!({ "status": "draft", "surfaces": { "widget_top": {} } });
        let published = json!({
            "status": "published",
            "surfaces": { "widget_top": {} }
        });
        let configs = vec![
            draft.as_object().unwrap(),
            top.as_object().unwrap(),
            published.as_object().unwrap(),
        ];
        let selected = select_config(&configs).unwrap();
        assert_eq!(text(selected.get("status")).as_deref(), Some("published"));
    }
}
