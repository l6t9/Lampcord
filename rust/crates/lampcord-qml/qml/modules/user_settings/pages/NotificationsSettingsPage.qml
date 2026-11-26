import QtQuick
import Lampcord

Column {
    id: root
    spacing: 16

    function setBool(prefKey, cfgKey, value) {
        Theme.prefs[prefKey] = value
        backend.setConfig(cfgKey, value ? "true" : "false")
    }

    SettingsGroupTitle { text: "Notifications" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Enable Notifications"
            subtitle: "Show notifications in your system tray when Lampcord is in the background."
            checked: Theme.prefs.desktopNotifications
            first: true; last: false
            onToggled: value => {
                root.setBool("desktopNotifications", "notifications_enabled", value)
                backend.desktopNotifications = value
            }
        }
        SettingsSwitchRow {
            title: "Show Message Preview"
            subtitle: "Include message content in notifications."
            checked: Theme.prefs.showMessagePreview
            first: false; last: false
            onToggled: value => root.setBool("showMessagePreview", "show_message_preview", value)
        }
        SettingsSwitchRow {
            title: "Unobtrusive Unreads"
            subtitle: "Use subtle dimmed unread dots for low-priority and non-mention channel activity."
            checked: Theme.prefs.unobtrusiveUnreads
            first: false; last: true
            onToggled: value => root.setBool("unobtrusiveUnreads", "unobtrusive_unreads", value)
        }
    }

    SettingsGroupTitle { text: "Sounds" }

    Column {
        width: parent.width
        spacing: 2

        SettingsSwitchRow {
            title: "Message Sound"
            subtitle: "Play a sound when you receive a message."
            checked: Theme.prefs.messageSound
            first: true; last: false
            onToggled: value => root.setBool("messageSound", "notification_sound", value)
        }
        SettingsSwitchRow {
            title: "Incoming Call Sound"
            subtitle: "Play a sound when you are being called."
            checked: Theme.prefs.incomingCallSound
            first: false; last: true
            onToggled: value => root.setBool("incomingCallSound", "incoming_call_sound", value)
        }
    }

    SettingsGroupTitle { text: "In-app notifications" }

    SettingsSwitchRow {
        title: "Show In-App Notifications"
        subtitle: "Display banners in the app when someone mentions or pings you."
        checked: Theme.prefs.inAppToasts
        first: true; last: true
        onToggled: value => root.setBool("inAppToasts", "show_in_app_notifications", value)
    }
}
