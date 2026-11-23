import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects

// Pinned messages for the open channel, dropped under the pin button.
Popup {
    id: root

    readonly property var pins: JSON.parse(backend.pinsJson)

    width: 420
    height: Math.min(520, header.height + Math.max(list.contentHeight, 120) + 16)
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.quick }
        NumberAnimation { property: "scale"; from: 0.94; to: 1; duration: Theme.medium; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: Theme.quick }
    }
    transformOrigin: Item.TopRight

    background: Item {
        RectangularShadow {
            anchors.fill: card
            radius: card.radius
            blur: 24
            offset.y: 6
            color: Qt.rgba(0, 0, 0, 0.35)
        }
        Rectangle {
            id: card
            anchors.fill: parent
            radius: 12
            color: Theme.surfaceContainerHigh
        }
    }

    contentItem: Item {
        Text {
            id: header
            width: parent.width
            padding: 16
            text: "Pinned Messages"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 16
            font.letterSpacing: 0.15
            font.weight: Font.DemiBold
        }

        ListView {
            id: list
            anchors.top: header.bottom
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 8
            width: parent.width
            clip: true
            spacing: 8
            model: root.pins
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

            delegate: Rectangle {
                required property var modelData
                x: 8
                width: list.width - 16
                height: message.implicitHeight + 8
                radius: 12
                color: Theme.surfaceContainer

                MessageDelegate {
                    id: message
                    y: 2
                    width: parent.width
                    author: modelData.author
                    avatar: modelData.avatar
                    content: modelData.content
                    time: modelData.time
                    edited: modelData.edited
                    grouped: false
                    system: modelData.system
                    kind: modelData.kind
                    attachments: modelData.attachments
                    replyAuthor: modelData.replyAuthor
                    replyAvatar: modelData.replyAvatar
                    replyContent: modelData.replyContent
                }
            }
        }

        Text {
            anchors.centerIn: list
            visible: root.pins.length === 0
            text: "No pinned messages"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
        }
    }
}
