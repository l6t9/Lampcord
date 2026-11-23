import QtQuick
import QtQuick.Effects

// An image clipped to a (possibly animated) rounded rectangle.
Item {
    id: root

    property alias source: image.source
    property real radius: width / 2
    readonly property bool ready: image.status === Image.Ready

    Image {
        id: image
        anchors.fill: parent
        visible: false
        asynchronous: true
        cache: true
        fillMode: Image.PreserveAspectCrop
        sourceSize: Qt.size(root.width * 2, root.height * 2)
    }

    Rectangle {
        id: mask
        anchors.fill: parent
        radius: root.radius
        visible: false
        layer.enabled: true
        antialiasing: true
    }

    MultiEffect {
        anchors.fill: parent
        visible: root.ready
        source: image
        maskEnabled: true
        maskSource: mask
        maskThresholdMin: 0.5
        maskSpreadAtMin: 1.0
    }
}
