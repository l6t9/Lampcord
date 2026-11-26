import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Compact M3 ContextMenuItem: 40px rows, 14px horizontal padding,
// M3 segmented hover highlight (12px outer top/bottom corners, 4px inner corners).
MenuItem {
    id: control

    property Component leading
    property bool danger: false
    property bool first: false
    property bool last: false

    readonly property bool isFirst: first || (menu !== null && menu.count > 0 && menu.itemAt(0) === control)
    readonly property bool isLast: last || (menu !== null && menu.count > 0 && menu.itemAt(menu.count - 1) === control)

    implicitHeight: visible ? 40 : 0
    height: visible ? 40 : 0
    leftPadding: 14
    rightPadding: 14
    topPadding: 0
    bottomPadding: 0
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
            font.pixelSize: 13
            font.letterSpacing: 0.1
            font.weight: Font.Medium
        }
    }

    background: Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        topLeftRadius: control.isFirst ? 12 : 4
        topRightRadius: control.isFirst ? 12 : 4
        bottomLeftRadius: control.isLast ? 12 : 4
        bottomRightRadius: control.isLast ? 12 : 4
        color: control.danger ? Theme.error : Theme.surfaceText
        opacity: control.down ? 0.14 : control.highlighted || control.hovered ? 0.08 : 0
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
