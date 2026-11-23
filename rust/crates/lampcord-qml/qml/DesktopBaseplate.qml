import QtQuick
import QtQuick.Layouts

// The Compose DesktopBaseplate: sidebar (rail + channels + account panel)
// and the chat surface, floating as rounded panels with 8px gutters.
Rectangle {
    color: Theme.surfaceContainerHigh

    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 8

        ColumnLayout {
            // Nested layouts fill by default; the sidebar keeps its width.
            Layout.fillWidth: false
            Layout.preferredWidth: parent.width < 900 ? 240 : 312
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
    }
}
