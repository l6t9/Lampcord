import QtQuick

// MemberItem: 44px rows inset by 4px, a 32px avatar with presence, the
// name and the first activity. Offline members fade until hovered.
Item {
    id: root

    property var member
    property bool mobile: false

    readonly property bool offline: member.status === "offline" || member.status === "invisible"

    implicitHeight: 44
    opacity: offline && !hover.hovered ? 0.4 : 1

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 4
        anchors.rightMargin: 4
        radius: 8
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

                Text {
                    width: parent.width
                    text: root.member.name
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.letterSpacing: 0.25
                    elide: Text.ElideRight
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
}
