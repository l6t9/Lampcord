use crate::gateway::session;
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use percent_encoding::AsciiSet;
use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::utf8_percent_encode;

use super::super::{HttpClient, RequestExtras};

#[derive(Clone, Debug, Default)]
pub struct ResolvedInvite {
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub channel_type: Option<u64>,
}

pub async fn resolve_invite(
    http: &HttpClient,
    code: &str,
    input: &str,
) -> Result<ResolvedInvite, String> {
    let input = form_encode(input.trim());
    let path = format!(
        "/invites/{code}?inputValue={input}&with_counts=true&with_expiration=true&with_permissions=true"
    );
    let response = http
        .request("GET /invites/:code", wreq::Method::GET, &path)
        .await
        .map_err(|e| super::error_message(&e))?;
    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let id_of = |value: Option<&serde_json::Value>| {
        value.and_then(|v| v.get("id")).and_then(|id| match id {
            serde_json::Value::String(text) if !text.is_empty() => Some(text.clone()),
            serde_json::Value::Number(n) => n.as_u64().map(|n| n.to_string()),
            _ => None,
        })
    };
    Ok(ResolvedInvite {
        guild_id: id_of(body.get("guild")),
        channel_id: id_of(body.get("channel")),
        channel_type: body
            .get("channel")
            .and_then(|c| c.get("type"))
            .and_then(|t| t.as_u64()),
    })
}

#[derive(Clone, Debug)]
pub struct InvitePreview {
    pub guild_id: Id<GuildMarker>,
    pub name: String,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub verified: bool,
    pub partnered: bool,
    pub online: Option<u64>,
    pub members: Option<u64>,
    pub resolved: ResolvedInvite,
}

pub async fn fetch_preview(http: &HttpClient, code: &str) -> Result<InvitePreview, String> {
    let path = format!("/invites/{code}?with_counts=true&with_expiration=true");
    let response = http
        .request("GET /invites/:code", wreq::Method::GET, &path)
        .await
        .map_err(|e| super::error_message(&e))?;
    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let guild = body
        .get("guild")
        .ok_or_else(|| "the invite has no server".to_string())?;
    let text =
        |value: Option<&serde_json::Value>| value.and_then(|v| v.as_str()).map(str::to_owned);
    let guild_id: Id<GuildMarker> = text(guild.get("id"))
        .and_then(|id| id.parse().ok())
        .map(Id::new)
        .ok_or_else(|| "the invite's server has no id".to_string())?;
    let has_feature = |name: &str| {
        guild
            .get("features")
            .and_then(|f| f.as_array())
            .is_some_and(|features| features.iter().any(|f| f.as_str() == Some(name)))
    };
    let id_of = |value: Option<&serde_json::Value>| {
        value
            .and_then(|v| v.get("id"))
            .and_then(|id| id.as_str())
            .map(str::to_owned)
    };
    Ok(InvitePreview {
        guild_id,
        name: text(guild.get("name")).unwrap_or_default(),
        icon: text(guild.get("icon")),
        banner: text(guild.get("banner")).or_else(|| text(guild.get("splash"))),
        verified: has_feature("VERIFIED"),
        partnered: has_feature("PARTNERED"),
        online: body
            .get("approximate_presence_count")
            .and_then(|c| c.as_u64()),
        members: body
            .get("approximate_member_count")
            .and_then(|c| c.as_u64()),
        resolved: ResolvedInvite {
            guild_id: id_of(body.get("guild")),
            channel_id: id_of(body.get("channel")),
            channel_type: body
                .get("channel")
                .and_then(|c| c.get("type"))
                .and_then(|t| t.as_u64()),
        },
    })
}

pub fn codes_in(content: &str) -> Vec<String> {
    let mut codes: Vec<String> = Vec::new();
    for host in [
        "discord.gg/",
        "discord.com/invite/",
        "discordapp.com/invite/",
    ] {
        let mut rest = content;
        while let Some(at) = rest.find(host) {
            rest = &rest[at + host.len()..];
            let code: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !code.is_empty() && !codes.contains(&code) {
                codes.push(code);
            }
        }
    }
    codes
}

