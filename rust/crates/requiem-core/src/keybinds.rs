use std::sync::OnceLock;

use requiem_ui::keymap::Chord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ServerNext,
    ServerPrev,
    ChannelNext,
    ChannelPrev,
    NavigateBack,
    NavigateForward,
    UnreadNext,
    UnreadPrev,
    MentionNext,
    MentionPrev,
    TogglePreviousGuild,

    JumpToGuild(u8),
    QuickSwitcher,
    CreateGuild,
    CreateDmGroup,
    MarkChannelRead,
    MarkServerRead,
    ToggleInbox,
    ToggleUsers,
    SearchEmojis,
    SearchGifs,
    ScrollUp,
    ScrollDown,
    JumpToFirstUnread,
    UploadFile,
    ToggleMute,
    ToggleDeafen,

    FocusSearch { prefill_channel: bool },

    ToggleHotkeys,

    ToggleHelp,
    CopyChannelLink,

    OpenContextMenu,

    DevTools,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Navigation,
    Chat,
    VoiceAndVideo,
    Miscellaneous,
    Message,
}

impl Group {
    pub const ALL: [Group; 5] = [
        Group::Navigation,
        Group::Chat,
        Group::VoiceAndVideo,
        Group::Miscellaneous,
        Group::Message,
    ];

    pub fn title(self) -> String {
        match self {
            Group::Navigation => requiem_i18n::t!("keybinds-group-navigation"),
            Group::Chat => requiem_i18n::t!("keybinds-group-chat"),
            Group::VoiceAndVideo => requiem_i18n::t!("keybinds-group-voice-and-video"),
            Group::Miscellaneous => requiem_i18n::t!("keybinds-group-miscellaneous"),
            Group::Message => requiem_i18n::t!("keybinds-group-messages"),
        }
    }
}

pub struct Entry {
    pub description: &'static str,
    pub binds: &'static [&'static str],
    pub group: Group,
}

struct Binding {
    action: Action,
    binds: &'static [&'static str],
}

const BINDINGS: &[Binding] = &[
    Binding {
        action: Action::ServerNext,
        binds: &["mod+alt+down", "mod+shift+]", "ctrl+tab"],
    },
    Binding {
        action: Action::ServerPrev,
        binds: &["mod+alt+up", "mod+shift+[", "ctrl+shift+tab"],
    },
    Binding {
        action: Action::ChannelNext,
        binds: &["alt+down"],
    },
    Binding {
        action: Action::ChannelPrev,
        binds: &["alt+up"],
    },
    Binding {
        action: Action::NavigateBack,
        binds: &["alt+left"],
    },
    Binding {
        action: Action::NavigateForward,
        binds: &["alt+right"],
    },
    Binding {
        action: Action::UnreadNext,
        binds: &["alt+shift+down"],
    },
    Binding {
        action: Action::UnreadPrev,
        binds: &["alt+shift+up"],
    },
    Binding {
        action: Action::MentionNext,
        binds: &["mod+shift+alt+down"],
    },
    Binding {
        action: Action::MentionPrev,
        binds: &["mod+shift+alt+up"],
    },
    Binding {
        action: Action::TogglePreviousGuild,
        binds: &["mod+alt+right"],
    },
    Binding {
        action: Action::JumpToGuild(1),
        binds: &["mod+1"],
    },
    Binding {
        action: Action::JumpToGuild(2),
        binds: &["mod+2"],
    },
    Binding {
        action: Action::JumpToGuild(3),
        binds: &["mod+3"],
    },
    Binding {
        action: Action::JumpToGuild(4),
        binds: &["mod+4"],
    },
    Binding {
        action: Action::JumpToGuild(5),
        binds: &["mod+5"],
    },
    Binding {
        action: Action::JumpToGuild(6),
        binds: &["mod+6"],
    },
    Binding {
        action: Action::JumpToGuild(7),
        binds: &["mod+7"],
    },
    Binding {
        action: Action::JumpToGuild(8),
        binds: &["mod+8"],
    },
    Binding {
        action: Action::JumpToGuild(9),
        binds: &["mod+9"],
    },
    Binding {
        action: Action::QuickSwitcher,
        binds: &["mod+k", "mod+t"],
    },
    Binding {
        action: Action::CreateGuild,
        binds: &["mod+shift+n"],
    },
    Binding {
        action: Action::CreateDmGroup,
        binds: &["mod+shift+t"],
    },
    Binding {
        action: Action::MarkChannelRead,
        binds: &["shift+pagedown"],
    },
    Binding {
        action: Action::MarkServerRead,
        binds: &["shift+esc"],
    },
    Binding {
        action: Action::ToggleInbox,
        binds: &["mod+i"],
    },
    Binding {
        action: Action::ToggleUsers,
        binds: &["mod+u"],
    },
    Binding {
        action: Action::SearchEmojis,
        binds: &["mod+e"],
    },
    Binding {
        action: Action::SearchGifs,
        binds: &["mod+g"],
    },
    Binding {
        action: Action::ScrollUp,
        binds: &["pageup"],
    },
    Binding {
        action: Action::ScrollDown,
        binds: &["pagedown"],
    },
    Binding {
        action: Action::JumpToFirstUnread,
        binds: &["shift+pageup"],
    },
    Binding {
        action: Action::UploadFile,
        binds: &["mod+shift+u"],
    },
    Binding {
        action: Action::ToggleMute,
        binds: &["mod+shift+m"],
    },
    Binding {
        action: Action::ToggleDeafen,
        binds: &["mod+shift+d"],
    },
    Binding {
        action: Action::FocusSearch {
            prefill_channel: true,
        },
        binds: &["mod+f"],
    },
    Binding {
        action: Action::FocusSearch {
            prefill_channel: false,
        },
        binds: &["mod+shift+f"],
    },
    Binding {
        action: Action::ToggleHotkeys,
        binds: &["mod+/", "mod+shift+/"],
    },
    Binding {
        action: Action::ToggleHelp,
        binds: &["mod+shift+h", "f1"],
    },
    Binding {
        action: Action::CopyChannelLink,
        binds: &["mod+shift+l"],
    },
    Binding {
        action: Action::OpenContextMenu,
        binds: &["shift+f10"],
    },
    Binding {
        action: Action::DevTools,
        binds: &["mod+alt+i"],
    },
];

