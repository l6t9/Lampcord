import QtQuick

Item {
    id: root

    property string name
    property bool collapsed: false
    signal toggled()

    implicitHeight: 12 + 20 + 4

    readonly property color tint: hover.hovered ? Theme.surfaceText : Theme.surfaceVariantText

    Row {
        x: 16
        y: 12
        height: 20
        spacing: 4

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.name
            color: root.tint
            font.family: Theme.font
            font.pixelSize: 12
            font.weight: Font.Bold
            font.letterSpacing: 0.5
        }
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            name: root.collapsed ? "ChevronRight" : "KeyboardArrowDown"
            size: 16
            color: root.tint
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }
    TapHandler {
        onTapped: root.toggled()
    }
}
