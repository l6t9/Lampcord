import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// ChannelItem at the default spacing: 36px rows inset by 8px, a small-shape
// highlight when selected and a pill on the left edge for unread channels.
Item {
    id: root

    property var channel
    property bool selected: false
    property bool mobile: false
    signal clicked()

    // Text, announcement, forum, thread, voice and stage channels can all be selected.
    readonly property bool openable: true
    readonly property bool isVoice: channel.kind === 2 || channel.kind === 13
    readonly property bool hasMention: (channel.mentions || 0) > 0
    readonly property bool unread: channel.unread && !selected
    readonly property bool unobtrusive: unread && !hasMention && (!!channel.unobtrusive || Theme.prefs.unobtrusiveUnreads)
    readonly property bool loudUnread: unread && !unobtrusive
    readonly property color tint: selected || loudUnread || hover.hovered
        ? Theme.surfaceText : Theme.surfaceVariantText

    implicitHeight: root.mobile ? 36 : 38
    opacity: root.channel.locked ? 0.52
           : (root.channel.muted && !root.unread && !root.selected) ? 0.45
           : isVoice ? 0.85 : 1

    Rectangle {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        visible: root.unread
        width: 4
        height: root.unobtrusive ? 8 : 10
        topRightRadius: 4
        bottomRightRadius: 4
        color: root.unobtrusive ? Theme.alpha(Theme.surfaceVariantText, 0.55) : Theme.surfaceText
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        anchors.topMargin: 1
        anchors.bottomMargin: 1
        radius: 8
        color: root.selected ? Theme.surfaceContainerHigh
             : hover.hovered && root.openable ? Theme.alpha(Theme.surfaceText, 0.05) : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 10
            spacing: 12

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: root.channel.locked ? "Lock"
                    : ({ 2: "VolumeUp", 13: "Podcasts", 5: "Campaign", 15: "Rounded.Forum" })[root.channel.kind] || "Tag"
                size: root.channel.locked ? 19 : (root.mobile ? 20 : 22)
                color: Theme.alpha(root.tint, root.selected ? 1 : root.mobile ? 0.9 : 0.8)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 36 - (badge.visible ? badge.width + 12 : 0) - (nsfwTag.visible ? nsfwTag.width + 8 : 0)
                text: root.channel.name
                color: root.tint
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: root.loudUnread ? Font.Bold : Font.Medium
                elide: Text.ElideRight
            }

            Rectangle {
                id: nsfwTag
                anchors.verticalCenter: parent.verticalCenter
                visible: !!root.channel.nsfw
                height: 16
                width: 34
                radius: 4
                color: Theme.errorContainer

                Text {
                    anchors.centerIn: parent
                    text: "18+"
                    color: Theme.errorContainerText
                    font.family: Theme.font
                    font.pixelSize: 9
                    font.weight: Font.Bold
                }
            }

            Rectangle {
                id: badge
                anchors.verticalCenter: parent.verticalCenter
                visible: root.channel.mentions > 0
                height: 16
                width: Math.max(16, count.implicitWidth + 8)
                radius: 8
                color: Theme.error

                Text {
                    id: count
                    anchors.centerIn: parent
                    text: root.channel.mentions > 99 ? "99+" : root.channel.mentions
                    color: Theme.errorText
                    font.family: Theme.font
                    font.pixelSize: 11
                    font.weight: Font.Bold
                }
            }
        }
    }

    HoverHandler {
        id: hover
        cursorShape: root.openable ? Qt.PointingHandCursor : Qt.ArrowCursor
    }
    MouseArea {
        anchors.fill: parent
        enabled: root.openable
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton)
                channelMenu.popup(mouse.x, mouse.y)
            else
                root.clicked()
        }
    }

    LampMenu {
        id: channelMenu

        LampMenuItem {
            text: "Mark as Read"
            leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                if (root.selected)
                    backend.ackLatest()
                else if (backend.selectedGuildId !== "")
                    backend.markGuildRead(backend.selectedGuildId)
            }
        }
        LampMenuItem {
            visible: root.channel.kind === 0 || root.channel.kind === 5 || root.channel.kind === 15
            text: "Threads"
            leading: Component { Icon { name: "Rounded.Topic"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                if (!root.selected)
                    root.clicked()
                if (window.toggleThreads && !window.threadPanelVisible)
                    window.toggleThreads()
            }
        }

        MenuSeparator {
            padding: 4
            contentItem: Rectangle {
                implicitHeight: 1
                color: Theme.outlineVariant
            }
        }

        LampMenuItem {
            text: "Copy Link"
            leading: Component { Icon { name: "Link"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                const gid = backend.selectedGuildId !== "" ? backend.selectedGuildId : "@me"
                if (window.copyText)
                    window.copyText("https://discord.com/channels/" + gid + "/" + root.channel.id)
            }
        }
        LampMenuItem {
            text: "Copy ID"
            leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: if (window.copyText) window.copyText(root.channel.id)
        }
    }
}
