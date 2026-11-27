use crate::models::Permissions;

pub struct PermissionRow {
    pub title: &'static str,

    pub description: &'static str,
    pub flag: Permissions,
}

pub struct PermissionSection {
    pub title: &'static str,
    pub rows: &'static [PermissionRow],
}

const fn row(title: &'static str, description: &'static str, flag: Permissions) -> PermissionRow {
    PermissionRow {
        title,
        description,
        flag,
    }
}

pub const SECTIONS: &[PermissionSection] = &[
    PermissionSection {
        title: requiem_i18n::key!("permission-section-general-server-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-view-channels"),
                requiem_i18n::key!("permission-view-channels-description"),
                Permissions::VIEW_CHANNEL,
            ),
            row(
                requiem_i18n::key!("permission-manage-channels"),
                requiem_i18n::key!("permission-manage-channels-description"),
                Permissions::MANAGE_CHANNELS,
            ),
            row(
                requiem_i18n::key!("permission-manage-roles"),
                requiem_i18n::key!("permission-manage-roles-description"),
                Permissions::MANAGE_ROLES,
            ),
            row(
                requiem_i18n::key!("permission-create-expressions"),
                requiem_i18n::key!("permission-create-expressions-description"),
                Permissions::CREATE_GUILD_EXPRESSIONS,
            ),
            row(
                requiem_i18n::key!("permission-manage-expressions"),
                requiem_i18n::key!("permission-manage-expressions-description"),
                Permissions::MANAGE_GUILD_EXPRESSIONS,
            ),
            row(
                requiem_i18n::key!("permission-view-audit-log"),
                requiem_i18n::key!("permission-view-audit-log-description"),
                Permissions::VIEW_AUDIT_LOG,
            ),
            row(
                requiem_i18n::key!("permission-manage-webhooks"),
                requiem_i18n::key!("permission-manage-webhooks-description"),
                Permissions::MANAGE_WEBHOOKS,
            ),
            row(
                requiem_i18n::key!("permission-manage-server"),
                requiem_i18n::key!("permission-manage-server-description"),
                Permissions::MANAGE_GUILD,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-membership-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-create-invite"),
                requiem_i18n::key!("permission-create-invite-description"),
                Permissions::CREATE_INSTANT_INVITE,
            ),
            row(
                requiem_i18n::key!("permission-change-nickname"),
                requiem_i18n::key!("permission-change-nickname-description"),
                Permissions::CHANGE_NICKNAME,
            ),
            row(
                requiem_i18n::key!("permission-manage-nicknames"),
                requiem_i18n::key!("permission-manage-nicknames-description"),
                Permissions::MANAGE_NICKNAMES,
            ),
            row(
                requiem_i18n::key!("permission-kick-approve-and-reject-members"),
                requiem_i18n::key!("permission-kick-approve-and-reject-members-description"),
                Permissions::KICK_MEMBERS,
            ),
            row(
                requiem_i18n::key!("permission-ban-members"),
                requiem_i18n::key!("permission-ban-members-description"),
                Permissions::BAN_MEMBERS,
            ),
            row(
                requiem_i18n::key!("permission-timeout-members"),
                requiem_i18n::key!("permission-timeout-members-description"),
                Permissions::MODERATE_MEMBERS,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-text-channel-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-send-messages-and-create-posts"),
                requiem_i18n::key!("permission-send-messages-and-create-posts-description"),
                Permissions::SEND_MESSAGES,
            ),
            row(
                requiem_i18n::key!("permission-send-messages-in-threads-and-posts"),
                requiem_i18n::key!("permission-send-messages-in-threads-and-posts-description"),
                Permissions::SEND_MESSAGES_IN_THREADS,
            ),
            row(
                requiem_i18n::key!("permission-create-public-threads"),
                requiem_i18n::key!("permission-create-public-threads-description"),
                Permissions::CREATE_PUBLIC_THREADS,
            ),
            row(
                requiem_i18n::key!("permission-create-private-threads"),
                requiem_i18n::key!("permission-create-private-threads-description"),
                Permissions::CREATE_PRIVATE_THREADS,
            ),
            row(
                requiem_i18n::key!("permission-embed-links"),
                requiem_i18n::key!("permission-embed-links-description"),
                Permissions::EMBED_LINKS,
            ),
            row(
                requiem_i18n::key!("permission-attach-files"),
                requiem_i18n::key!("permission-attach-files-description"),
                Permissions::ATTACH_FILES,
            ),
            row(
                requiem_i18n::key!("permission-add-reactions"),
                requiem_i18n::key!("permission-add-reactions-description"),
                Permissions::ADD_REACTIONS,
            ),
            row(
                requiem_i18n::key!("permission-use-external-emoji"),
                requiem_i18n::key!("permission-use-external-emoji-description"),
                Permissions::USE_EXTERNAL_EMOJIS,
            ),
            row(
                requiem_i18n::key!("permission-use-external-stickers"),
                requiem_i18n::key!("permission-use-external-stickers-description"),
                Permissions::USE_EXTERNAL_STICKERS,
            ),
            row(
                requiem_i18n::key!("permission-mention-everyone-here-and-all-roles"),
                requiem_i18n::key!("permission-mention-everyone-here-and-all-roles-description"),
                Permissions::MENTION_EVERYONE,
            ),
            row(
                requiem_i18n::key!("permission-manage-messages"),
                requiem_i18n::key!("permission-manage-messages-description"),
                Permissions::MANAGE_MESSAGES,
            ),
            row(
                requiem_i18n::key!("permission-pin-messages"),
                requiem_i18n::key!("permission-pin-messages-description"),
                Permissions::PIN_MESSAGES,
            ),
            row(
                requiem_i18n::key!("permission-bypass-slowmode"),
                requiem_i18n::key!("permission-bypass-slowmode-description"),
                Permissions::BYPASS_SLOWMODE,
            ),
            row(
                requiem_i18n::key!("permission-manage-threads-and-posts"),
                requiem_i18n::key!("permission-manage-threads-and-posts-description"),
                Permissions::MANAGE_THREADS,
            ),
            row(
                requiem_i18n::key!("permission-read-message-history"),
                requiem_i18n::key!("permission-read-message-history-description"),
                Permissions::READ_MESSAGE_HISTORY,
            ),
            row(
                requiem_i18n::key!("permission-send-text-to-speech-messages"),
                requiem_i18n::key!("permission-send-text-to-speech-messages-description"),
                Permissions::SEND_TTS_MESSAGES,
            ),
            row(
                requiem_i18n::key!("permission-send-voice-messages"),
                requiem_i18n::key!("permission-send-voice-messages-description"),
                Permissions::SEND_VOICE_MESSAGES,
            ),
            row(
                requiem_i18n::key!("permission-create-polls"),
                requiem_i18n::key!("permission-create-polls-description"),
                Permissions::SEND_POLLS,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-voice-channel-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-connect"),
                requiem_i18n::key!("permission-connect-description"),
                Permissions::CONNECT,
            ),
            row(
                requiem_i18n::key!("permission-speak"),
                requiem_i18n::key!("permission-speak-description"),
                Permissions::SPEAK,
            ),
            row(
                requiem_i18n::key!("permission-video"),
                requiem_i18n::key!("permission-video-description"),
                Permissions::STREAM,
            ),
            row(
                requiem_i18n::key!("permission-use-soundboard"),
                requiem_i18n::key!("permission-use-soundboard-description"),
                Permissions::USE_SOUNDBOARD,
            ),
            row(
                requiem_i18n::key!("permission-use-external-sounds"),
                requiem_i18n::key!("permission-use-external-sounds-description"),
                Permissions::USE_EXTERNAL_SOUNDS,
            ),
            row(
                requiem_i18n::key!("permission-use-voice-activity"),
                requiem_i18n::key!("permission-use-voice-activity-description"),
                Permissions::USE_VAD,
            ),
            row(
                requiem_i18n::key!("permission-priority-speaker"),
                requiem_i18n::key!("permission-priority-speaker-description"),
                Permissions::PRIORITY_SPEAKER,
            ),
            row(
                requiem_i18n::key!("permission-mute-members"),
                requiem_i18n::key!("permission-mute-members-description"),
                Permissions::MUTE_MEMBERS,
            ),
            row(
                requiem_i18n::key!("permission-deafen-members"),
                requiem_i18n::key!("permission-deafen-members-description"),
                Permissions::DEAFEN_MEMBERS,
            ),
            row(
                requiem_i18n::key!("permission-move-members"),
                requiem_i18n::key!("permission-move-members-description"),
                Permissions::MOVE_MEMBERS,
            ),
            row(
                requiem_i18n::key!("permission-set-voice-channel-status"),
                requiem_i18n::key!("permission-set-voice-channel-status-description"),
                Permissions::SET_VOICE_CHANNEL_STATUS,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-apps-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-use-application-commands"),
                requiem_i18n::key!("permission-use-application-commands-description"),
                Permissions::USE_APPLICATION_COMMANDS,
            ),
            row(
                requiem_i18n::key!("permission-use-activities"),
                requiem_i18n::key!("permission-use-activities-description"),
                Permissions::USE_EMBEDDED_ACTIVITIES,
            ),
            row(
                requiem_i18n::key!("permission-use-external-apps"),
                requiem_i18n::key!("permission-use-external-apps-description"),
                Permissions::USE_EXTERNAL_APPS,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-events-permissions"),
        rows: &[
            row(
                requiem_i18n::key!("permission-create-events"),
                requiem_i18n::key!("permission-create-events-description"),
                Permissions::CREATE_EVENTS,
            ),
            row(
                requiem_i18n::key!("permission-manage-events"),
                requiem_i18n::key!("permission-manage-events-description"),
                Permissions::MANAGE_EVENTS,
            ),
        ],
    },
    PermissionSection {
        title: requiem_i18n::key!("permission-section-advanced-permissions"),
        rows: &[row(
            requiem_i18n::key!("permission-administrator"),
            requiem_i18n::key!("permission-administrator-description"),
            Permissions::ADMINISTRATOR,
        )],
    },
];

