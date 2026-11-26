import QtQuick
import Lampcord

// User avatar: the CDN image when we know it, else a tinted monogram so a
// missing hash never leaves an empty hole in the message list.
Item {
    id: root

    property string source
    property string name
    property real size: 40

    width: size
    height: size

    readonly property var tints: [Theme.primaryContainer, Theme.secondaryContainer, Theme.errorContainer, Theme.surfaceContainerHighest]

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        visible: !image.ready
        color: root.tints[Math.abs(root.hash(root.name)) % root.tints.length]

        Text {
            anchors.centerIn: parent
            text: root.name.length > 0 ? root.name[0].toUpperCase() : "?"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: root.size * 0.42
            font.weight: Font.DemiBold
        }
    }

    RoundedImage {
        id: image
        anchors.fill: parent
        source: root.source
    }

    function hash(text) {
        let h = 0
        for (let i = 0; i < text.length; i++)
            h = (h * 31 + text.charCodeAt(i)) | 0
        return h
    }
}
