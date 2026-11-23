import QtQuick
import QtQuick.Controls.Basic

// DropdownMenuItem: 48px rows, 12px padding, optional leading item.
MenuItem {
    id: control

    property Component leading
    property bool danger: false

    implicitHeight: 48
    leftPadding: 12
    rightPadding: 12
    hoverEnabled: true

    contentItem: Row {
        spacing: 12

        Loader {
            anchors.verticalCenter: parent.verticalCenter
            active: control.leading !== null
            sourceComponent: control.leading
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: control.text
            color: control.danger ? Theme.error : Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 14
            font.letterSpacing: 0.1
            font.weight: Font.Medium
        }
    }

    background: Rectangle {
        color: Theme.surfaceText
        opacity: control.down ? 0.12 : control.highlighted || control.hovered ? 0.08 : 0
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
