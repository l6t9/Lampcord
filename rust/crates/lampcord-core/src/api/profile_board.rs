use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::ApiError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardGameEntry {
    pub game_id: String,
    pub comment: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardWidget {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default)]
    pub games: Vec<BoardGameEntry>,
    pub application_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardGameInfo {
    pub name: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardStat {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
const GAME_BATCH: usize = 25;
const APP_ICONS: &str = "https://cdn.discordapp.com/app-icons";
const WIDGET_TYPES: &[&str] = &[
    "favorite_games",
    "played_games",
    "current_games",
    "want_to_play_games",
    "application",
];

pub async fn load<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: &str,
) -> Result<Option<ProfileBoard>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let profile_req = config.request(
        HttpMethod::Get,
        &format!("/users/{user_id}/profile?with_mutual_guilds=false&with_mutual_friends=false"),
        config.standard_headers(),
        Body::None,
    );
    let profile_resp = client.send(profile_req).await?;
    if !(200..300).contains(&profile_resp.status) {
        return Ok(None);
    }
    let profile: Value = serde_json::from_str(&profile_resp.body)?;

    let widgets = parse_widgets(profile.get("widgets"));
    if widgets.is_empty() {
        return Ok(Some(ProfileBoard::default()));
    }

    let mut game_ids: Vec<String> = widgets
        .iter()
        .flat_map(|w| w.games.iter().map(|g| g.game_id.clone()))
        .collect();
    dedupe_strings(&mut game_ids);

    let games = resolve_games(client, config, &game_ids).await?;
    let applications = resolve_applications(client, config, user_id, &widgets).await?;

    Ok(Some(ProfileBoard {
        widgets,
        games,
        applications,
    }))
}

fn dedupe_strings(items: &mut Vec<String>) {
    let mut seen = HashSet::new();
    items.retain(|s| seen.insert(s.clone()));
}

async fn get_json_opt<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    path: &str,
) -> Result<Option<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(HttpMethod::Get, path, config.standard_headers(), Body::None);
    let resp = client.send(req).await?;
    if !(200..300).contains(&resp.status) {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&resp.body)?))
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
        let Some(wtype) = text(data.get("type")) else {
            continue;
        };
        if !WIDGET_TYPES.contains(&wtype.as_str()) {
            continue;
        }
        let games = data
            .get("games")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|game| {
                        let obj = game.as_object()?;
                        let id = text(obj.get("game_id"))?;
                        let tags = obj
                            .get("tags")
                            .and_then(Value::as_array)
                            .map(|tags| tags.iter().filter_map(text_or_null).collect())
                            .unwrap_or_default();
                        Some(BoardGameEntry {
                            game_id: id,
                            comment: text(obj.get("comment")),
                            tags,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let application_id = text(data.get("application_id"));
        if !games.is_empty() || application_id.is_some() {
            widgets.push(BoardWidget {
                widget_type: wtype,
                games,
                application_id,
            });
        }
    }
    widgets
}

async fn resolve_games<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    ids: &[String],
) -> Result<HashMap<String, BoardGameInfo>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut games: HashMap<String, BoardGameInfo> = HashMap::new();

    for batch in ids.chunks(GAME_BATCH) {
        let query = batch
            .iter()
            .map(|id| format!("game_ids={id}"))
            .collect::<Vec<_>>()
            .join("&");
        let path = format!("/games?{query}");
        if let Ok(Some(value)) = get_json_opt(client, config, &path).await {
            let array = match value {
                Value::Array(arr) => Some(arr),
                Value::Object(ref obj) => obj.get("games").and_then(Value::as_array).cloned(),
                _ => None,
            };
            if let Some(array) = array {
                for element in array {
                    if let Some((id, info)) = parse_game(element.as_object()) {
                        games.insert(id, info);
                    }
                }
            }
        }
    }

    for id in ids {
        if games.contains_key(id) {
            continue;
        }
        let path = format!("/games/{id}");
        if let Ok(Some(value)) = get_json_opt(client, config, &path).await {
            if let Some((game_id, info)) = parse_game(value.as_object()) {
                games.insert(game_id, info);
            }
        }
    }

    Ok(games)
}

