import QtQuick
import Lampcord

Column {
    id: root
    property string guildId: ""
    property int newChannelKind: 0

    spacing: 12

    Text {
        text: "Channels"
        color: Theme.surfaceText
        font.family: Theme.font
        font.pixelSize: 20
        font.weight: Font.Bold
    }

    Flow {
        width: parent.width
        spacing: 6
        Repeater {
            model: [
                { kind: 0, label: "Text" },
                { kind: 2, label: "Voice" },
                { kind: 5, label: "Announcement" },
                { kind: 13, label: "Stage" },
                { kind: 15, label: "Forum" },
                { kind: 4, label: "Category" }
            ]
            delegate: Rectangle {
                required property var modelData
                readonly property bool active: root.newChannelKind === modelData.kind
                width: kLabel.implicitWidth + 20
                height: 30
                radius: 15
                color: active ? Theme.primaryContainer : Theme.surfaceContainer
                Text {
                    id: kLabel
                    anchors.centerIn: parent
                    text: modelData.label
                    color: active ? Theme.primaryContainerText : Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: active ? Font.DemiBold : Font.Normal
                }
                HoverHandler { cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: root.newChannelKind = modelData.kind }
            }
        }
    }

    Row {
        width: parent.width
        spacing: 8

        OutlinedField {
            id: newChanName
            width: parent.width - createChanBtn.width - 8
            label: "new-channel-name"
            onAccepted: createChanBtn.clicked()
        }
        FilledButton {
            id: createChanBtn
            anchors.verticalCenter: newChanName.verticalCenter
            text: "Create"
            enabled: newChanName.text.trim() !== ""
            onClicked: {
                backend.createChannel(root.guildId, newChanName.text.trim(), root.newChannelKind)
                newChanName.text = ""
            }
        }
    }

    Repeater {
        model: window.channels

        delegate: Rectangle {
            required property var modelData
            width: parent.width
            height: 42
            radius: 12
            color: Theme.surfaceContainer

            Row {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 8
                spacing: 8

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: modelData.category ? "Folder"
                        : modelData.kind === 2 ? "VolumeUp"
                        : modelData.kind === 13 ? "Podcasts"
                        : modelData.kind === 15 ? "Forum"
                        : modelData.kind === 5 ? "Campaign" : "Tag"
                    size: 18
                    color: Theme.surfaceVariantText
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 72
                    text: modelData.name
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 13
                    elide: Text.ElideRight
                }
                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: "Delete"
                    iconColor: Theme.error
                    tip: "Delete Channel"
                    onClicked: backend.deleteChannel(modelData.id)
                }
            }
        }
    }
}
