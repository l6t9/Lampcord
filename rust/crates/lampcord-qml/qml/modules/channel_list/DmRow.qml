import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// DMItem: 48px rows inset by 8px with a 32px avatar, small-shape highlight
// when selected.
Item {
    id: root

    property var dm
    property bool selected: false
    property bool mobile: false
    signal clicked()

    implicitHeight: 48

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        radius: 8
        color: root.selected ? Theme.surfaceVariant
             : hover.hovered ? Theme.alpha(Theme.surfaceText, 0.05) : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }

        // Phoenix DM rows are flat; desktop keeps the spotlight nameplate.
        Nameplate {
            anchors.fill: parent
            source: root.dm.nameplate
            shown: !root.mobile && (hover.hovered || root.selected)
        }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 12

            Item {
                anchors.verticalCenter: parent.verticalCenter
                width: 32
                height: 32

                UserAvatar {
                    visible: !root.dm.group
                    size: 32
                    name: root.dm.name
                    source: root.dm.avatar
                    decoration: root.dm.decoration
                    status: root.dm.status
                    ring: root.selected ? Theme.surfaceVariant : Theme.surface
                }
                Rectangle {
                    visible: root.dm.group
                    anchors.fill: parent
                    radius: 16
                    color: Theme.primaryContainer

                    Icon {
                        anchors.centerIn: parent
                        name: "Group"
                        size: 20
                        color: Theme.primaryContainerText
                    }
                }
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 44 - (badge.visible ? badge.width + 12 : 0)

                Row {
                    width: parent.width
                    spacing: 6

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(implicitWidth, parent.width - (tagPill.visible ? tagPill.width + 6 : 0))
                        text: root.dm.name
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.letterSpacing: 0.25
                        elide: Text.ElideRight
                    }

                    Rectangle {
                        id: tagPill
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !!root.dm.system || !!root.dm.bot
                        height: 16
                        width: tagLabel.implicitWidth + 10
                        radius: 4
                        color: root.dm.system ? "#F0B232" : "#5865F2"

                        Text {
                            id: tagLabel
                            anchors.centerIn: parent
                            text: root.dm.system ? "SYSTEM" : "BOT"
                            color: root.dm.system ? "#111214" : "#FFFFFF"
                            font.family: Theme.font
                            font.pixelSize: 9
                            font.weight: Font.Bold
                            font.letterSpacing: 0.3
                        }
                    }
                }
                Text {
                    width: parent.width
                    visible: root.dm.group
                    text: root.dm.members + " Members"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }
            }

            Rectangle {
                id: badge
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm.mentions > 0
                height: 16
                width: Math.max(16, count.implicitWidth + 8)
                radius: 8
                color: Theme.error

                Text {
                    id: count
                    anchors.centerIn: parent
                    text: root.dm.mentions > 99 ? "99+" : root.dm.mentions
                    color: Theme.errorText
                    font.family: Theme.font
                    font.pixelSize: 11
                    font.weight: Font.Bold
                }
            }
        }
    }

    HoverHandler {
        id: hover
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton) {
                dmMenu.popup(mouse.x, mouse.y)
            } else if (mouse.button === Qt.LeftButton) {
                root.clicked()
            }
        }
    }

    LampMenu {
        id: dmMenu

        LampMenuItem {
            text: "Mark as Read"
            leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                window.openDm(root.dm.id)
                backend.ackLatest()
            }
        }
        LampMenuItem {
            visible: !root.dm.group && root.dm.recipientId && root.dm.recipientId !== ""
            text: "Profile"
            leading: Component { Icon { name: "AccountCircle"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                if (window.openProfile) {
                    const pt = root.mapToItem(Overlay.overlay, root.width, 0)
                    window.openProfile(root.dm.recipientId, root.dm.name, root.dm.avatar, root.dm.decoration, root.dm.status, "", pt.x, pt.y)
                }
            }
        }
        MenuSeparator {
            implicitHeight: 9
            height: 9
            padding: 4
            contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
        }
        LampMenuItem {
            text: root.dm.group ? "Leave Group" : "Close DM"
            danger: true
            leading: Component { Icon { name: "Close"; size: 17; color: Theme.error } }
            onTriggered: backend.deleteChannel(root.dm.id)
        }
        MenuSeparator {
            implicitHeight: 9
            height: 9
            padding: 4
            contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
        }
        LampMenuItem {
            text: "Copy ID"
            leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: if (window.copyText) window.copyText(root.dm.id)
        }
    }
}
