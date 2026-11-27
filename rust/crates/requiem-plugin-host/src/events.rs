pub const EVENT_KINDS: &[(&str, Option<&str>)] = &[
    ("message-create", Some("messages.read")),
    ("message-update", Some("messages.read")),
    ("message-delete", Some("messages.read")),
    ("reaction-add", Some("messages.read")),
    ("reaction-remove", Some("messages.read")),
    ("typing-start", Some("presence.read")),
    ("presence-update", Some("presence.read")),
    ("voice-state-update", Some("voice.read")),
    ("call-ring", Some("voice.read")),
    ("relationship-add", Some("relationships.read")),
    ("relationship-remove", Some("relationships.read")),
    ("guild-join", Some("guilds.read")),
    ("guild-leave", Some("guilds.read")),
    ("member-join", Some("members.read")),
    ("member-leave", Some("members.read")),
    ("channel-select", None),
    ("guild-select", None),
    ("notification-click", None),
    ("view-closed", None),
    ("window-focus", None),
    ("window-blur", None),
    ("idle-change", None),
    ("connection-open", None),
    ("composer-change", Some("composer")),
];

pub fn permission_for(kind: &str) -> Option<Option<&'static str>> {
    EVENT_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, p)| *p)
}
