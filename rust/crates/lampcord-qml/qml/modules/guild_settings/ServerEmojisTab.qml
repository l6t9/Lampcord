import QtQuick
import Lampcord

Column {
    id: root
    property string guildId: ""
    property var emojis: []

    spacing: 12

    Text {
        text: "Custom Emojis (" + root.emojis.length + ")"
        color: Theme.surfaceText
        font.family: Theme.font
        font.pixelSize: 20
        font.weight: Font.Bold
    }

    Flow {
        width: parent.width
        spacing: 8

        Repeater {
            model: root.emojis

            delegate: Rectangle {
                required property var modelData
                width: 170
                height: 48
                radius: 12
                color: Theme.surfaceContainer

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 4
                    spacing: 8

                    Image {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 28
                        height: 28
                        source: modelData.url
                        fillMode: Image.PreserveAspectFit
                        asynchronous: true
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 76
                        text: ":" + modelData.name + ":"
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }
                    IconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        glyph: "Close"
                        iconSize: 16
                        iconColor: Theme.error
                        tip: "Delete Emoji"
                        onClicked: backend.deleteGuildEmoji(root.guildId, modelData.id)
                    }
                }
            }
        }
    }
}
