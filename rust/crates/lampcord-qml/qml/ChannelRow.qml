import QtQuick

// ChannelItem at the default spacing: 36px rows inset by 8px, a small-shape
// highlight when selected and a pill on the left edge for unread channels.
Item {
    id: root

    property var channel
    property bool selected: false
    property bool mobile: false
    signal clicked()

    // Only text-like channels can be opened until voice and forums land.
    readonly property bool openable: channel.kind === 0 || channel.kind === 5
    readonly property bool unread: channel.unread && !selected
    readonly property color tint: selected || unread || (hover.hovered && openable)
        ? Theme.surfaceText : Theme.surfaceVariantText

    implicitHeight: root.mobile ? 36 : 38
    opacity: openable ? 1 : 0.5

    Rectangle {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        visible: root.unread
        width: 4
        height: 12
        topRightRadius: 4
        bottomRightRadius: 4
        color: Theme.surfaceText
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        anchors.topMargin: 1
        anchors.bottomMargin: 1
        radius: 8
        color: root.selected ? Theme.surfaceContainerHigh
             : hover.hovered && root.openable ? Theme.alpha(Theme.surfaceText, 0.05) : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            spacing: 12

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: ({ 2: "VolumeUp", 13: "VolumeUp", 5: "Campaign", 15: "Rounded.Forum" })[root.channel.kind] || "Tag"
                size: root.mobile ? 20 : 24
                color: Theme.alpha(root.tint, root.selected ? 1 : root.mobile ? 0.9 : 0.8)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 36 - (badge.visible ? badge.width + 12 : 0)
                text: root.channel.name
                color: root.tint
                font.family: Theme.font
                font.pixelSize: 16
                font.weight: root.unread ? Font.Bold : Font.Medium
                elide: Text.ElideRight
            }

            Rectangle {
                id: badge
                anchors.verticalCenter: parent.verticalCenter
                visible: root.channel.mentions > 0
                height: 16
                width: Math.max(16, count.implicitWidth + 8)
                radius: 8
                color: Theme.error

                Text {
                    id: count
                    anchors.centerIn: parent
                    text: root.channel.mentions > 99 ? "99+" : root.channel.mentions
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
        cursorShape: root.openable ? Qt.PointingHandCursor : Qt.ArrowCursor
    }
    TapHandler {
        enabled: root.openable
        onTapped: root.clicked()
    }
}
