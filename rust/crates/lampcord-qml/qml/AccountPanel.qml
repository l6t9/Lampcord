import QtQuick
import QtQuick.Controls.Basic

Rectangle {
    id: root

    implicitHeight: 64
    radius: 20
    color: Theme.surfaceContainerHighest

    Row {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 8
        spacing: 10

        Avatar {
            anchors.verticalCenter: parent.verticalCenter
            size: 40
            name: backend.currentUser
            source: backend.currentUserAvatar
        }

        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 40 - 40 - 20

            Text {
                width: parent.width
                text: backend.currentUser || "…"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 14
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
            Text {
                width: parent.width
                text: backend.connection === "ready" ? "Online"
                    : backend.connection === "resuming" ? "Reconnecting…" : "Connecting…"
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 12
                font.weight: Font.Medium
                elide: Text.ElideRight
            }
        }

        IconButton {
            id: settings
            anchors.verticalCenter: parent.verticalCenter
            glyph: "settings"
            iconSize: 20
            tip: "User Settings"
            onClicked: menu.open()

            Menu {
                id: menu
                y: -implicitHeight - 8
                width: 220
                padding: 4

                background: Rectangle {
                    radius: 12
                    color: Theme.surfaceContainer
                    border.color: Theme.outlineVariant
                }

                MenuItem {
                    id: logoutItem
                    text: "Log Out"
                    onTriggered: backend.logout()
                    contentItem: Row {
                        spacing: 12
                        leftPadding: 4
                        Icon { name: "logout"; size: 20; color: Theme.error; anchors.verticalCenter: parent.verticalCenter }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: logoutItem.text
                            color: Theme.error
                            font.family: Theme.font
                            font.pixelSize: 14
                        }
                    }
                    background: Rectangle {
                        radius: 8
                        color: logoutItem.highlighted ? Theme.alpha(Theme.surfaceText, 0.08) : "transparent"
                    }
                }
            }
        }
    }
}
