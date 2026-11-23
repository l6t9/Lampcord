use lampcord_model::{channel::Channel, guild::Guild, snowflake::Snowflake};
use serde_json::Value;

pub const ADMINISTRATOR: u64 = 1 << 3;
pub const VIEW_CHANNEL: u64 = 1 << 10;
pub const SEND_MESSAGES: u64 = 1 << 11;
pub const SEND_MESSAGES_IN_THREADS: u64 = 1 << 38;
const ALL_PERMISSIONS: u64 = u64::MAX;
const OVERWRITE_ROLE: u64 = 0;
const OVERWRITE_MEMBER: u64 = 1;

pub fn base_permissions(guild: &Guild, member_roles: &[Snowflake], user_id: Snowflake) -> u64 {
    if guild.owner_id == Some(user_id) {
        return ALL_PERMISSIONS;
    }
    let mut permissions = role_permissions(guild, guild.id);
    for role_id in member_roles {
        permissions |= role_permissions(guild, *role_id);
    }
    if permissions & ADMINISTRATOR != 0 {
        ALL_PERMISSIONS
    } else {
        permissions
    }
}

pub fn can_view_channel(
    guild: &Guild,
    channel: &Channel,
    member_roles: &[Snowflake],
    user_id: Snowflake,
) -> bool {
    if guild.roles.is_empty() {
        return true;
    }
    let base = base_permissions(guild, member_roles, user_id);
    if base & ADMINISTRATOR != 0 {
        return true;
    }
    let permissions = apply_overwrites(base, channel, guild.id, member_roles, user_id);
    permissions & VIEW_CHANNEL != 0
}

/// Whether the user can send messages in the channel. Threads use SEND_MESSAGES_IN_THREADS.
pub fn can_send_messages(
    guild: &Guild,
    channel: &Channel,
    member_roles: &[Snowflake],
    user_id: Snowflake,
) -> bool {
    if channel.guild_id.is_none() || guild.roles.is_empty() {
        return true;
    }
    if !can_view_channel(guild, channel, member_roles, user_id) {
        return false;
    }
    let base = base_permissions(guild, member_roles, user_id);
    if base & ADMINISTRATOR != 0 {
        return true;
    }
    let permissions = apply_overwrites(base, channel, guild.id, member_roles, user_id);
    let required = if channel.kind.is_thread() {
        SEND_MESSAGES_IN_THREADS
    } else {
        SEND_MESSAGES
    };
    permissions & required != 0
}

fn role_permissions(guild: &Guild, role_id: Snowflake) -> u64 {
    guild
        .roles
        .iter()
        .find(|role| role.id == role_id)
        .map(|role| parse_bits(&role.permissions))
        .unwrap_or(0)
}

fn apply_overwrites(
    base: u64,
    channel: &Channel,
    everyone_id: Snowflake,
    member_roles: &[Snowflake],
    user_id: Snowflake,
) -> u64 {
    let overwrites: Vec<Overwrite> = channel
        .permission_overwrites
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(parse_overwrite)
        .collect();

    let mut permissions = base;
    if let Some(everyone) = overwrites
        .iter()
        .find(|overwrite| overwrite.kind == OVERWRITE_ROLE && overwrite.id == everyone_id)
    {
        permissions = (permissions & !everyone.deny) | everyone.allow;
    }

    let (mut allow, mut deny) = (0u64, 0u64);
    for overwrite in overwrites.iter().filter(|overwrite| {
        overwrite.kind == OVERWRITE_ROLE
            && overwrite.id != everyone_id
            && member_roles.contains(&overwrite.id)
    }) {
        allow |= overwrite.allow;
        deny |= overwrite.deny;
    }
    permissions = (permissions & !deny) | allow;

    if let Some(member) = overwrites
        .iter()
        .find(|overwrite| overwrite.kind == OVERWRITE_MEMBER && overwrite.id == user_id)
    {
        permissions = (permissions & !member.deny) | member.allow;
    }
    permissions
}

struct Overwrite {
    id: Snowflake,
    kind: u64,
    allow: u64,
    deny: u64,
}

