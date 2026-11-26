import QtQuick
import Lampcord

Column {
    id: root
    spacing: 16

    function setBool(prefKey, cfgKey, value) {
        Theme.prefs[prefKey] = value
        backend.setConfig(cfgKey, value ? "true" : "false")
    }

    SettingsGroupTitle { text: "Profile Enhancements" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "3y3 Profile Colors"
            subtitle: "Invisible bio text."
            checked: Theme.prefs.profile3y3
            first: true; last: false
            onToggled: value => root.setBool("profile3y3", "profile_3y3", value)
        }
        SettingsSwitchRow {
            title: "UserBG Banners"
            subtitle: "Custom banners database."
            checked: Theme.prefs.userbgEnabled
            first: false; last: false
            onToggled: value => root.setBool("userbgEnabled", "user_bg", value)
        }
        SettingsSwitchRow {
            title: "UserPFP Icons"
            subtitle: "Custom icons database."
            checked: Theme.prefs.userpfpEnabled
            first: false; last: false
            onToggled: value => root.setBool("userpfpEnabled", "user_pfp", value)
        }
        SettingsActionRow {
            icon: "Public"
            title: "Set UserBG Banner"
            subtitle: "Join server."
            first: false; last: true
            onClicked: Qt.openUrlExternally("https://discord.gg/ECg96KZ3Fh")
        }
    }

    SettingsGroupTitle { text: "Enhancements" }

    SettingsSwitchRow {
        title: "Favorites Server"
        subtitle: "Pin channels from any server to a Favorites list at the top of the rail. Free for everyone."
        checked: Theme.prefs.favoritesServerEnabled
        first: true; last: true
        onToggled: value => root.setBool("favoritesServerEnabled", "favorites_server_enabled", value)
    }

    SettingsGroupTitle { text: "Display" }

    SettingsSwitchRow {
        title: "Scrolling Text"
        subtitle: "Scroll long names and titles that don't fit instead of cutting them off."
        checked: Theme.prefs.marqueeText
        first: true; last: true
        onToggled: value => root.setBool("marqueeText", "marquee_text", value)
    }

    SettingsGroupTitle { text: "Clean Channels" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Remove Emojis"
            subtitle: "Remove emojis from channel names."
            checked: Theme.prefs.cleanChannelsRemoveEmojis
            first: true; last: false
            onToggled: value => root.setBool("cleanChannelsRemoveEmojis", "clean_channels_remove_emojis", value)
        }
        SettingsSwitchRow {
            title: "Hide Symbols"
            subtitle: "Restrict channel names to letters, digits, spaces, dashes, underscores, and #."
            checked: Theme.prefs.cleanChannelsHideSymbols
            first: false; last: false
            onToggled: value => root.setBool("cleanChannelsHideSymbols", "clean_channels_hide_symbols", value)
        }
        SettingsSwitchRow {
            title: "Normalize Letters"
            subtitle: "Convert accented and special letters to standard letters."
            checked: Theme.prefs.cleanChannelsNormalizeLetters
            first: false; last: false
            onToggled: value => root.setBool("cleanChannelsNormalizeLetters", "clean_channels_normalize_letters", value)
        }
        SettingsSwitchRow {
            title: "Capitalize Categories"
            subtitle: "Uniformly capitalize category names."
            checked: Theme.prefs.cleanChannelsCapitalizeCategories
            first: false; last: true
            onToggled: value => root.setBool("cleanChannelsCapitalizeCategories", "clean_channels_capitalize_categories", value)
        }
    }

    SettingsGroupTitle { text: "Chatbox" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Show Avatar in Chatbox"
            subtitle: "Displays your current avatar inside the chat input bar."
            checked: Theme.prefs.chatboxShowAvatar
            first: true; last: false
            onToggled: value => root.setBool("chatboxShowAvatar", "chatbox_show_avatar", value)
        }
        SettingsSwitchRow {
            title: "Silent Typing"
            subtitle: "Show a keyboard control for hiding your typing indicator."
            checked: Theme.prefs.silentTypingButtonEnabled
            first: false; last: true
            onToggled: value => root.setBool("silentTypingButtonEnabled", "silent_typing_button_enabled", value)
        }
    }

    SettingsGroupTitle { text: "Emoji" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Free Nitro Emojis"
            subtitle: "Use emojis from any server for free."
            checked: Theme.prefs.freeNitroEmojis
            first: true; last: false
            onToggled: value => root.setBool("freeNitroEmojis", "free_nitro_emojis", value)
        }
        SettingsSwitchRow {
            title: "Realmojis"
            subtitle: "Makes free nitro emojis look like real ones."
            checked: Theme.prefs.realmojis
            first: false; last: true
            onToggled: value => root.setBool("realmojis", "realmojis", value)
        }
    }

    SettingsGroupTitle { text: "Logger" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Message Logger"
            subtitle: "Keep a local history of deleted and edited messages."
            checked: Theme.prefs.logDeletedMessages
            first: true; last: false
            onToggled: value => {
                root.setBool("logDeletedMessages", "message_logger_enabled", value)
                backend.logDeletedMessages = value
            }
        }
        SettingsSwitchRow {
            visible: Theme.prefs.logDeletedMessages
            title: "Ignore Bots"
            checked: Theme.prefs.messageLoggerIgnoreBots
            first: false; last: false
            onToggled: value => root.setBool("messageLoggerIgnoreBots", "message_logger_ignore_bots", value)
        }
        SettingsSwitchRow {
            visible: Theme.prefs.logDeletedMessages
            title: "Ignore Self"
            checked: Theme.prefs.messageLoggerIgnoreSelf
            first: false; last: false
            onToggled: value => root.setBool("messageLoggerIgnoreSelf", "message_logger_ignore_self", value)
        }
        SettingsActionRow {
            title: "Clear Logged Messages"
            subtitle: "Permanently delete every message saved by Message Logger on this device."
            first: false; last: true
            onClicked: backend.clearDeletedLog()
        }
    }

    SettingsGroupTitle { text: "Text Replacement" }

    SettingsActionRow {
        icon: "TextFields"
        title: "Text Replacement Rules"
        subtitle: "Manage automatic text substitution rules."
        first: true; last: true
    }
}
