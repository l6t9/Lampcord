import QtQuick
import QtQuick.Controls.Basic
import QtMultimedia
import Lampcord

Popup {
    id: root

    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 0
    width: parent ? parent.width : 800
    height: parent ? parent.height : 600
    x: 0
    y: 0

    property string mediaUrl: ""
    property string filename: ""
    property bool isVideo: false
    property real zoom: 1.0

    function openImage(url, name) {
        isVideo = false
        mediaUrl = url
        filename = name || ""
        zoom = 1.0
        open()
    }

    function openVideo(url, name) {
        isVideo = true
        mediaUrl = url
        filename = name || ""
        zoom = 1.0
        open()
        player.play()
    }

    onClosed: {
        player.stop()
        mediaUrl = ""
    }

    Overlay.modal: Rectangle { color: Theme.alpha("#000000", 0.84) }
    background: Item {}

    contentItem: Item {
        anchors.fill: parent

        TapHandler { onTapped: root.close() }

        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 56
            z: 5
            color: Theme.alpha("#000000", 0.55)

            Row {
                anchors.left: parent.left
                anchors.leftMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                spacing: 8

                Text {
                    text: root.filename
                    color: "#FFFFFF"
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.weight: Font.DemiBold
                }
            }

            Row {
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6

                IconButton {
                    visible: !root.isVideo
                    glyph: "Remove"
                    iconColor: "#FFFFFF"
                    tip: "Zoom Out"
                    onClicked: root.zoom = Math.max(0.5, root.zoom - 0.25)
                }
                IconButton {
                    visible: !root.isVideo
                    glyph: "Add"
                    iconColor: "#FFFFFF"
                    tip: "Zoom In"
                    onClicked: root.zoom = Math.min(4.0, root.zoom + 0.25)
                }
                IconButton {
                    glyph: "Download"
                    iconColor: "#FFFFFF"
                    tip: "Open / Download"
                    onClicked: Qt.openUrlExternally(root.mediaUrl)
                }
                IconButton {
                    glyph: "Close"
                    iconColor: "#FFFFFF"
                    tip: "Close"
                    onClicked: root.close()
                }
            }
        }

        Item {
            anchors.fill: parent
            anchors.topMargin: 60
            anchors.bottomMargin: 24
            visible: !root.isVideo

            Image {
                anchors.centerIn: parent
                width: Math.min(parent.width - 48, sourceSize.width > 0 ? sourceSize.width : parent.width - 48) * root.zoom
                height: Math.min(parent.height - 32, sourceSize.height > 0 ? sourceSize.height : parent.height - 32) * root.zoom
                source: !root.isVideo ? root.mediaUrl : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                smooth: true
                Behavior on width { NumberAnimation { duration: 120 } }
                Behavior on height { NumberAnimation { duration: 120 } }
            }
        }

        Item {
            anchors.fill: parent
            anchors.topMargin: 60
            anchors.bottomMargin: 72
            visible: root.isVideo

            MediaPlayer {
                id: player
                source: root.isVideo ? root.mediaUrl : ""
                videoOutput: videoOut
                audioOutput: AudioOutput {}
            }

            VideoOutput {
                id: videoOut
                anchors.fill: parent
                anchors.margins: 16
                fillMode: VideoOutput.PreserveAspectFit
            }
        }

        Rectangle {
            visible: root.isVideo
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottomMargin: 18
            width: Math.min(560, parent.width - 32)
            height: 48
            radius: 24
            color: Theme.alpha("#18181B", 0.9)
            z: 5

            Row {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 16
                spacing: 8

                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: player.playbackState === MediaPlayer.PlayingState ? "Pause" : "PlayArrow"
                    iconColor: "#FFFFFF"
                    tip: player.playbackState === MediaPlayer.PlayingState ? "Pause" : "Play"
                    onClicked: player.playbackState === MediaPlayer.PlayingState ? player.pause() : player.play()
                }

                Slider {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 130
                    from: 0
                    to: Math.max(1, player.duration)
                    value: player.position
                    onMoved: player.position = value
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: Math.floor(player.position / 60000) + ":" + String(Math.floor((player.position % 60000) / 1000)).padStart(2, "0")
                    color: "#FFFFFF"
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                }
            }
        }
    }
}
