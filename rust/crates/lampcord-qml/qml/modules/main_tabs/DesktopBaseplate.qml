import QtQuick
import QtQuick.Layouts
import Lampcord

// The Compose DesktopBaseplate: sidebar (rail + channels + account panel),
// the chat/friends surface and the member panel, floating as rounded panels
// with 8px gutters. Widths follow the M3 window size classes Compose uses.
Rectangle {
    id: root

    color: Theme.surfaceContainerHigh

    readonly property bool medium: width < 840
    readonly property bool extraLarge: width >= 1600
    readonly property bool showMembers: !root.medium && !window.showingFriends && backend.selectedChannelId !== "" && memory.memberPanel

    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 8

        ColumnLayout {
            // Nested layouts fill by default; the sidebar keeps its width.
            Layout.fillWidth: false
            Layout.preferredWidth: root.medium ? 240 : 312
            Layout.fillHeight: true
            spacing: 8

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                GuildRail {
                    Layout.preferredWidth: 72
                    Layout.fillHeight: true
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 16
                    color: Theme.surface
                    clip: true

                    ChannelList {
                        anchors.fill: parent
                    }
                }
            }

            AccountPanel {
                Layout.fillWidth: true
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: 16
            color: Theme.surface
            clip: true

            ChatView {
                anchors.fill: parent
                visible: !window.showingFriends
            }

            FriendsView {
                anchors.fill: parent
                visible: window.showingFriends
                onOpenProfile: user => window.openProfile(user.id, user.name, user.avatar, user.decoration, user.status, user.activity)
            }
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: (!root.medium && !window.showingFriends && backend.selectedChannelId !== "" && window.threadPanelVisible) ? 340 : 0
            visible: Layout.preferredWidth > 0
            radius: 16
            color: Theme.surface
            clip: true
            Behavior on Layout.preferredWidth { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }

            ThreadPanel {
                anchors.fill: parent
                onCloseRequested: window.threadPanelVisible = false
            }
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: root.showMembers ? (root.extraLarge ? 300 : 240) : 0
            visible: Layout.preferredWidth > 0
            radius: 16
            color: Theme.surface
            clip: true
            Behavior on Layout.preferredWidth { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }

            MemberPanel {
                anchors.fill: parent
            }
        }
    }
}
