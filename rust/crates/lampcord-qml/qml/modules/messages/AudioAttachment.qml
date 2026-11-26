import QtQuick
import QtQuick.Controls.Basic
import QtMultimedia
import Lampcord

Rectangle {
    id: root

    property string url: ""
    property string filename: ""
    property real bytes: 0

    height: 64
    radius: 14
    color: Theme.surfaceContainerHigh

    MediaPlayer {
        id: audio
        source: root.url
        audioOutput: AudioOutput {}
    }

    function fmt(ms) {
        const s = Math.max(0, Math.floor(ms / 1000))
        return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0")
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 10
        anchors.rightMargin: 12
        spacing: 10

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: 40
            height: 40
            radius: 20
            color: Theme.primary

            Icon {
                anchors.centerIn: parent
                name: audio.playbackState === MediaPlayer.PlayingState ? "Pause" : "PlayArrow"
                size: 22
                color: Theme.primaryText
            }
            HoverHandler { cursorShape: Qt.PointingHandCursor }
            TapHandler {
                onTapped: audio.playbackState === MediaPlayer.PlayingState ? audio.pause() : audio.play()
            }
        }

        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 96
            spacing: 2

            Text {
                width: parent.width
                text: root.filename
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 13
                font.weight: Font.DemiBold
                elide: Text.ElideMiddle
            }

            Row {
                width: parent.width
                spacing: 8

                Slider {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - timeText.width - 8
                    from: 0
                    to: Math.max(1, audio.duration)
                    value: audio.position
                    onMoved: audio.position = value
                }

                Text {
                    id: timeText
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.fmt(audio.position) + " / " + root.fmt(audio.duration)
                    color: Theme.surfaceVariantText
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }
            }
        }

        IconButton {
            anchors.verticalCenter: parent.verticalCenter
            glyph: "Download"
            tip: "Download"
            onClicked: Qt.openUrlExternally(root.url)
        }
    }
}
