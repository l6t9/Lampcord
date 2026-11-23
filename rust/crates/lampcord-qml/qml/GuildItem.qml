import QtQuick
import QtQuick.Controls.Basic

// RegularGuildItem: 48px squircle that tightens from 24 to 16 radius on
// hover/selection, with the left pill indicator.
Item {
    id: root

    property bool selected: false
    property bool unread: false
    property string icon
    property string label
    property string glyph       // Icon name used instead of initials
    property string tip

    signal clicked()

    implicitWidth: 72
    implicitHeight: 48

    readonly property bool hovered: hover.hovered
    readonly property bool active: selected || hovered
    readonly property bool monogram: icon === "" || !image.ready

    Rectangle {
        id: indicator
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        width: 4
        height: parent.height * (root.selected ? 0.8 : root.hovered ? 0.4 : root.unread ? 0.15 : 0)
        topRightRadius: 4
        bottomRightRadius: 4
        color: Theme.surfaceText
        opacity: root.selected || root.hovered || root.unread ? 1 : 0

        Behavior on height { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutBack; easing.overshoot: 2 } }
        Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    }

    Item {
        id: tile
        anchors.centerIn: parent
        width: 48
        height: 48

        property real radius: root.active ? 16 : 24
        Behavior on radius { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutBack; easing.overshoot: 2.5 } }

        Rectangle {
            anchors.fill: parent
            visible: root.monogram
            radius: tile.radius
            color: root.active ? Theme.primary : Theme.surfaceVariant
            Behavior on color { ColorAnimation { duration: Theme.quick } }

            Icon {
                anchors.centerIn: parent
                visible: root.glyph !== ""
                name: root.glyph
                size: 28
                color: root.active ? Theme.primaryText : Theme.primary
            }

            Text {
                anchors.centerIn: parent
                width: parent.width - 6
                visible: root.glyph === ""
                text: root.label
                color: root.active ? Theme.primaryText : Theme.primary
                font.family: Theme.font
                font.pixelSize: text.length > 2 ? 13 : 16
                font.weight: Font.DemiBold
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }
        }

        RoundedImage {
            id: image
            anchors.fill: parent
            radius: tile.radius
            source: root.icon
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        onTapped: root.clicked()
    }

    ToolTip {
        visible: root.hovered && root.tip !== ""
        delay: 300
        x: root.width + 4
        y: (root.height - height) / 2
        text: root.tip
        contentItem: Text {
            text: root.tip
            color: Theme.surface
            font.family: Theme.font
            font.pixelSize: 14
            font.weight: Font.DemiBold
        }
        background: Rectangle {
            radius: 8
            color: Theme.surfaceText
        }
    }
}
