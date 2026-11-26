import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// MemberItem: 44px rows inset by 4px, a 32px avatar with presence, the
// name and the first activity. Offline members fade until hovered.
Item {
    id: root

    property var member
    property bool mobile: false

    readonly property bool offline: member.status === "offline" || member.status === "invisible"

    implicitHeight: 48
    opacity: offline && !hover.hovered ? 0.4 : 1

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        radius: 10
        color: hover.hovered ? Theme.alpha(Theme.surfaceText, 0.05) : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }

        Nameplate {
            anchors.fill: parent
            source: root.member.nameplate
            shown: !root.mobile
        }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 12

            UserAvatar {
                anchors.verticalCenter: parent.verticalCenter
                size: 32
                name: root.member.name
                source: root.member.avatar
                decoration: root.member.decoration
                status: root.member.status
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 44

                Row {
                    width: parent.width
                    spacing: 5

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(implicitWidth, parent.width - (ownerCrown.visible ? 20 : 0) - (clanPill.visible ? clanPill.width + 6 : 0))
                        text: root.member.name
                        color: root.member.roleColor && root.member.roleColor !== "" ? root.member.roleColor : Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.letterSpacing: 0.25
                        elide: Text.ElideRight
                    }
                    // Guild / Clan Tag badge (e.g. NAMA, MRC, CLEM)
                    Rectangle {
                        id: clanPill
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !!(root.member.clanTag && root.member.clanTag !== "")
                        height: 18
                        width: clanRow.implicitWidth + 10
                        radius: 6
                        color: Theme.surfaceContainerHigh
                        border.width: 1
                        border.color: Theme.alpha(Theme.outlineVariant, 0.5)

                        Row {
                            id: clanRow
                            anchors.centerIn: parent
                            spacing: 4

                            Image {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: !!(root.member.clanBadge && root.member.clanBadge !== "")
                                width: 12
                                height: 12
                                source: root.member.clanBadge || ""
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: root.member.clanTag || ""
                                color: Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 10
                                font.weight: Font.Bold
                            }
                        }
                    }
                    Icon {
                        id: ownerCrown
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !!root.member.owner
                        name: "Crown"
                        size: 14
                        color: "#F9A825"
                    }
                }
                Text {
                    width: parent.width
                    visible: text !== ""
                    text: root.member.activity
                    color: Theme.surfaceVariantText
                    opacity: 0.7
                    font.family: Theme.font
                    font.pixelSize: 11
                    font.weight: Font.Medium
                    elide: Text.ElideRight
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
                userMenu.popup(mouse.x, mouse.y)
            } else if (mouse.button === Qt.LeftButton) {
                if (window.openProfile) {
                    const pt = mapToItem(Overlay.overlay, 0, 0)
                    window.openProfile(root.member.id, root.member.name, root.member.avatar, root.member.decoration, root.member.status, root.member.activity, pt.x, pt.y)
                }
            }
        }
    }

    UserContextMenu {
        id: userMenu
        userId: root.member.id
        userName: root.member.name
        userAvatar: root.member.avatar
        userDecoration: root.member.decoration
        userStatus: root.member.status
        userActivity: root.member.activity
    }
}