fn parse_overwrite(value: &Value) -> Option<Overwrite> {
    let id = value.get("id")?.as_str()?.parse::<u64>().ok()?;
    let kind = value.get("type")?.as_u64()?;
    Some(Overwrite {
        id: Snowflake::from(id),
        kind,
        allow: value
            .get("allow")
            .and_then(Value::as_str)
            .map(parse_bits)
            .unwrap_or(0),
        deny: value
            .get("deny")
            .and_then(Value::as_str)
            .map(parse_bits)
            .unwrap_or(0),
    })
}

fn parse_bits(value: &str) -> u64 {
    value.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn guild(owner: u64) -> Guild {
        serde_json::from_value(json!({
            "id": "1", "owner_id": owner.to_string(),
            "roles": [
                {"id": "1", "permissions": "1024"},
                {"id": "50", "permissions": "0"},
                {"id": "60", "permissions": "8"}
            ]
        }))
        .expect("guild")
    }

    fn channel(overwrites: Value) -> Channel {
        serde_json::from_value(json!({
            "id": "100", "guild_id": "1", "type": 0,
            "permission_overwrites": overwrites
        }))
        .expect("channel")
    }

    fn id(value: u64) -> Snowflake {
        Snowflake::from(value)
    }

    #[test]
    fn everyone_role_with_view_shows_a_plain_channel() {
        assert!(can_view_channel(&guild(9), &channel(json!([])), &[], id(2)));
    }

    #[test]
    fn owner_sees_everything() {
        let hidden = channel(json!([{"id": "1", "type": 0, "allow": "0", "deny": "1024"}]));
        assert!(can_view_channel(&guild(2), &hidden, &[], id(2)));
    }

    #[test]
    fn private_category_is_hidden_unless_a_role_allows_it() {
        let private = channel(json!([{"id": "1", "type": 0, "allow": "0", "deny": "1024"}]));
        assert!(!can_view_channel(&guild(9), &private, &[id(50)], id(2)));

        let granted = channel(json!([
            {"id": "1", "type": 0, "allow": "0", "deny": "1024"},
            {"id": "50", "type": 0, "allow": "1024", "deny": "0"}
        ]));
        assert!(can_view_channel(&guild(9), &granted, &[id(50)], id(2)));
        assert!(!can_view_channel(&guild(9), &granted, &[], id(2)));
    }

    #[test]
    fn member_overwrite_beats_role_overwrite() {
        let channel = channel(json!([
            {"id": "1", "type": 0, "allow": "0", "deny": "1024"},
            {"id": "50", "type": 0, "allow": "1024", "deny": "0"},
            {"id": "2", "type": 1, "allow": "0", "deny": "1024"}
        ]));
        assert!(!can_view_channel(&guild(9), &channel, &[id(50)], id(2)));
    }

    #[test]
    fn administrator_role_sees_private_channels() {
        let private = channel(json!([{"id": "1", "type": 0, "allow": "0", "deny": "1024"}]));
        assert!(can_view_channel(&guild(9), &private, &[id(60)], id(2)));
    }

    #[test]
    fn channels_stay_visible_when_roles_are_not_known_yet() {
        let guild: Guild =
            serde_json::from_value(json!({"id": "1", "owner_id": "9"})).expect("guild");
        let private = channel(json!([{"id": "1", "type": 0, "allow": "0", "deny": "1024"}]));
        assert!(can_view_channel(&guild, &private, &[], id(2)));
    }

    #[test]
    fn send_requires_the_send_messages_bit_after_overwrites() {
        let readonly = channel(json!([
            {"id": "1", "type": 0, "allow": "0", "deny": "2048"}
        ]));
        assert!(!can_send_messages(&guild(9), &readonly, &[], id(2)));

        let writable = channel(json!([
            {"id": "1", "type": 0, "allow": "2048", "deny": "0"}
        ]));
        assert!(can_send_messages(&guild(9), &writable, &[], id(2)));
    }

    #[test]
    fn dms_without_a_guild_are_always_visible() {
        let dm: Channel = serde_json::from_value(json!({"id": "900", "type": 1})).expect("dm");
        assert!(can_view_channel(&guild(9), &dm, &[], id(2)));
    }
}
