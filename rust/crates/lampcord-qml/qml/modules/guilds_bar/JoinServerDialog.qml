import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Popup {
    id: root

    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 24
    width: Math.min(420, parent ? parent.width - 32 : 420)
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    onOpened: {
        inviteField.text = ""
        inviteField.forceActiveFocus()
    }

    enter: Transition {
        ParallelAnimation {
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.quick }
            NumberAnimation { property: "scale"; from: 0.94; to: 1; duration: Theme.quick; easing.type: Easing.OutCubic }
        }
    }

    Overlay.modal: Rectangle { color: Theme.alpha("#000000", 0.5) }

    background: Rectangle {
        radius: 24
        color: Theme.surfaceContainer
        border.width: 1
        border.color: Theme.outlineVariant
    }

    contentItem: Column {
        spacing: 14

        Text {
            text: "Join a Server"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 20
            font.weight: Font.Bold
        }

        Text {
            width: parent.width
            text: "Enter an invite link or code below to join an existing server."
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 13
            wrapMode: Text.Wrap
        }

        OutlinedField {
            id: inviteField
            width: parent.width
            label: "Invite Link"
            placeholderText: "https://discord.gg/hTKzmak"
            onAccepted: {
                if (text.trim() !== "") {
                    backend.joinGuild(text.trim())
                    root.close()
                }
            }
        }

        Row {
            anchors.right: parent.right
            spacing: 8

            TextButton {
                text: "Cancel"
                onClicked: root.close()
            }
            FilledButton {
                text: "Join Server"
                enabled: inviteField.text.trim() !== ""
                onClicked: {
                    backend.joinGuild(inviteField.text.trim())
                    root.close()
                }
            }
        }
    }
}