fn parse_game(game: Option<&serde_json::Map<String, Value>>) -> Option<(String, BoardGameInfo)> {
    let game = game?;
    let id = text(game.get("id"))?;
    let supplemental = game
        .get("supplemental_game_data")
        .and_then(Value::as_object);
    let name = text(game.get("name"))
        .filter(|n| n != &id)
        .or_else(|| text(supplemental.and_then(|s| s.get("name"))).filter(|n| n != &id))
        .unwrap_or_else(|| UNKNOWN_GAME_NAME.to_owned());

    let artwork = supplemental
        .and_then(|s| s.get("artwork_urls"))
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(text_or_null);
    let cover = text(supplemental.and_then(|s| s.get("cover_image_url")));
    let icon =
        text(supplemental.and_then(|s| s.get("icon_hash"))).or_else(|| text(game.get("icon_hash")));
    let media_cover = game
        .get("media")
        .and_then(Value::as_object)
        .and_then(|m| m.get("cover"))
        .and_then(Value::as_object);
    let media_cover_url = match text(media_cover.and_then(|m| m.get("type"))).as_deref() {
        Some("url") => text(media_cover.and_then(|m| m.get("value"))),
        Some("hash") => text(media_cover.and_then(|m| m.get("value")))
            .map(|v| format!("{APP_ICONS}/{id}/{v}.png?size=256")),
        _ => None,
    };

    let image = if media_cover_url
        .as_deref()
        .is_some_and(|s| s.starts_with("https://"))
    {
        media_cover_url
    } else if artwork
        .as_deref()
        .is_some_and(|s| s.starts_with("https://"))
    {
        artwork
    } else if cover.as_deref().is_some_and(|s| s.starts_with("https://")) {
        cover
    } else if let Some(icon) = icon {
        Some(format!("{APP_ICONS}/{id}/{icon}.png?size=256"))
    } else {
        None
    };

    Some((id, BoardGameInfo { name, image }))
}

async fn resolve_applications<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: &str,
    widgets: &[BoardWidget],
) -> Result<HashMap<String, BoardApplication>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut ids: Vec<String> = widgets
        .iter()
        .filter_map(|w| w.application_id.clone())
        .collect();
    dedupe_strings(&mut ids);
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let identities_value = get_json_opt(
        client,
        config,
        &format!("/users/{user_id}/application-identities?with_profiles=true"),
    )
    .await?
    .unwrap_or(Value::Null);
    let identities = list_objects(Some(&identities_value), "identities")
        .into_iter()
        .filter_map(|obj| text(obj.get("application_id")).map(|id| (id, obj.clone())))
        .collect::<HashMap<_, _>>();

    let public_query = ids
        .iter()
        .map(|id| format!("application_ids={id}"))
        .collect::<Vec<_>>()
        .join("&");
    let public_value = get_json_opt(
        client,
        config,
        &format!("/applications/public?{public_query}"),
    )
    .await?
    .unwrap_or(Value::Null);
    let public_apps = list_objects(Some(&public_value), "applications")
        .into_iter()
        .filter_map(|obj| text(obj.get("id")).map(|id| (id, obj.clone())))
        .collect::<HashMap<_, _>>();

    let mut result = HashMap::new();
    for id in ids {
        let configs_value = get_json_opt(
            client,
            config,
            &format!("/applications/{id}/widget-configs"),
        )
        .await?
        .unwrap_or(Value::Null);
        let configs = list_objects(Some(&configs_value), "configs");
        let selected = select_config(&configs);
        let app_obj = public_apps.get(&id);
        let identity_obj = identities.get(&id);
        result.insert(
            id.clone(),
            parse_application(&id, app_obj, identity_obj, selected),
        );
    }

    Ok(result)
}

fn list_objects<'a>(
    value: Option<&'a Value>,
    key: &str,
) -> Vec<&'a serde_json::Map<String, Value>> {
    value
        .and_then(Value::as_object)
        .and_then(|obj| obj.get(key))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_object).collect())
        .unwrap_or_default()
}

