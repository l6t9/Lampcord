import QtQuick
import Lampcord

Column {
    id: root
    property string guildId: ""
    property var roles: []

    spacing: 12

    Text {
        text: "Roles (" + root.roles.length + ")"
        color: Theme.surfaceText
        font.family: Theme.font
        font.pixelSize: 20
        font.weight: Font.Bold
    }

    Row {
        width: parent.width
        spacing: 8

        OutlinedField {
            id: newRoleName
            width: parent.width - createRoleBtn.width - 8
            label: "Role name"
            onAccepted: createRoleBtn.clicked()
        }
        FilledButton {
            id: createRoleBtn
            anchors.verticalCenter: newRoleName.verticalCenter
            text: "Create Role"
            enabled: newRoleName.text.trim() !== ""
            onClicked: {
                backend.createGuildRole(root.guildId, newRoleName.text.trim(), 0x5865F2)
                newRoleName.text = ""
            }
        }
    }

    Repeater {
        model: root.roles

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
                spacing: 10

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 12
                    height: 12
                    radius: 6
                    color: modelData.color !== "" ? modelData.color : Theme.outline
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 70
                    text: modelData.name
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    elide: Text.ElideRight
                }
                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: modelData.name !== "@everyone"
                    glyph: "Delete"
                    iconColor: Theme.error
                    tip: "Delete Role"
                    onClicked: backend.deleteGuildRole(root.guildId, modelData.id)
                }
            }
        }
    }
}
