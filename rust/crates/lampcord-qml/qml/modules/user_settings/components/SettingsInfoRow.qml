import QtQuick
import Lampcord

Rectangle {
    id: infoRow
    property string title
    property string subtitle: ""
    property bool first: true
    property bool last: true

    width: parent ? parent.width : 0
    height: Math.max(60, infoCol.implicitHeight + 24)
    topLeftRadius: first ? 20 : 4
    topRightRadius: first ? 20 : 4
    bottomLeftRadius: last ? 20 : 4
    bottomRightRadius: last ? 20 : 4
    color: Theme.surfaceContainer

    Column {
        id: infoCol
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 18
        anchors.rightMargin: 18
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: infoRow.title
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 15
            font.weight: Font.Medium
            wrapMode: Text.Wrap
        }
        Text {
            width: parent.width
            visible: infoRow.subtitle !== ""
            text: infoRow.subtitle
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 13
            wrapMode: Text.Wrap
        }
    }
}
