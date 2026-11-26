import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Button {
    id: control

    property alias horizontalAlignment: label.horizontalAlignment

    implicitHeight: 40
    leftPadding: 12
    rightPadding: 12
    hoverEnabled: true
    font.family: Theme.font
    font.pixelSize: 14
    font.weight: Font.Medium
    font.letterSpacing: 0.1

    contentItem: Text {
        id: label
        text: control.text
        font: control.font
        color: Theme.primary
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }

    background: Rectangle {
        radius: height / 2
        color: Theme.primary
        opacity: control.down ? 0.12 : control.hovered ? 0.08 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
