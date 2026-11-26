import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Column {
    id: root
    property string targetUserId: ""
    property string displayName: ""
    property string username: ""
    property string avatarUrl: ""
    property string decorationUrl: ""
    property string bannerUrl: ""
    property string status: "offline"
    property string activity: ""
    property bool isMe: false
    property var resolvedTheme: ({})

    spacing: 0

    Item {
        width: parent.width
        height: 105

        Rectangle {
            anchors.fill: parent
            color: root.resolvedTheme.bannerColor
            topLeftRadius: 12
            topRightRadius: 12
        }

        RoundedImage {
            anchors.fill: parent
            visible: root.bannerUrl !== ""
            source: root.bannerUrl
            radius: 12
        }

        Rectangle {
            id: moreBtn
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: 10
            width: 30
            height: 30
            radius: 15
            color: Theme.alpha("#000000", moreHover.hovered ? 0.55 : 0.4)

            Icon {
                anchors.centerIn: parent
                name: "MoreHoriz"
                size: 18
                color: "#FFFFFF"
            }

            HoverHandler { id: moreHover; cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: moreMenu.popup(moreBtn.width - moreMenu.width, moreBtn.height + 4) }

            LampMenu {
                id: moreMenu

                LampMenuItem {
                    visible: !root.isMe && root.username !== ""
                    text: "Add Friend"
                    leading: Component { Icon { name: "PersonAdd"; size: 17; color: Theme.surfaceVariantText } }
                    onTriggered: backend.sendFriendRequest(root.username)
                }
                LampMenuItem {
                    visible: !root.isMe
                    text: "Block"
                    danger: true
                    leading: Component { Icon { name: "Block"; size: 17; color: Theme.error } }
                    onTriggered: backend.blockUser(root.targetUserId)
                }
                LampMenuItem {
                    text: "Copy User ID"
                    leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
                    onTriggered: if (window.copyText) window.copyText(root.targetUserId)
                }
            }
        }
    }

    Item {
        width: parent.width
        height: 46
        z: 2

        Rectangle {
            x: 10
            y: -49
            width: 94
            height: 94
            radius: 47
            color: root.resolvedTheme.cutout

            UserAvatar {
                anchors.centerIn: parent
                size: 82
                name: root.displayName
                source: root.avatarUrl
                decoration: root.decorationUrl
                status: root.status
                ring: root.resolvedTheme.cutout
            }
        }

        Item {
            visible: root.activity !== ""
            x: 92
            y: -42
            width: parent.width - 104
            height: 44

            Rectangle {
                x: 4
                y: 12
                width: 10
                height: 10
                radius: 5
                color: Theme.surfaceContainerLow
            }
            Rectangle {
                x: 16
                y: 16
                width: 20
                height: 20
                radius: 10
                color: Theme.surfaceContainerLow
            }
            Rectangle {
                x: 24
                y: 22
                width: Math.min(parent.width - 24, thoughtText.implicitWidth + 22)
                height: thoughtText.implicitHeight + 12
                radius: 12
                color: Theme.surfaceContainerLow

                Text {
                    id: thoughtText
                    anchors.centerIn: parent
                    width: Math.min(implicitWidth, parent.parent.width - 46)
                    text: root.activity
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 12
                    elide: Text.ElideRight
                    maximumLineCount: 2
                    wrapMode: Text.Wrap
                }
            }
        }
    }
}