pub fn action_for(chord: &Chord) -> Option<Action> {
    static TABLE: OnceLock<Vec<(Chord, Action)>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        BINDINGS
            .iter()
            .flat_map(|b| {
                b.binds
                    .iter()
                    .filter_map(move |bind| Chord::parse(bind).map(|chord| (chord, b.action)))
            })
            .collect()
    });
    table
        .iter()
        .find(|(bound, _)| bound == chord)
        .map(|(_, action)| *action)
}

pub const ENTRIES: &[Entry] = &[
    Entry {
        description: requiem_i18n::key!("keybinds-navigate-servers"),
        binds: &["mod+alt+up", "mod+alt+down"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-navigate-channels"),
        binds: &["alt+up", "alt+down"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-navigate-history"),
        binds: &["alt+left", "alt+right"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-navigate-unread"),
        binds: &["alt+shift+up", "alt+shift+down"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-navigate-mentions"),
        binds: &["mod+shift+alt+up", "mod+shift+alt+down"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-previous-server"),
        binds: &["mod+alt+right"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-jump-to-server"),
        binds: &["mod+1-9"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-quick-switcher"),
        binds: &["mod+k"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-create-or-join-server"),
        binds: &["mod+shift+n"],
        group: Group::Navigation,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-mark-server-read"),
        binds: &["shift+esc"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-mark-channel-read"),
        binds: &["esc"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-create-group"),
        binds: &["mod+shift+t"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-inbox"),
        binds: &["mod+i"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-members"),
        binds: &["mod+u"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-emoji"),
        binds: &["mod+e"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-gif"),
        binds: &["mod+g"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-scroll-chat"),
        binds: &["pageup", "pagedown"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-jump-to-unread"),
        binds: &["shift+pageup"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-upload-file"),
        binds: &["mod+shift+u"],
        group: Group::Chat,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-mute"),
        binds: &["mod+shift+m"],
        group: Group::VoiceAndVideo,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-toggle-deafen"),
        binds: &["mod+shift+d"],
        group: Group::VoiceAndVideo,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-get-help"),
        binds: &["f1"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-search"),
        binds: &["mod+f"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-search-everywhere"),
        binds: &["mod+shift+f"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-copy-channel-link"),
        binds: &["mod+shift+l"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-open-context-menu"),
        binds: &["shift+f10"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-show-shortcuts"),
        binds: &["mod+/"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-zoom"),
        binds: &["mod+plus", "mod+minus", "mod+0"],
        group: Group::Miscellaneous,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-edit-message"),
        binds: &["e"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-delete-message"),
        binds: &["backspace"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-pin-message"),
        binds: &["p"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-add-reaction"),
        binds: &["plus"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-reply"),
        binds: &["r"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-copy-text"),
        binds: &["mod+c"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-mark-unread"),
        binds: &["alt+enter"],
        group: Group::Message,
    },
    Entry {
        description: requiem_i18n::key!("keybinds-focus-text-area"),
        binds: &["escape"],
        group: Group::Message,
    },
];

pub fn keycaps(bind: &str) -> Vec<String> {
    bind.split('+')
        .map(|part| match part {
            "mod" if cfg!(target_os = "macos") => "⌘".to_string(),
            "mod" | "ctrl" => "CTRL".to_string(),
            "alt" => "ALT".to_string(),
            "shift" => "SHIFT".to_string(),
            "up" => "↑".to_string(),
            "down" => "↓".to_string(),
            "left" => "←".to_string(),
            "right" => "→".to_string(),
            "esc" | "escape" => "ESC".to_string(),
            "pageup" => "PGUP".to_string(),
            "pagedown" => "PGDN".to_string(),
            "backspace" => "⌫".to_string(),
            "tab" => "TAB".to_string(),
            "plus" => "+".to_string(),
            "minus" => "-".to_string(),
            other => other.to_uppercase(),
        })
        .collect()
}

pub fn display_groups(binds: &[&str]) -> Vec<Vec<String>> {
    let split = |bind: &str| -> (String, String) {
        match bind.rsplit_once('+') {
            Some((mods, key)) if !mods.is_empty() => (mods.to_string(), key.to_string()),
            _ => (String::new(), bind.to_string()),
        }
    };
    let parts: Vec<(String, String)> = binds.iter().map(|b| split(b)).collect();
    if parts.len() > 1
        && parts.iter().all(|(mods, _)| *mods == parts[0].0)
        && matches!(
            parts[0].1.as_str(),
            "up" | "down" | "left" | "right" | "pageup" | "pagedown"
        )
    {
        let mut caps = if parts[0].0.is_empty() {
            Vec::new()
        } else {
            keycaps(&format!("{}+x", parts[0].0))
                .into_iter()
                .rev()
                .skip(1)
                .rev()
                .collect()
        };
        caps.extend(parts.iter().flat_map(|(_, key)| keycaps(key)));
        return vec![caps];
    }
    binds.iter().map(|b| keycaps(b)).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn every_bind_parses() {
        for binding in BINDINGS {
            for bind in binding.binds {
                assert!(Chord::parse(bind).is_some(), "{bind} does not parse");
            }
        }
    }

    #[test]
    fn no_chord_is_bound_twice() {
        let mut seen = HashSet::new();
        for binding in BINDINGS {
            for bind in binding.binds {
                assert!(
                    seen.insert(Chord::parse(bind).unwrap()),
                    "{bind} is bound twice"
                );
            }
        }
    }

    #[test]
    fn lookup_resolves_discord_defaults() {
        let chord = |bind| Chord::parse(bind).unwrap();
        assert_eq!(action_for(&chord("mod+k")), Some(Action::QuickSwitcher));
        assert_eq!(action_for(&chord("alt+down")), Some(Action::ChannelNext));
        assert_eq!(action_for(&chord("mod+5")), Some(Action::JumpToGuild(5)));
        assert_eq!(
            action_for(&chord("mod+shift+f")),
            Some(Action::FocusSearch {
                prefill_channel: false
            })
        );
        assert_eq!(action_for(&chord("mod+q")), None);
    }

    #[test]
    fn up_and_down_share_their_modifiers() {
        assert_eq!(display_groups(&["alt+up", "alt+down"]), [["ALT", "↑", "↓"]]);
        assert_eq!(display_groups(&["pageup", "pagedown"]), [["PGUP", "PGDN"]]);
        assert_eq!(display_groups(&["mod+k"]), [["CTRL", "K"]]);
        assert_eq!(display_groups(&["mod+shift+n", "mod+t"]).len(), 2);
    }

    #[test]
    fn keycaps_read_naturally() {
        assert_eq!(keycaps("mod+shift+k"), ["CTRL", "SHIFT", "K"]);
        assert_eq!(keycaps("alt+down"), ["ALT", "↓"]);
    }
}
