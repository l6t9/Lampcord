import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Lampcord

Popup {
    id: root

    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 0
    width: Math.min(1080, parent ? parent.width - 48 : 1080)
    height: Math.min(820, parent ? parent.height - 64 : 820)
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    property string category: "account"
    readonly property var savedAccounts: {
        try { return JSON.parse(backend.savedAccountsJson) } catch (e) { return [] }
    }
    readonly property var myProfile: {
        try {
            const p = JSON.parse(backend.profileJson)
            return p && String(p.user_id) === backend.currentUserId ? p : null
        } catch (e) { return null }
    }

    readonly property var navGroups: [
        {
            title: "User Settings",
            items: [
                { key: "account", label: "Account", desc: "Manage your account details and switcher", icon: "AccountCircle" },
                { key: "profile", label: "Profiles", desc: "Customize your appearance and status", icon: "Person" },
                { key: "privacy", label: "Privacy & Safety", desc: "Manage who can contact you and data", icon: "Security" },
                { key: "connections", label: "Connections", desc: "Connect your accounts from other services", icon: "Link" },
                { key: "devices", label: "Devices", desc: "Manage your active sessions", icon: "Tv" }
            ]
        },
        {
            title: "App Settings",
            items: [
                { key: "appearance", label: "Appearance", desc: "Theme, colors, and message display", icon: "Palette" },
                { key: "accessibility", label: "Accessibility", desc: "Visual and interactive adjustments", icon: "Accessibility" },
                { key: "language", label: "Language", desc: "Select your preferred language", icon: "Language" },
                { key: "voice", label: "Voice & Video", desc: "Input, output, and camera settings", icon: "Mic" },
                { key: "chat", label: "Chat", desc: "Control how you interact with chat and media", icon: "Forum" },
                { key: "notifications", label: "Notifications", desc: "Control how you're notified", icon: "Notifications" },
                { key: "advanced", label: "Advanced", desc: "Developer settings and experimental features", icon: "Tune" },
                { key: "additional", label: "Additional", desc: "Profile enhancements, emoji tweaks and logger", icon: "AutoAwesome" }
            ]
        },
        {
            title: "",
            items: [
                { key: "updates", label: "Updates", desc: "Check for updates and read the changelog", icon: "Refresh" },
                { key: "about", label: "About", desc: "App information and credits", icon: "Info" }
            ]
        }
    ]

    readonly property string activeTitle: {
        for (const g of navGroups) {
            for (const item of g.items) {
                if (item.key === root.category)
                    return item.label
            }
        }
        return "Account"
    }

    function matchesSearch(item) {
        const q = searchField.text.trim().toLowerCase()
        if (q === "")
            return true
        return item.label.toLowerCase().includes(q) || item.desc.toLowerCase().includes(q)
    }

    onOpened: {
        searchField.text = ""
        Theme.loadFromLampcordConfig(backend.configJson)
        if (backend.currentUserId !== "")
            backend.fetchProfile(backend.currentUserId)
    }

    enter: Transition {
        ParallelAnimation {
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.quick }
            NumberAnimation { property: "scale"; from: 0.97; to: 1; duration: Theme.quick; easing.type: Easing.OutCubic }
        }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 120 }
    }

    Overlay.modal: Rectangle { color: Theme.alpha("#000000", 0.55) }

    background: Rectangle {
        radius: 28
        color: Theme.surfaceContainerLowest
        clip: true
    }

    contentItem: RowLayout {
        anchors.fill: parent
        spacing: 0

        Item {
            Layout.preferredWidth: root.width < 760 ? 240 : 336
            Layout.fillHeight: true

            Rectangle {
                id: searchPill
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: 16
                height: 48
                radius: 24
                color: Theme.surfaceContainerHigh

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 16
                    anchors.rightMargin: 16
                    spacing: 10

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "Search"
                        size: 20
                        color: Theme.surfaceVariantText
                    }
                    TextInput {
                        id: searchField
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 30
                        color: Theme.surfaceText
                        selectionColor: Theme.alpha(Theme.primary, 0.4)
                        selectedTextColor: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 15
                        clip: true

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: searchField.text === "" && !searchField.activeFocus
                            text: "Search settings"
                            color: Theme.surfaceVariantText
                            font: searchField.font
                        }
                    }
                }
                HoverHandler { cursorShape: Qt.IBeamCursor }
                TapHandler { onTapped: searchField.forceActiveFocus() }
            }

            Flickable {
                anchors.top: searchPill.bottom
                anchors.topMargin: 8
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.leftMargin: 16
                anchors.rightMargin: 12
                anchors.bottomMargin: 12
                contentHeight: navColumn.implicitHeight + 16
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 4 }

                Column {
                    id: navColumn
                    width: parent.width - 4
                    spacing: 16

                    Repeater {
                        model: root.navGroups

                        delegate: Column {
                            required property var modelData
                            readonly property var visibleItems: modelData.items.filter(it => root.matchesSearch(it))
                            visible: visibleItems.length > 0
                            width: parent.width
                            spacing: 2

                            Text {
                                visible: modelData.title !== ""
                                text: modelData.title
                                leftPadding: 14
                                bottomPadding: 8
                                color: Theme.surfaceVariantText
                                font.family: Theme.font
                                font.pixelSize: 13
                                font.weight: Font.Medium
                            }

                            Repeater {
                                model: visibleItems

                                delegate: Rectangle {
                                    required property var modelData
                                    required property int index
                                    readonly property int total: visibleItems.length
                                    readonly property bool active: root.category === modelData.key
                                    readonly property bool first: index === 0
                                    readonly property bool last: index === total - 1

                                    width: parent.width
                                    height: 62
                                    topLeftRadius: active ? 20 : (first ? 20 : 4)
                                    topRightRadius: active ? 20 : (first ? 20 : 4)
                                    bottomLeftRadius: active ? 20 : (last ? 20 : 4)
                                    bottomRightRadius: active ? 20 : (last ? 20 : 4)
                                    color: active ? Theme.secondaryContainer
                                         : navHover.hovered ? Theme.surfaceContainerHigh
                                         : Theme.surfaceContainer
                                    Behavior on color { ColorAnimation { duration: Theme.quick } }

                                    Row {
                                        anchors.fill: parent
                                        anchors.leftMargin: 16
                                        anchors.rightMargin: 16
                                        spacing: 14

                                        Icon {
                                            anchors.verticalCenter: parent.verticalCenter
                                            name: active ? modelData.icon : ("Rounded." + modelData.icon)
                                            size: 22
                                            color: active ? Theme.secondaryContainerText : Theme.surfaceVariantText
                                        }

                                        Column {
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: parent.width - 36
                                            spacing: 2

                                            Text {
                                                width: parent.width
                                                text: modelData.label
                                                color: active ? Theme.secondaryContainerText : Theme.surfaceText
                                                font.family: Theme.font
                                                font.pixelSize: 15
                                                font.weight: Font.Medium
                                                elide: Text.ElideRight
                                            }
                                            Text {
                                                width: parent.width
                                                text: modelData.desc
                                                color: active ? Theme.alpha(Theme.secondaryContainerText, 0.8) : Theme.surfaceVariantText
                                                font.family: Theme.font
                                                font.pixelSize: 12
                                                elide: Text.ElideRight
                                            }
                                        }
                                    }

                                    HoverHandler { id: navHover; cursorShape: Qt.PointingHandCursor }
                                    TapHandler { onTapped: root.category = modelData.key }
                                }
                            }
                        }
                    }

                    Rectangle {
                        width: parent.width
                        height: 54
                        radius: 20
                        color: logoutHover.hovered ? Theme.alpha(Theme.error, 0.14) : Theme.surfaceContainer
                        Behavior on color { ColorAnimation { duration: Theme.quick } }

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16
                            spacing: 14

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: "Logout"
                                size: 22
                                color: Theme.error
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "Log Out"
                                color: Theme.error
                                font.family: Theme.font
                                font.pixelSize: 15
                                font.weight: Font.Medium
                            }
                        }

                        HoverHandler { id: logoutHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                root.close()
                                backend.logout()
                            }
                        }
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.rightMargin: 12
            Layout.bottomMargin: 12
            spacing: 0

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 80

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 28
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.activeTitle
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 28
                    font.weight: Font.Bold
                }

                IconButton {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: "Close"
                    iconSize: 20
                    tip: "Close"
                    onClicked: root.close()
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 28
                color: Theme.surfaceContainerLow
                clip: true

                Flickable {
                    anchors.fill: parent
                    anchors.margins: 20
                    contentHeight: contentCol.implicitHeight + 24
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

                    Column {
                        id: contentCol
                        width: parent.width - 8
                        spacing: 20

                        AccountSettingsPage {
                            width: parent.width
                            visible: root.category === "account"
                            savedAccounts: root.savedAccounts
                            onEditProfileRequested: root.category = "profile"
                            onCloseRequested: root.close()
                        }

                        ProfileSettingsPage {
                            width: parent.width
                            visible: root.category === "profile"
                            myProfile: root.myProfile
                        }

                        AppearanceSettingsPage {
                            width: parent.width
                            visible: root.category === "appearance"
                        }

                        ChatSettingsPage {
                            width: parent.width
                            visible: root.category === "chat"
                        }

                        AdditionalSettingsPage {
                            width: parent.width
                            visible: root.category === "additional"
                        }

                        NotificationsSettingsPage {
                            width: parent.width
                            visible: root.category === "notifications"
                        }

                        Column {
                            width: parent.width
                            spacing: 12
                            visible: ["privacy", "connections", "devices", "accessibility", "language", "voice", "advanced", "updates"].indexOf(root.category) !== -1

                            SettingsGroupTitle { text: root.activeTitle }

                            SettingsInfoRow {
                                title: root.activeTitle
                                subtitle: "Additional options for " + root.activeTitle + " will appear here as backend support expands."
                                first: true
                                last: true
                            }
                        }

                        AboutSettingsPage {
                            width: parent.width
                            visible: root.category === "about"
                        }
                    }
                }
            }
        }
    }
}
