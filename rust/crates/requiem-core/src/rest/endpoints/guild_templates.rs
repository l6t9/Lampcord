use serde::Serialize;

pub const GUILD_CATEGORY: u8 = 4;
pub const GUILD_TEXT: u8 = 0;
pub const GUILD_VOICE: u8 = 2;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct TemplateChannel {
    pub id: &'static str,
    pub parent_id: Option<&'static str>,
    pub name: &'static str,
    #[serde(rename = "type")]
    pub kind: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct GuildTemplate {
    pub code: &'static str,
    pub channels: &'static [TemplateChannel],
    pub system_channel_id: Option<&'static str>,
}

const fn category(id: &'static str, name: &'static str) -> TemplateChannel {
    TemplateChannel {
        id,
        parent_id: None,
        name,
        kind: GUILD_CATEGORY,
    }
}

const fn text(id: &'static str, parent: &'static str, name: &'static str) -> TemplateChannel {
    TemplateChannel {
        id,
        parent_id: Some(parent),
        name,
        kind: GUILD_TEXT,
    }
}

const fn voice(id: &'static str, parent: &'static str, name: &'static str) -> TemplateChannel {
    TemplateChannel {
        id,
        parent_id: Some(parent),
        name,
        kind: GUILD_VOICE,
    }
}

pub const CREATE: GuildTemplate = GuildTemplate {
    code: "2TffvPucqHkN",
    channels: &[],
    system_channel_id: None,
};

pub const GAMING: GuildTemplate = GuildTemplate {
    code: "hvtBQMfw6uSJ",
    channels: &[
        category("00", "Text Channels"),
        text("01", "00", "general"),
        text("02", "00", "clips-and-highlights"),
        category("10", "Voice Channels"),
        voice("11", "10", "Lobby"),
        voice("12", "10", "Gaming"),
    ],
    system_channel_id: Some("01"),
};

pub const FRIENDS: GuildTemplate = GuildTemplate {
    code: "hgM48av5Q69A",
    channels: &[
        category("00", "Text Channels"),
        text("01", "00", "general"),
        text("02", "00", "games"),
        text("03", "00", "music"),
        category("10", "Voice Channels"),
        voice("11", "10", "Lounge"),
        voice("12", "10", "Stream Room"),
    ],
    system_channel_id: Some("01"),
};

pub const STUDY_GROUP: GuildTemplate = GuildTemplate {
    code: "FbwUwRp4j8Es",
    channels: &[
        category("00", "Information"),
        text("01", "00", "welcome-and-rules"),
        text("02", "00", "notes-resources"),
        category("10", "Text Channels"),
        text("11", "10", "general"),
        text("12", "10", "homework-help"),
        text("13", "10", "session-planning"),
        text("14", "10", "off-topic"),
        category("20", "Voice Channels"),
        voice("21", "20", "Lounge"),
        voice("22", "20", "Study Room 1"),
        voice("23", "20", "Study Room 2"),
    ],
    system_channel_id: Some("11"),
};

pub const SCHOOL_CLUB: GuildTemplate = GuildTemplate {
    code: "Ctg7PUHcQmZu",
    channels: &[
        category("00", "Information"),
        text("01", "00", "welcome-and-rules"),
        text("02", "00", "announcements"),
        text("03", "00", "resources"),
        category("10", "Text Channels"),
        text("11", "10", "general"),
        text("12", "10", "meeting-plans"),
        text("13", "10", "off-topic"),
        category("20", "Voice Channels"),
        voice("21", "20", "Lounge"),
        voice("22", "20", "Meeting Room 1"),
        voice("22", "20", "Meeting Room 2"),
    ],
    system_channel_id: Some("11"),
};

pub const LOCAL_COMMUNITY: GuildTemplate = GuildTemplate {
    code: "64UDvRNCC52Y",
    channels: &[
        category("00", "Information"),
        text("01", "00", "welcome-and-rules"),
        text("02", "00", "announcements"),
        text("03", "00", "resources"),
        category("10", "Text Channels"),
        text("11", "10", "general"),
        text("12", "10", "meeting-plans"),
        text("13", "10", "off-topic"),
        category("20", "Voice Channels"),
        voice("21", "20", "Lounge"),
        voice("22", "20", "Meeting Room"),
    ],
    system_channel_id: Some("11"),
};

pub const ARTISTS_CREATORS: GuildTemplate = GuildTemplate {
    code: "6exdzMgjZgah",
    channels: &[
        category("00", "Information"),
        text("01", "00", "welcome-and-rules"),
        text("02", "00", "announcements"),
        category("10", "Text Channels"),
        text("11", "10", "general"),
        text("12", "10", "events"),
        text("13", "10", "ideas-and-feedback"),
        category("20", "Voice Channels"),
        voice("21", "20", "Lounge"),
        voice("22", "20", "Community Hangout"),
        voice("23", "20", "Stream Room"),
    ],
    system_channel_id: Some("11"),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_match_what_the_real_client_sent() {
        let counts: Vec<usize> = [
            CREATE,
            GAMING,
            FRIENDS,
            STUDY_GROUP,
            SCHOOL_CLUB,
            LOCAL_COMMUNITY,
            ARTISTS_CREATORS,
        ]
        .iter()
        .map(|t| t.channels.len())
        .collect();
        assert_eq!(counts, [0, 6, 7, 12, 12, 11, 11]);
    }

    #[test]
    fn a_channel_serialises_in_the_real_key_order() {
        assert_eq!(
            serde_json::to_string(&GAMING.channels[1]).unwrap(),
            r#"{"id":"01","parent_id":"00","name":"general","type":0}"#
        );
        assert_eq!(
            serde_json::to_string(&GAMING.channels[0]).unwrap(),
            r#"{"id":"00","parent_id":null,"name":"Text Channels","type":4}"#
        );
    }
}
