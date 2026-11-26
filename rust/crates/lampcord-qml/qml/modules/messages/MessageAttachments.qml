import QtQuick
import Lampcord

Column {
    id: root
    property var files: []
    visible: files.length > 0
    spacing: 2

    function formatSize(bytes) {
        if (bytes < 1024)
            return bytes + " B"
        if (bytes < 1024 * 1024)
            return (bytes / 1024).toFixed(1) + " KB"
        return (bytes / 1024 / 1024).toFixed(1) + " MB"
    }

    Repeater {
        model: root.files

        delegate: Item {
            required property var modelData
            width: root.width
            height: (modelData.image ? image.height
                   : modelData.video ? videoCard.height
                   : modelData.audio ? audioCard.height
                   : card.height) + 4

            // Clamp inside min(400, width) x 300 so images never overflow narrow bubbles.
            readonly property real maxW: Math.min(400, Math.max(120, root.width))
            readonly property real fit: (modelData.width > 0 && modelData.height > 0)
                ? Math.min(1, maxW / modelData.width, 300 / modelData.height) : 1

            RoundedImage {
                id: image
                visible: !!modelData.image
                y: 4
                width: modelData.width > 0 ? Math.round(modelData.width * parent.fit) : Math.min(300, parent.maxW)
                height: modelData.height > 0 ? Math.round(modelData.height * parent.fit) : 200
                radius: 12
                source: modelData.image ? modelData.preview : ""

                Rectangle {
                    anchors.fill: parent
                    radius: 12
                    visible: !image.ready
                    color: Theme.surfaceContainerHigh
                }
                TapHandler {
                    onTapped: {
                        if (window.openImage)
                            window.openImage(modelData.url, modelData.filename)
                        else
                            Qt.openUrlExternally(modelData.url)
                    }
                }
                HoverHandler { cursorShape: Qt.PointingHandCursor }
            }

            Rectangle {
                id: videoCard
                visible: !!modelData.video && !modelData.image
                y: 4
                width: Math.min(root.width, 380)
                height: 210
                radius: 12
                color: "#111114"

                Column {
                    anchors.centerIn: parent
                    spacing: 8

                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 52
                        height: 52
                        radius: 26
                        color: Theme.alpha(Theme.primary, 0.9)

                        Icon {
                            anchors.centerIn: parent
                            name: "PlayArrow"
                            size: 28
                            color: Theme.primaryText
                        }
                    }
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: modelData.filename
                        color: "#FFFFFF"
                        font.family: Theme.font
                        font.pixelSize: 12
                        elide: Text.ElideMiddle
                    }
                }

                HoverHandler { cursorShape: Qt.PointingHandCursor }
                TapHandler {
                    onTapped: {
                        if (window.openVideo)
                            window.openVideo(modelData.url, modelData.filename)
                        else
                            Qt.openUrlExternally(modelData.url)
                    }
                }
            }

            AudioAttachment {
                id: audioCard
                visible: !!modelData.audio && !modelData.image && !modelData.video
                y: 4
                width: Math.min(root.width, 400)
                url: modelData.audio ? modelData.url : ""
                filename: modelData.filename
                bytes: modelData.size
            }

            Rectangle {
                id: card
                visible: !modelData.image && !modelData.video && !modelData.audio
                y: 4
                width: Math.min(root.width, 400)
                height: 64
                radius: 12
                color: Theme.surfaceContainerHigh

                Row {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 12

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "Description"
                        size: 32
                        color: Theme.primary
                    }
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 32 - 40 - 24

                        Text {
                            width: parent.width
                            text: modelData.filename
                            color: Theme.primary
                            font.family: Theme.font
                            font.pixelSize: 14
                            elide: Text.ElideMiddle
                        }
                        Text {
                            text: root.formatSize(modelData.size)
                            color: Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 12
                        }
                    }
                    IconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        glyph: "Download"
                        tip: "Download"
                        onClicked: Qt.openUrlExternally(modelData.url)
                    }
                }
            }
        }
    }
}
