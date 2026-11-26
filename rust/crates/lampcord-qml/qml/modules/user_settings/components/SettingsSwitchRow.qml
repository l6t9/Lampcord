import QtQuick
import Lampcord

Rectangle {
    id: swRow
    property string title
    property string subtitle: ""
    property bool checked: false
    property bool first: true
    property bool last: true
    signal toggled(bool value)

    width: parent ? parent.width : 0
    height: Math.max(64, swCol.implicitHeight + 24)
    topLeftRadius: first ? 20 : 4
    topRightRadius: first ? 20 : 4
    bottomLeftRadius: last ? 20 : 4
    bottomRightRadius: last ? 20 : 4
    color: Theme.surfaceContainer

    Column {
        id: swCol
        anchors.left: parent.left
        anchors.leftMargin: 18
        anchors.right: swPill.left
        anchors.rightMargin: 12
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: swRow.title
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 15
            font.weight: Font.Medium
            wrapMode: Text.Wrap
        }
        Text {
            width: parent.width
            visible: swRow.subtitle !== ""
            text: swRow.subtitle
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 13
            wrapMode: Text.Wrap
        }
    }

    Rectangle {
        id: swPill
        anchors.right: parent.right
        anchors.rightMargin: 18
        anchors.verticalCenter: parent.verticalCenter
        width: 50
        height: 30
        radius: 15
        color: swRow.checked ? Theme.primary : Theme.surfaceContainerHighest
        border.width: swRow.checked ? 0 : 1
        border.color: Theme.outline
        Behavior on color { ColorAnimation { duration: Theme.quick } }

        Rectangle {
            width: swRow.checked ? 22 : 16
            height: width
            radius: width / 2
            anchors.verticalCenter: parent.verticalCenter
            x: swRow.checked ? parent.width - width - 4 : 6
            color: swRow.checked ? Theme.primaryText : Theme.outline
            Behavior on x { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }
        }

        HoverHandler { cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: swRow.toggled(!swRow.checked) }
    }
}
