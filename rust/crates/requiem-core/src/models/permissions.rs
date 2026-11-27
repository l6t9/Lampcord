use std::collections::HashMap;
use std::ops;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::id::{
    Id,
    marker::{GuildMarker, RoleMarker, UserMarker},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Permissions(pub u64);

macro_rules! permission_bits {
    ($($name:ident = $bit:expr,)*) => {
        impl Permissions {
            $(pub const $name: Permissions = Permissions(1 << $bit);)*

            pub const ALL: Permissions = Permissions(0 $(| (1 << $bit))*);

            const NAMED: &'static [(&'static str, u64)] = &[$((stringify!($name), 1u64 << $bit)),*];
        }
    };
}

permission_bits! {
    CREATE_INSTANT_INVITE = 0,
    KICK_MEMBERS = 1,
    BAN_MEMBERS = 2,
    ADMINISTRATOR = 3,
    MANAGE_CHANNELS = 4,
    MANAGE_GUILD = 5,
    ADD_REACTIONS = 6,
    VIEW_AUDIT_LOG = 7,
    PRIORITY_SPEAKER = 8,
    STREAM = 9,
    VIEW_CHANNEL = 10,
    SEND_MESSAGES = 11,
    SEND_TTS_MESSAGES = 12,
    MANAGE_MESSAGES = 13,
    EMBED_LINKS = 14,
    ATTACH_FILES = 15,
    READ_MESSAGE_HISTORY = 16,
    MENTION_EVERYONE = 17,
    USE_EXTERNAL_EMOJIS = 18,
    VIEW_GUILD_INSIGHTS = 19,
    CONNECT = 20,
    SPEAK = 21,
    MUTE_MEMBERS = 22,
    DEAFEN_MEMBERS = 23,
    MOVE_MEMBERS = 24,
    USE_VAD = 25,
    CHANGE_NICKNAME = 26,
    MANAGE_NICKNAMES = 27,
    MANAGE_ROLES = 28,
    MANAGE_WEBHOOKS = 29,
    MANAGE_GUILD_EXPRESSIONS = 30,
    USE_APPLICATION_COMMANDS = 31,
    REQUEST_TO_SPEAK = 32,
    MANAGE_EVENTS = 33,
    MANAGE_THREADS = 34,
    CREATE_PUBLIC_THREADS = 35,
    CREATE_PRIVATE_THREADS = 36,
    USE_EXTERNAL_STICKERS = 37,
    SEND_MESSAGES_IN_THREADS = 38,
    USE_EMBEDDED_ACTIVITIES = 39,
    MODERATE_MEMBERS = 40,
    VIEW_CREATOR_MONETIZATION_ANALYTICS = 41,
    USE_SOUNDBOARD = 42,
    CREATE_GUILD_EXPRESSIONS = 43,
    CREATE_EVENTS = 44,
    USE_EXTERNAL_SOUNDS = 45,
    SEND_VOICE_MESSAGES = 46,
    SET_VOICE_CHANNEL_STATUS = 48,
    SEND_POLLS = 49,
    USE_EXTERNAL_APPS = 50,
    PIN_MESSAGES = 51,
    BYPASS_SLOWMODE = 52,
}

impl Permissions {
    pub const NONE: Permissions = Permissions(0);

    pub fn named() -> Vec<(String, Permissions)> {
        Self::NAMED
            .iter()
            .map(|(name, bit)| {
                let mut parts = name.split('_').map(str::to_lowercase);
                let mut camel = parts.next().unwrap_or_default();
                for part in parts {
                    let mut chars = part.chars();
                    if let Some(first) = chars.next() {
                        camel.extend(first.to_uppercase());
                        camel.push_str(chars.as_str());
                    }
                }
                (camel, Permissions(*bit))
            })
            .collect()
    }

    pub fn names(self) -> Vec<String> {
        Self::NAMED
            .iter()
            .filter(|(_, bit)| self.0 & bit != 0)
            .map(|(name, _)| name.to_lowercase().replace('_', "-"))
            .collect()
    }

    pub const fn contains(self, other: Permissions) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Permissions) -> Permissions {
        Permissions(self.0 | other.0)
    }

    pub const fn without(self, other: Permissions) -> Permissions {
        Permissions(self.0 & !other.0)
    }
}