pub fn filtered(query: &str) -> Vec<(&'static str, Vec<&'static PermissionRow>)> {
    let query = query.trim_start().to_lowercase();
    SECTIONS
        .iter()
        .map(|section| {
            (
                section.title,
                section
                    .rows
                    .iter()
                    .filter(|row| {
                        requiem_i18n::text(row.title)
                            .to_lowercase()
                            .contains(&query)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, rows)| !rows.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_permission_appears_once() {
        let mut seen = Permissions::NONE;
        for section in SECTIONS {
            for row in section.rows {
                assert!(
                    !seen.contains(row.flag),
                    "{} listed twice",
                    requiem_i18n::text(row.title)
                );
                seen = seen | row.flag;
            }
        }

        assert_eq!(
            seen,
            Permissions::ALL
                .without(Permissions::VIEW_GUILD_INSIGHTS)
                .without(Permissions::VIEW_CREATOR_MONETIZATION_ANALYTICS)
                .without(Permissions::REQUEST_TO_SPEAK)
        );
    }

    #[test]
    fn search_matches_titles_case_insensitively() {
        let found = filtered("  MESSAGES");
        let titles: Vec<String> = found
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|r| requiem_i18n::text(r.title)))
            .collect();
        assert!(titles.contains(&"Manage Messages".to_string()));
        assert!(!titles.contains(&"Connect".to_string()));
    }
}
