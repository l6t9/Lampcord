import QtQuick
import QtQuick.Layouts

// The Compose DesktopBaseplate: sidebar (rail + channels + account panel),
// the chat surface and the member panel, floating as rounded panels with
// 8px gutters. Widths follow the M3 window size classes Compose uses.
Rectangle {
    id: root

    color: Theme.surfaceContainerHigh

    readonly property bool medium: width < 840
    readonly property bool extraLarge: width >= 1600

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
            }
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: root.extraLarge ? 300 : 240
            visible: !root.medium && backend.selectedChannelId !== "" && memory.memberPanel
            radius: 16
            color: Theme.surface
            clip: true

            MemberPanel {
                anchors.fill: parent
            }
        }
    }
}