impl ops::BitOr for Permissions {
    type Output = Permissions;
    fn bitor(self, rhs: Permissions) -> Permissions {
        self.union(rhs)
    }
}

fn serialize_bits<S: Serializer>(bits: &Permissions, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&bits.0.to_string())
}

fn serialize_raw_id<S: Serializer>(id: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&id.to_string())
}

pub fn deserialize_bits<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Permissions, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Wire {
        Text(String),
        Number(u64),
    }
    Ok(match Wire::deserialize(deserializer)? {
        Wire::Text(text) => Permissions(text.parse().unwrap_or(0)),
        Wire::Number(number) => Permissions(number),
    })
}

fn deserialize_raw_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Wire {
        Text(String),
        Number(u64),
    }
    Ok(match Wire::deserialize(deserializer)? {
        Wire::Text(text) => text.parse().unwrap_or(0),
        Wire::Number(number) => number,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverwriteKind {
    Role,
    Member,
}

impl Serialize for OverwriteKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(match self {
            OverwriteKind::Role => 0,
            OverwriteKind::Member => 1,
        })
    }
}

impl<'de> Deserialize<'de> for OverwriteKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match deserialize_raw_id(deserializer)? {
            1 => OverwriteKind::Member,
            _ => OverwriteKind::Role,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PermissionOverwrite {
    #[serde(
        deserialize_with = "deserialize_raw_id",
        serialize_with = "serialize_raw_id"
    )]
    pub id: u64,
    #[serde(rename = "type")]
    pub kind: OverwriteKind,
    #[serde(
        default,
        deserialize_with = "deserialize_bits",
        serialize_with = "serialize_bits"
    )]
    pub allow: Permissions,
    #[serde(
        default,
        deserialize_with = "deserialize_bits",
        serialize_with = "serialize_bits"
    )]
    pub deny: Permissions,
}

pub struct MemberContext<'a> {
    pub guild_id: Id<GuildMarker>,
    pub owner_id: Option<Id<UserMarker>>,
    pub user_id: Id<UserMarker>,

    pub roles: &'a HashMap<Id<RoleMarker>, Permissions>,

    pub member_roles: &'a [Id<RoleMarker>],

    pub timed_out: bool,
}

pub const TIMED_OUT_ALLOWED: Permissions =
    Permissions::VIEW_CHANNEL.union(Permissions::READ_MESSAGE_HISTORY);

pub const fn is_voice_kind(kind: u8) -> bool {
    matches!(kind, 2 | 13)
}

pub const fn is_thread_kind(kind: u8) -> bool {
    matches!(kind, 10..=12)
}

fn raw_base(ctx: &MemberContext) -> Permissions {
    if ctx.owner_id == Some(ctx.user_id) {
        return Permissions::ALL;
    }
    let everyone = Id::<RoleMarker>::new(ctx.guild_id.get());
    let mut bits = ctx.roles.get(&everyone).copied().unwrap_or_default();
    for role in ctx.member_roles {
        bits = bits | ctx.roles.get(role).copied().unwrap_or_default();
    }
    if bits.contains(Permissions::ADMINISTRATOR) {
        Permissions::ALL
    } else {
        bits
    }
}

pub fn compute_base(ctx: &MemberContext) -> Permissions {
    let bits = raw_base(ctx);
    if ctx.timed_out && !bits.contains(Permissions::ADMINISTRATOR) {
        return Permissions(bits.0 & TIMED_OUT_ALLOWED.0);
    }
    bits
}

pub fn compute_channel(ctx: &MemberContext, overwrites: &[PermissionOverwrite]) -> Permissions {
    compute_channel_of_kind(ctx, overwrites, 0)
}

