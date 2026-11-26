import QtQuick
import Lampcord

Rectangle {
    id: actRow
    property string icon: ""
    property string title
    property string subtitle: ""
    property bool first: true
    property bool last: true
    signal clicked()

    width: parent ? parent.width : 0
    height: Math.max(56, actCol.implicitHeight + 24)
    topLeftRadius: first ? 20 : 4
    topRightRadius: first ? 20 : 4
    bottomLeftRadius: last ? 20 : 4
    bottomRightRadius: last ? 20 : 4
    color: actHover.hovered ? Theme.surfaceContainerHigh : Theme.surfaceContainer
    Behavior on color { ColorAnimation { duration: Theme.quick } }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 18
        anchors.rightMargin: 18
        spacing: 14

        Icon {
            anchors.verticalCenter: parent.verticalCenter
            visible: actRow.icon !== ""
            name: actRow.icon
            size: 22
            color: Theme.surfaceVariantText
        }
        Column {
            id: actCol
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - (actRow.icon !== "" ? 36 : 0)
            spacing: 2

            Text {
                width: parent.width
                text: actRow.title
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: Font.Medium
                wrapMode: Text.Wrap
            }
            Text {
                width: parent.width
                visible: actRow.subtitle !== ""
                text: actRow.subtitle
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 13
                wrapMode: Text.Wrap
            }
        }
    }

    HoverHandler { id: actHover; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: actRow.clicked() }
}