fn select_config<'a>(
    configs: &'a [&'a serde_json::Map<String, Value>],
) -> Option<&'a serde_json::Map<String, Value>> {
    let mut first: Option<&serde_json::Map<String, Value>> = None;
    let mut with_top: Option<&serde_json::Map<String, Value>> = None;
    for config in configs {
        if first.is_none() {
            first = Some(*config);
        }
        let has_top = config
            .get("surfaces")
            .and_then(Value::as_object)
            .and_then(|s| s.get("widget_top"))
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
    application: Option<&serde_json::Map<String, Value>>,
    identity: Option<&serde_json::Map<String, Value>>,
    config: Option<&serde_json::Map<String, Value>>,
) -> BoardApplication {
    let app = application.cloned().or_else(|| {
        config
            .and_then(|c| c.get("application"))
            .and_then(Value::as_object)
            .cloned()
    });
    let name = text(config.and_then(|c| c.get("display_name")))
        .or_else(|| text(app.as_ref().and_then(|a| a.get("name"))))
        .unwrap_or_else(|| "Game Stats".to_owned());
    let icon_hash = text(app.as_ref().and_then(|a| a.get("icon")))
        .or_else(|| text(app.as_ref().and_then(|a| a.get("icon_hash"))));
    let icon = icon_hash.map(|h| format!("{APP_ICONS}/{id}/{h}.png?size=64"));

    let profile = identity
        .and_then(|i| i.get("profile"))
        .and_then(Value::as_object)
        .cloned()
        .or_else(|| {
            identity
                .and_then(|i| i.get("profiles"))
                .and_then(Value::as_array)
                .and_then(|arr| arr.first())
                .and_then(Value::as_object)
                .cloned()
        });
    let values = profile_values(profile.as_ref());

    let assets = config
        .and_then(|c| c.get("resolved_assets"))
        .and_then(Value::as_array)
        .cloned();
    let surfaces = config
        .and_then(|c| c.get("surfaces"))
        .and_then(Value::as_object);
    let top = surfaces
        .and_then(|s| s.get("widget_top"))
        .and_then(Value::as_object);
    let bottom = surfaces
        .and_then(|s| s.get("widget_bottom"))
        .and_then(Value::as_object);
    let top_components = top
        .and_then(|t| t.get("components"))
        .and_then(Value::as_object);

    let title = component_text(
        top_components,
        "title",
        "text",
        &values,
        assets.as_ref(),
        id,
    )
    .or_else(|| text(profile.as_ref().and_then(|p| p.get("username"))));
    let image = component_image(top_components, "hero_image", &values, assets.as_ref(), id)
        .or_else(|| {
            component_image(
                top_components,
                "contained_image",
                &values,
                assets.as_ref(),
                id,
            )
        })
        .or_else(|| media_url(values.get("featured_played_character_image")));
    let subtitles = (1..=3)
        .filter_map(|i| {
            component_text(
                top_components,
                &format!("subtitle_{i}"),
                "text",
                &values,
                assets.as_ref(),
                id,
            )
        })
        .collect::<Vec<_>>();

    let mut stats = Vec::new();
    let bottom_components = bottom
        .and_then(|b| b.get("components"))
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
                    .and_then(|f| f.get("value"))
                    .and_then(Value::as_object),
                &values,
                assets.as_ref(),
                id,
                0,
            )
            .and_then(|r| r.text);
            let label = resolve_field(
                fields
                    .and_then(|f| f.get("label"))
                    .and_then(Value::as_object),
                &values,
                assets.as_ref(),
                id,
                0,
            )
            .and_then(|r| r.text);
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
    components: Option<&serde_json::Map<String, Value>>,
    name: &str,
    field: &str,
    values: &HashMap<String, Value>,
    assets: Option<&Vec<Value>>,
    application_id: &str,
) -> Option<String> {
    let field_obj = components
        .and_then(|c| c.get(name))
        .and_then(Value::as_object)
        .and_then(|c| c.get("fields"))
        .and_then(Value::as_object)
        .and_then(|f| f.get(field))
        .and_then(Value::as_object);
    resolve_field(field_obj, values, assets, application_id, 0).and_then(|r| r.text)
}

fn component_image(
    components: Option<&serde_json::Map<String, Value>>,
    name: &str,
    values: &HashMap<String, Value>,
    assets: Option<&Vec<Value>>,
    application_id: &str,
) -> Option<String> {
    let field_obj = components
        .and_then(|c| c.get(name))
        .and_then(Value::as_object)
        .and_then(|c| c.get("fields"))
        .and_then(Value::as_object)
        .and_then(|f| f.get("image"))
        .and_then(Value::as_object);
    resolve_field(field_obj, values, assets, application_id, 0).and_then(|r| r.image)
}