pub fn compute_channel_of_kind(
    ctx: &MemberContext,
    overwrites: &[PermissionOverwrite],
    kind: u8,
) -> Permissions {
    let base = raw_base(ctx);
    if base.contains(Permissions::ADMINISTRATOR) {
        return Permissions::ALL;
    }
    let mut bits = base;
    let apply = |bits: Permissions, overwrite: &PermissionOverwrite| {
        bits.without(overwrite.deny).union(overwrite.allow)
    };

    if let Some(everyone) = overwrites
        .iter()
        .find(|o| o.kind == OverwriteKind::Role && o.id == ctx.guild_id.get())
    {
        bits = apply(bits, everyone);
    }
    let (mut allow, mut deny) = (Permissions::NONE, Permissions::NONE);
    for overwrite in overwrites.iter().filter(|o| {
        o.kind == OverwriteKind::Role && ctx.member_roles.iter().any(|r| r.get() == o.id)
    }) {
        allow = allow | overwrite.allow;
        deny = deny | overwrite.deny;
    }
    bits = bits.without(deny).union(allow);
    if let Some(member) = overwrites
        .iter()
        .find(|o| o.kind == OverwriteKind::Member && o.id == ctx.user_id.get())
    {
        bits = apply(bits, member);
    }

    if ctx.timed_out {
        bits = Permissions(bits.0 & TIMED_OUT_ALLOWED.0);
    }

    if !bits.contains(Permissions::VIEW_CHANNEL) {
        return Permissions::NONE;
    }

    if is_voice_kind(kind) && !bits.contains(Permissions::CONNECT) {
        return Permissions::VIEW_CHANNEL;
    }
    if is_thread_kind(kind) {
        bits = if bits.contains(Permissions::SEND_MESSAGES_IN_THREADS) {
            bits.union(Permissions::SEND_MESSAGES)
        } else {
            bits.without(Permissions::SEND_MESSAGES)
        };
    }
    if !bits.contains(Permissions::SEND_MESSAGES) {
        bits = bits.without(
            Permissions::SEND_TTS_MESSAGES
                | Permissions::MENTION_EVERYONE
                | Permissions::ATTACH_FILES
                | Permissions::EMBED_LINKS,
        );
    }
    bits
}

