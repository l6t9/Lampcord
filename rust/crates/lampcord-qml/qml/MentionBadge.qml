import QtQuick

// Red count pill with the surface-coloured ring the rail badges use.
Rectangle {
    property int count

    height: 20
    width: Math.max(20, label.implicitWidth + 10)
    radius: 10
    color: Theme.error
    border.width: 2
    border.color: Theme.surface
    visible: count > 0

    Text {
        id: label
        anchors.centerIn: parent
        text: parent.count > 99 ? "99+" : parent.count
        color: Theme.errorText
        font.family: Theme.font
        font.pixelSize: 11
        font.weight: Font.ExtraBold
    }
}