pub async fn accept_invite(
    http: &HttpClient,
    code: &str,
    invite: &ResolvedInvite,
) -> Result<Option<Id<GuildMarker>>, String> {
    let extras = RequestExtras {
        context: Some(join_context(invite)),
        ..RequestExtras::default()
    };
    let mut body = serde_json::Map::new();
    if let Some(session_id) = session::current_id() {
        body.insert("session_id".into(), session_id.into());
    }
    let response = http
        .request_with(
            "POST /invites/:code",
            wreq::Method::POST,
            &format!("/invites/{code}"),
            Some(&serde_json::Value::Object(body)),
            &extras,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(body
        .get("guild")
        .and_then(|g| g.get("id"))
        .and_then(|id| id.as_str()?.parse().ok())
        .map(Id::new))
}

fn join_context(invite: &ResolvedInvite) -> serde_json::Value {
    let mut context = serde_json::Map::new();
    context.insert("location".into(), "Join Guild".into());
    if let Some(guild_id) = &invite.guild_id {
        context.insert("location_guild_id".into(), guild_id.clone().into());
    }
    if let Some(channel_id) = &invite.channel_id {
        context.insert("location_channel_id".into(), channel_id.clone().into());
    }
    if let Some(channel_type) = invite.channel_type {
        context.insert("location_channel_type".into(), channel_type.into());
    }
    serde_json::Value::Object(context)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InviteSettings {
    pub max_age: u64,

    pub max_uses: u32,
    pub temporary: bool,
}

impl Default for InviteSettings {
    fn default() -> Self {
        Self {
            max_age: 604_800,
            max_uses: 0,
            temporary: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validate<'a> {
    Code(Option<&'a str>),
    Omit,
}

/// Discord's code for a server that already has as many invites as it may.
pub const TOO_MANY_INVITES: u64 = 30016;

/// What a create request came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Created {
    Code(String),
    /// The server is at its invite limit.
    TooMany,
}

pub async fn create_invite(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    settings: InviteSettings,
    validate: Validate<'_>,
    location: &str,
) -> Result<Created, String> {
    let response = match http
        .request_with(
            "POST /channels/:id/invites",
            wreq::Method::POST,
            &format!("/channels/{channel_id}/invites"),
            Some(&create_body(settings, validate)),
            &RequestExtras::context(location),
        )
        .await
    {
        Ok(response) => response,
        Err(super::super::RestError::Status { body, .. })
            if serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v.get("code")?.as_u64())
                == Some(TOO_MANY_INVITES) =>
        {
            return Ok(Created::TooMany);
        }
        Err(e) => return Err(super::error_message(&e)),
    };
    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    body.get("code")
        .and_then(|c| c.as_str())
        .map(|code| Created::Code(code.to_owned()))
        .ok_or_else(|| "the invite came back without a code".to_string())
}

fn create_body(settings: InviteSettings, validate: Validate<'_>) -> serde_json::Value {
    let mut body = serde_json::Map::new();
    if let Validate::Code(code) = validate {
        body.insert(
            "validate".into(),
            code.map_or(serde_json::Value::Null, Into::into),
        );
    }
    body.insert("max_age".into(), settings.max_age.into());
    body.insert("max_uses".into(), settings.max_uses.into());
    body.insert("target_type".into(), serde_json::Value::Null);
    body.insert("temporary".into(), settings.temporary.into());
    body.insert("flags".into(), 0.into());
    serde_json::Value::Object(body)
}

pub fn invite_url(code: &str) -> String {
    format!("https://discord.gg/{code}")
}

fn form_encode(text: &str) -> String {
    const KEEP: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'*')
        .remove(b'-')
        .remove(b'.')
        .remove(b'_');
    utf8_percent_encode(text, KEEP)
        .to_string()
        .replace("%20", "+")
}

pub fn invite_code(input: &str) -> Option<String> {
    let trimmed = input.trim().trim_end_matches('/');
    let last = trimmed.rsplit('/').next()?.split(['?', '#']).next()?;
    (!last.is_empty()
        && last
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
    .then(|| last.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        InviteSettings, ResolvedInvite, Validate, create_body, form_encode, invite_code,
        join_context,
    };

    #[test]
    fn the_create_body_matches_the_live_modal() {
        assert_eq!(
            create_body(InviteSettings::default(), Validate::Code(Some("CC4EeSeX"))).to_string(),
            r#"{"validate":"CC4EeSeX","max_age":604800,"max_uses":0,"target_type":null,"temporary":false,"flags":0}"#
        );
        assert_eq!(
            create_body(InviteSettings::default(), Validate::Code(None)).to_string(),
            r#"{"validate":null,"max_age":604800,"max_uses":0,"target_type":null,"temporary":false,"flags":0}"#
        );
        let settings = InviteSettings {
            max_age: 0,
            max_uses: 5,
            temporary: true,
        };
        assert_eq!(
            create_body(settings, Validate::Omit).to_string(),
            r#"{"max_age":0,"max_uses":5,"target_type":null,"temporary":true,"flags":0}"#
        );
    }

    #[test]
    fn the_join_context_matches_the_real_clients_request() {
        let invite = ResolvedInvite {
            guild_id: Some("1555424073230585856".into()),
            channel_id: Some("1555424073272532992".into()),
            channel_type: Some(0),
        };
        assert_eq!(
            join_context(&invite).to_string(),
            r#"{"location":"Join Guild","location_guild_id":"1555424073230585856","location_channel_id":"1555424073272532992","location_channel_type":0}"#
        );
        assert_eq!(
            join_context(&ResolvedInvite::default()).to_string(),
            r#"{"location":"Join Guild"}"#
        );
    }

    #[test]
    fn the_input_value_is_form_encoded_like_the_real_client_sends_it() {
        assert_eq!(
            form_encode("https://discord.gg/KMa2npYd"),
            "https%3A%2F%2Fdiscord.gg%2FKMa2npYd"
        );
        assert_eq!(form_encode("a b"), "a+b");
    }

    #[test]
    fn codes_are_pulled_out_of_every_invite_shape() {
        assert_eq!(invite_code("hTKzmak").as_deref(), Some("hTKzmak"));
        assert_eq!(
            invite_code("https://discord.gg/hTKzmak").as_deref(),
            Some("hTKzmak")
        );
        assert_eq!(
            invite_code("discord.gg/wumpus-friends/").as_deref(),
            Some("wumpus-friends")
        );
        assert_eq!(
            invite_code("https://discord.com/invite/abc123?event=1").as_deref(),
            Some("abc123")
        );
        assert_eq!(invite_code("   "), None);
        assert_eq!(invite_code("not a code!"), None);
    }
}