pub fn everyone_can(
    roles: &HashMap<Id<RoleMarker>, Permissions>,
    guild_id: Id<GuildMarker>,
    overwrites: &[PermissionOverwrite],
    permission: Permissions,
) -> bool {
    let everyone = Id::<RoleMarker>::new(guild_id.get());
    let mut bits = roles.get(&everyone).copied().unwrap_or_default();
    if bits.contains(Permissions::ADMINISTRATOR) {
        return true;
    }
    if let Some(overwrite) = overwrites
        .iter()
        .find(|o| o.kind == OverwriteKind::Role && o.id == guild_id.get())
    {
        bits = bits.without(overwrite.deny).union(overwrite.allow);
    }
    bits.contains(permission)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUILD: u64 = 100;
    const ME: u64 = 7;

    fn roles(entries: &[(u64, Permissions)]) -> HashMap<Id<RoleMarker>, Permissions> {
        entries
            .iter()
            .map(|(id, bits)| (Id::new(*id), *bits))
            .collect()
    }

    fn ctx<'a>(
        roles: &'a HashMap<Id<RoleMarker>, Permissions>,
        member_roles: &'a [Id<RoleMarker>],
        owner: Option<u64>,
    ) -> MemberContext<'a> {
        MemberContext {
            guild_id: Id::new(GUILD),
            owner_id: owner.map(Id::new),
            user_id: Id::new(ME),
            roles,
            member_roles,
            timed_out: false,
        }
    }

    fn role_overwrite(id: u64, allow: Permissions, deny: Permissions) -> PermissionOverwrite {
        PermissionOverwrite {
            id,
            kind: OverwriteKind::Role,
            allow,
            deny,
        }
    }

    fn defaults() -> Permissions {
        Permissions::VIEW_CHANNEL
            | Permissions::SEND_MESSAGES
            | Permissions::ATTACH_FILES
            | Permissions::ADD_REACTIONS
    }

    #[test]
    fn the_live_everyone_role_matches_the_mooncord_test_guild() {
        let everyone = Permissions(70634561);
        assert!(everyone.contains(
            Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY
        ));
        assert!(!everyone.contains(Permissions::MANAGE_CHANNELS));
    }

    #[test]
    fn everyone_deny_hides_a_channel_and_a_role_allow_brings_it_back() {
        let roles = roles(&[(GUILD, defaults()), (200, Permissions::NONE)]);
        let overwrites = [
            role_overwrite(GUILD, Permissions::NONE, Permissions::VIEW_CHANNEL),
            role_overwrite(200, Permissions::VIEW_CHANNEL, Permissions::NONE),
        ];
        let without_role = compute_channel(&ctx(&roles, &[], None), &overwrites);
        assert!(!without_role.contains(Permissions::VIEW_CHANNEL));
        assert_eq!(
            without_role,
            Permissions::NONE,
            "a hidden channel grants nothing"
        );
        let held = [Id::new(200)];
        assert!(
            compute_channel(&ctx(&roles, &held, None), &overwrites)
                .contains(Permissions::VIEW_CHANNEL)
        );
    }

    #[test]
    fn a_send_deny_also_drops_the_send_extras() {
        let roles = roles(&[(GUILD, defaults())]);
        let overwrites = [role_overwrite(
            GUILD,
            Permissions::NONE,
            Permissions::SEND_MESSAGES,
        )];
        let bits = compute_channel(&ctx(&roles, &[], None), &overwrites);
        assert!(bits.contains(Permissions::VIEW_CHANNEL));
        assert!(!bits.contains(Permissions::SEND_MESSAGES));
        assert!(!bits.contains(Permissions::ATTACH_FILES));
        assert!(bits.contains(Permissions::ADD_REACTIONS));
    }

    #[test]
    fn a_member_overwrite_beats_role_overwrites() {
        let roles = roles(&[(GUILD, defaults()), (200, Permissions::NONE)]);
        let held = [Id::new(200)];
        let overwrites = [
            role_overwrite(200, Permissions::NONE, Permissions::SEND_MESSAGES),
            PermissionOverwrite {
                id: ME,
                kind: OverwriteKind::Member,
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::NONE,
            },
        ];
        assert!(
            compute_channel(&ctx(&roles, &held, None), &overwrites)
                .contains(Permissions::SEND_MESSAGES)
        );
    }

    #[test]
    fn role_allow_wins_over_another_roles_deny() {
        let roles = roles(&[
            (GUILD, defaults()),
            (200, Permissions::NONE),
            (201, Permissions::NONE),
        ]);
        let held = [Id::new(200), Id::new(201)];
        let overwrites = [
            role_overwrite(200, Permissions::NONE, Permissions::SEND_MESSAGES),
            role_overwrite(201, Permissions::SEND_MESSAGES, Permissions::NONE),
        ];
        assert!(
            compute_channel(&ctx(&roles, &held, None), &overwrites)
                .contains(Permissions::SEND_MESSAGES)
        );
    }

    #[test]
    fn owner_and_administrators_bypass_everything() {
        let overwrites = [role_overwrite(
            GUILD,
            Permissions::NONE,
            Permissions::VIEW_CHANNEL,
        )];
        let plain = roles(&[(GUILD, defaults())]);
        assert_eq!(
            compute_channel(&ctx(&plain, &[], Some(ME)), &overwrites),
            Permissions::ALL
        );
        let admin = roles(&[(GUILD, defaults()), (200, Permissions::ADMINISTRATOR)]);
        let held = [Id::new(200)];
        assert_eq!(
            compute_channel(&ctx(&admin, &held, None), &overwrites),
            Permissions::ALL
        );
    }

    #[test]
    fn everyone_can_reads_only_the_everyone_overwrite() {
        let roles = roles(&[(GUILD, defaults()), (200, Permissions::NONE)]);
        let overwrites = [
            role_overwrite(GUILD, Permissions::NONE, Permissions::VIEW_CHANNEL),
            role_overwrite(200, Permissions::VIEW_CHANNEL, Permissions::NONE),
        ];
        assert!(!everyone_can(
            &roles,
            Id::new(GUILD),
            &overwrites,
            Permissions::VIEW_CHANNEL
        ));
        assert!(everyone_can(
            &roles,
            Id::new(GUILD),
            &[],
            Permissions::VIEW_CHANNEL
        ));
    }

    #[test]
    fn overwrites_decode_from_the_wire_shape() {
        let wire = r#"[{"id":"1527668330054295552","type":0,"allow":"0","deny":"2048"},{"id":"5","type":1,"allow":"1024","deny":"0"}]"#;
        let decoded: Vec<PermissionOverwrite> = serde_json::from_str(wire).unwrap();
        assert_eq!(decoded[0].id, 1527668330054295552);
        assert_eq!(decoded[0].deny, Permissions::SEND_MESSAGES);
        assert_eq!(decoded[1].kind, OverwriteKind::Member);
        assert_eq!(decoded[1].allow, Permissions::VIEW_CHANNEL);
    }

    #[test]
    fn a_timeout_masks_to_view_and_history_even_against_allows() {
        let roles = roles(&[(
            GUILD,
            defaults() | Permissions::READ_MESSAGE_HISTORY | Permissions::CONNECT,
        )]);
        let overwrites = [role_overwrite(
            GUILD,
            Permissions::MANAGE_MESSAGES,
            Permissions::NONE,
        )];
        let timed_out = MemberContext {
            timed_out: true,
            ..ctx(&roles, &[], None)
        };
        assert_eq!(
            compute_base(&timed_out),
            Permissions::VIEW_CHANNEL | Permissions::READ_MESSAGE_HISTORY
        );
        assert_eq!(
            compute_channel(&timed_out, &overwrites),
            Permissions::VIEW_CHANNEL | Permissions::READ_MESSAGE_HISTORY
        );

        assert_eq!(
            compute_channel_of_kind(&timed_out, &overwrites, 2),
            Permissions::VIEW_CHANNEL
        );

        let hidden = [role_overwrite(
            GUILD,
            Permissions::NONE,
            Permissions::VIEW_CHANNEL,
        )];
        assert_eq!(compute_channel(&timed_out, &hidden), Permissions::NONE);

        assert!(
            compute_channel(&ctx(&roles, &[], None), &overwrites)
                .contains(Permissions::SEND_MESSAGES | Permissions::MANAGE_MESSAGES)
        );
    }

    #[test]
    fn owners_and_administrators_are_exempt_from_timeouts() {
        let plain = roles(&[(GUILD, defaults())]);
        let owner = MemberContext {
            timed_out: true,
            ..ctx(&plain, &[], Some(ME))
        };
        assert_eq!(compute_base(&owner), Permissions::ALL);
        assert_eq!(compute_channel(&owner, &[]), Permissions::ALL);
        let admin = roles(&[(GUILD, defaults()), (200, Permissions::ADMINISTRATOR)]);
        let held = [Id::new(200)];
        let admin_ctx = MemberContext {
            timed_out: true,
            ..ctx(&admin, &held, None)
        };
        assert_eq!(compute_base(&admin_ctx), Permissions::ALL);
        assert_eq!(
            compute_channel_of_kind(&admin_ctx, &[], 2),
            Permissions::ALL
        );
    }

    #[test]
    fn a_voice_channel_without_connect_is_only_listed() {
        let roles = roles(&[(
            GUILD,
            defaults() | Permissions::CONNECT | Permissions::SPEAK,
        )]);
        let no_connect = [role_overwrite(
            GUILD,
            Permissions::NONE,
            Permissions::CONNECT,
        )];
        for kind in [2, 13] {
            assert_eq!(
                compute_channel_of_kind(&ctx(&roles, &[], None), &no_connect, kind),
                Permissions::VIEW_CHANNEL,
                "kind {kind}"
            );
            let bits = compute_channel_of_kind(&ctx(&roles, &[], None), &[], kind);
            assert!(
                bits.contains(
                    Permissions::CONNECT | Permissions::SPEAK | Permissions::SEND_MESSAGES
                )
            );
        }

        assert!(
            compute_channel_of_kind(&ctx(&roles, &[], None), &no_connect, 0)
                .contains(Permissions::SEND_MESSAGES)
        );
    }

    #[test]
    fn a_threads_send_bit_is_send_messages_in_threads() {
        let roles = roles(&[(GUILD, defaults())]);
        let bits = compute_channel_of_kind(&ctx(&roles, &[], None), &[], 11);
        assert!(
            !bits.contains(Permissions::SEND_MESSAGES) && !bits.contains(Permissions::ATTACH_FILES)
        );
        let threads = [role_overwrite(
            GUILD,
            Permissions::SEND_MESSAGES_IN_THREADS,
            Permissions::SEND_MESSAGES,
        )];
        let bits = compute_channel_of_kind(&ctx(&roles, &[], None), &threads, 11);
        assert!(bits.contains(Permissions::SEND_MESSAGES | Permissions::ATTACH_FILES));
    }
}
