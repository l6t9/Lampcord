import QtQuick
import Lampcord

Column {
    id: root
    property string guildId: ""
    property var bans: []

    spacing: 12

    Text {
        text: "Bans (" + root.bans.length + ")"
        color: Theme.surfaceText
        font.family: Theme.font
        font.pixelSize: 20
        font.weight: Font.Bold
    }

    Repeater {
        model: root.bans

        delegate: Rectangle {
            required property var modelData
            width: parent.width
            height: 52
            radius: 12
            color: Theme.surfaceContainer

            Row {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 10
                spacing: 10

                Avatar {
                    anchors.verticalCenter: parent.verticalCenter
                    size: 32
                    name: modelData.username
                    source: modelData.avatar
                }
                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 130
                    Text {
                        text: modelData.username
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                    }
                    Text {
                        visible: modelData.reason !== ""
                        text: modelData.reason
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 11
                        elide: Text.ElideRight
                    }
                }
                TextButton {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Unban"
                    onClicked: backend.unbanMember(root.guildId, modelData.id)
                }
            }
        }
    }

    Text {
        visible: root.bans.length === 0
        text: "No banned users"
        color: Theme.surfaceVariantText
        font.family: Theme.font
        font.pixelSize: 13
    }
}
