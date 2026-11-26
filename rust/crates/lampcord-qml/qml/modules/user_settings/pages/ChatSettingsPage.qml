import QtQuick
import Lampcord

Column {
    id: root
    spacing: 16

    function setBool(prefKey, cfgKey, value) {
        Theme.prefs[prefKey] = value
        backend.setConfig(cfgKey, value ? "true" : "false")
    }

    SettingsGroupTitle { text: "Display" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Chat Bubbles"
            subtitle: "Display messages inside rounded chat bubbles."
            checked: Theme.prefs.chatBubbles
            first: true; last: false
            onToggled: value => root.setBool("chatBubbles", "chat_bubbles", value)
        }
        SettingsSwitchRow {
            title: "Compact Mode"
            subtitle: "Display messages in a compact IRC-style layout."
            checked: Theme.prefs.compactMessages
            first: false; last: false
            onToggled: value => root.setBool("compactMessages", "compact_mode", value)
        }
        SettingsSwitchRow {
            title: "Show Message in Context Menu"
            subtitle: "View the message and quick reactions inside the context menu."
            checked: Theme.prefs.showContextMenuMessage
            first: false; last: false
            onToggled: value => root.setBool("showContextMenuMessage", "show_context_menu_message", value)
        }
        SettingsSwitchRow {
            title: "Show Search in Header"
            subtitle: "Show the search button in the chat header."
            checked: Theme.prefs.showChatSearch
            first: false; last: false
            onToggled: value => root.setBool("showChatSearch", "show_chat_search", value)
        }
        SettingsSwitchRow {
            title: "Show Pins in Header"
            subtitle: "Show the pinned messages button in the chat header."
            checked: Theme.prefs.showChatPins
            first: false; last: false
            onToggled: value => root.setBool("showChatPins", "show_chat_pins", value)
        }
        SettingsSwitchRow {
            title: "Show Call Button"
            subtitle: "Show the call button in DM headers."
            checked: Theme.prefs.showCallButton
            first: false; last: true
            onToggled: value => root.setBool("showCallButton", "show_call_button", value)
        }
    }

    SettingsGroupTitle { text: "Voice" }

    SettingsSwitchRow {
        title: "Noise Cancellation"
        subtitle: "Remove background noise from your microphone using RNNoise."
        checked: Theme.prefs.noiseCancellation
        first: true; last: true
        onToggled: value => root.setBool("noiseCancellation", "noise_cancellation", value)
    }

    SettingsGroupTitle { text: "Chatbox Customization" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSliderRow {
            title: "Font Scale"
            label: Math.round(Theme.prefs.chatboxFontSize * 100) + "%"
            from: 0.5; to: 2.0; value: Theme.prefs.chatboxFontSize
            first: true; last: false
            onMoved: v => {
                Theme.prefs.chatboxFontSize = v
                backend.setConfig("chatbox_font_size", v.toFixed(2))
            }
        }
        SettingsSliderRow {
            title: "Background Opacity"
            label: Math.round(Theme.prefs.chatboxBackgroundOpacity * 100) + "%"
            from: 0.0; to: 1.0; value: Theme.prefs.chatboxBackgroundOpacity
            first: false; last: false
            onMoved: v => {
                Theme.prefs.chatboxBackgroundOpacity = v
                backend.setConfig("chatbox_background_opacity", v.toFixed(2))
            }
        }
        SettingsSliderRow {
            title: "Border Radius"
            label: Theme.prefs.chatboxBorderRadius + "dp"
            from: 0; to: 32; value: Theme.prefs.chatboxBorderRadius
            first: false; last: false
            onMoved: v => {
                Theme.prefs.chatboxBorderRadius = Math.round(v)
                backend.setConfig("chatbox_border_radius", String(Math.round(v)))
            }
        }
        SettingsSwitchRow {
            title: "Hide Upload Button"
            checked: Theme.prefs.chatboxHideUploadButton
            first: false; last: false
            onToggled: value => root.setBool("chatboxHideUploadButton", "chatbox_hide_upload_button", value)
        }
        SettingsSwitchRow {
            title: "Hide Emoji Button"
            checked: Theme.prefs.chatboxHideEmojiButton
            first: false; last: true
            onToggled: value => root.setBool("chatboxHideEmojiButton", "chatbox_hide_emoji_button", value)
        }
    }

    SettingsGroupTitle { text: "Gestures" }

    Column {
        width: parent.width
        spacing: 2

        Rectangle {
            width: parent.width
            height: 98
            topLeftRadius: 20; topRightRadius: 20; bottomLeftRadius: 4; bottomRightRadius: 4
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10
                Text { text: "TapTap Action"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
                Row {
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: [
                            { key: "reply_or_edit", cfg: "REPLY_OR_EDIT", label: "Reply/Edit", icon: "Reply" },
                            { key: "react", cfg: "REACT", label: "React", icon: "AddReaction" },
                            { key: "delete", cfg: "DELETE", label: "Delete", icon: "Delete" },
                            { key: "disabled", cfg: "DISABLED", label: "Disabled", icon: "Close" }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            readonly property bool active: Theme.prefs.tapTap === modelData.key
                            width: (parent.width - 12) / 4
                            height: 38
                            radius: active ? 19 : 10
                            color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                            Row {
                                anchors.centerIn: parent
                                spacing: 5
                                Icon { anchors.verticalCenter: parent.verticalCenter; name: modelData.icon; size: 15; color: active ? Theme.primaryContainerText : Theme.surfaceVariantText }
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 12; font.weight: Font.DemiBold }
                            }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.tapTap = modelData.key
                                    backend.setConfig("taptap_action", modelData.cfg)
                                }
                            }
                        }
                    }
                }
            }
        }

        SettingsSwitchRow {
            title: "Swipe to Reply"
            subtitle: "Swipe a message left to reply to it."
            checked: Theme.prefs.swipeToReplyEnabled
            first: false; last: true
            onToggled: value => root.setBool("swipeToReplyEnabled", "swipe_to_reply_enabled", value)
        }
    }

    SettingsGroupTitle { text: "Media" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Auto-display uploads"
            subtitle: "Images and videos uploaded directly to Discord."
            checked: Theme.prefs.autoDisplayUploads
            first: true; last: false
            onToggled: value => Theme.prefs.autoDisplayUploads = value
        }
        SettingsSwitchRow {
            title: "Auto-display links"
            subtitle: "Links to rich media from other websites."
            checked: Theme.prefs.autoDisplayLinks
            first: false; last: false
            onToggled: value => Theme.prefs.autoDisplayLinks = value
        }
        SettingsSwitchRow {
            title: "Show embeds"
            subtitle: "Previews for website links pasted into chat."
            checked: Theme.prefs.showEmbeds
            first: false; last: false
            onToggled: value => Theme.prefs.showEmbeds = value
        }
        SettingsSwitchRow {
            title: "Show reactions"
            subtitle: "Display reactions on messages."
            checked: Theme.prefs.showReactions
            first: false; last: true
            onToggled: value => Theme.prefs.showReactions = value
        }
    }

    SettingsGroupTitle { text: "Emoji and Stickers" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Animate Emoji"
            checked: Theme.prefs.animateEmoji
            first: true; last: false
            onToggled: value => Theme.prefs.animateEmoji = value
        }

        Rectangle {
            width: parent.width
            height: 98
            topLeftRadius: 4; topRightRadius: 4; bottomLeftRadius: 20; bottomRightRadius: 20
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10
                Text { text: "Animate Stickers"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
                Row {
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: [
                            { key: "always", cfg: "ALWAYS", label: "Always", icon: "PlayArrow" },
                            { key: "on_interaction", cfg: "ON_INTERACTION", label: "On interaction", icon: "TouchApp" },
                            { key: "never", cfg: "NEVER", label: "Never", icon: "Pause" }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            readonly property bool active: Theme.prefs.animateStickers === modelData.key
                            width: (parent.width - 8) / 3
                            height: 38
                            topLeftRadius: index === 0 ? 19 : 8; bottomLeftRadius: index === 0 ? 19 : 8
                            topRightRadius: index === 2 ? 19 : 8; bottomRightRadius: index === 2 ? 19 : 8
                            color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                            Row {
                                anchors.centerIn: parent
                                spacing: 5
                                Icon { anchors.verticalCenter: parent.verticalCenter; name: modelData.icon; size: 15; color: active ? Theme.primaryContainerText : Theme.surfaceVariantText }
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 12; font.weight: Font.DemiBold }
                            }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.animateStickers = modelData.key
                                    backend.setConfig("animate_stickers", modelData.cfg)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
