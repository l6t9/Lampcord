import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Rectangle {
    id: slRow
    property string title
    property string label: ""
    property real from: 0
    property real to: 1
    property real value: 0
    property bool first: true
    property bool last: true
    signal moved(real v)

    width: parent ? parent.width : 0
    height: 76
    topLeftRadius: first ? 20 : 4
    topRightRadius: first ? 20 : 4
    bottomLeftRadius: last ? 20 : 4
    bottomRightRadius: last ? 20 : 4
    color: Theme.surfaceContainer

    Column {
        anchors.fill: parent
        anchors.leftMargin: 18
        anchors.rightMargin: 18
        anchors.topMargin: 12
        anchors.bottomMargin: 8
        spacing: 6

        Row {
            width: parent.width
            Text {
                width: parent.width - valLbl.implicitWidth
                text: slRow.title
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 14
                font.weight: Font.Medium
            }
            Text {
                id: valLbl
                text: slRow.label
                color: Theme.primary
                font.family: Theme.font
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
        }

        Slider {
            id: m3Slider
            width: parent.width
            height: 30
            from: slRow.from
            to: slRow.to
            value: slRow.value
            onMoved: slRow.moved(value)

            background: Item {
                x: m3Slider.leftPadding
                y: m3Slider.topPadding + (m3Slider.availableHeight - height) / 2
                width: m3Slider.availableWidth
                height: 14

                Rectangle {
                    x: 0
                    width: Math.max(0, m3Slider.visualPosition * parent.width - 7)
                    height: parent.height
                    topLeftRadius: 7; bottomLeftRadius: 7; topRightRadius: 3; bottomRightRadius: 3
                    color: Theme.primary
                }
                Rectangle {
                    x: Math.min(parent.width, m3Slider.visualPosition * parent.width + 7)
                    width: Math.max(0, parent.width - x)
                    height: parent.height
                    topLeftRadius: 3; bottomLeftRadius: 3; topRightRadius: 7; bottomRightRadius: 7
                    color: Theme.surfaceContainerHighest

                    Rectangle {
                        visible: parent.width > 12
                        anchors.right: parent.right
                        anchors.rightMargin: 5
                        anchors.verticalCenter: parent.verticalCenter
                        width: 4; height: 4; radius: 2
                        color: Theme.primary
                    }
                }
            }

            handle: Rectangle {
                x: m3Slider.leftPadding + m3Slider.visualPosition * m3Slider.availableWidth - width / 2
                y: m3Slider.topPadding + (m3Slider.availableHeight - height) / 2
                width: 4
                height: 28
                radius: 2
                color: Theme.primary
            }
        }
    }
}