#[derive(Debug, Clone, Default)]
struct Resolved {
    text: Option<String>,
    image: Option<String>,
}

fn resolve_field(
    field: Option<&serde_json::Map<String, Value>>,
    values: &HashMap<String, Value>,
    assets: Option<&Vec<Value>>,
    application_id: &str,
    depth: usize,
) -> Option<Resolved> {
    let field = field?;
    if depth > 3 {
        return None;
    }
    let key = text(field.get("value"));
    let resolved = match text(field.get("value_type")).as_deref() {
        Some("custom_string") => key.map(|s| Resolved {
            text: Some(s),
            image: None,
        }),
        Some("data") => {
            let data = key.as_ref().and_then(|k| values.get(k));
            match data {
                Some(Value::Object(obj)) => {
                    media_url(Some(&Value::Object(obj.clone()))).map(|u| Resolved {
                        text: None,
                        image: Some(u),
                    })
                }
                Some(Value::String(s)) => Some(Resolved {
                    text: Some(s.clone()),
                    image: None,
                }),
                Some(Value::Number(n)) => Some(Resolved {
                    text: Some(n.to_string()),
                    image: None,
                }),
                Some(Value::Bool(b)) => Some(Resolved {
                    text: Some(b.to_string()),
                    image: None,
                }),
                _ => None,
            }
        }
        Some("application_asset") => {
            let asset_id = find_asset(assets, key.as_deref()).and_then(|a| text(a.get("asset_id")));
            asset_id.map(|asset_id| Resolved {
                text: None,
                image: Some(format!(
                    "https://cdn.discordapp.com/app-assets/{application_id}/{asset_id}.webp?size=512"
                )),
            })
        }
        _ => None,
    };

    if resolved.is_some() {
        resolved
    } else {
        resolve_field(
            field.get("fallback").and_then(Value::as_object),
            values,
            assets,
            application_id,
            depth + 1,
        )
    }
}

fn find_asset<'a>(
    assets: Option<&'a Vec<Value>>,
    key: Option<&str>,
) -> Option<&'a serde_json::Map<String, Value>> {
    let key = key?;
    assets?
        .iter()
        .filter_map(Value::as_object)
        .find(|a| text(a.get("key")).as_deref() == Some(key))
}

fn profile_values(profile: Option<&serde_json::Map<String, Value>>) -> HashMap<String, Value> {
    let mut values = HashMap::new();
    let Some(profile) = profile else {
        return values;
    };
    if let Some(username) = text(profile.get("username")) {
        values.insert("username".to_owned(), Value::String(username));
    }
    let data = profile.get("data").and_then(Value::as_object);
    if let Some(primary) = data
        .and_then(|d| d.get("primary"))
        .and_then(Value::as_object)
    {
        for (k, v) in primary {
            if v.is_string() || v.is_number() || v.is_boolean() || v.is_object() {
                values.insert(k.clone(), v.clone());
            }
        }
    }
    if let Some(dynamic) = data
        .and_then(|d| d.get("dynamic"))
        .and_then(Value::as_array)
    {
        for field in dynamic {
            let Some(obj) = field.as_object() else {
                continue;
            };
            let Some(name) = text(obj.get("name")) else {
                continue;
            };
            let Some(value) = obj.get("value") else {
                continue;
            };
            if value.is_string() || value.is_number() || value.is_boolean() || value.is_object() {
                values.insert(name, value.clone());
            }
        }
    }
    values
}

fn media_url(value: Option<&Value>) -> Option<String> {
    let media = value?.as_object()?;
    if let Some(proxy) = text(media.get("proxy_url")) {
        if proxy.starts_with("https://") {
            return Some(proxy);
        }
    }
    text(media.get("url")).filter(|u| u.starts_with("https://"))
}

fn text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::Null => None,
        Value::String(s) => {
            if s.is_empty() || s == "null" {
                None
            } else {
                Some(s.clone())
            }
        }
        _ => None,
    }
}

fn text_or_null(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.is_empty() && s != "null" => Some(s.clone()),
        _ => None,
    }
}
