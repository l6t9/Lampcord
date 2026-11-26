import QtQuick
import Lampcord

ListView {
    id: rail

    property bool mobile: false

    model: window.rail
    spacing: mobile ? 6 : 8
    topMargin: 8
    bottomMargin: 8
    clip: true
    boundsBehavior: Flickable.StopAtBounds

    Rectangle {
        anchors.fill: parent
        z: -1
        color: rail.mobile ? Theme.surfaceContainerLowest : "transparent"
    }

    header: Column {
        width: rail.width
        spacing: rail.mobile ? 6 : 8
        bottomPadding: rail.mobile ? 4 : 8

        GuildItem {
            width: rail.width
            mobile: rail.mobile
            glyph: "discord"
            tip: "Direct Messages"
            selected: backend.selectedGuildId === ""
            onClicked: window.openHome()
        }

        // DMs with pings sit under the home button, like the Compose rail.
        Repeater {
            model: window.dms.filter(dm => dm.mentions > 0)

            delegate: Item {
                required property var modelData
                width: rail.width
                height: 48

                GuildItem {
                    anchors.fill: parent
                    mobile: rail.mobile
                    icon: modelData.avatar
                    label: modelData.name.slice(0, 2)
                    tip: modelData.name
                    selected: backend.selectedChannelId === modelData.id
                    onClicked: window.openDm(modelData.id)
                }
                MentionBadge {
                    x: 12 + 48 - width + 3
                    y: 48 - height + 3
                    count: modelData.mentions
                }
            }
        }
    }

    delegate: Loader {
        required property var modelData
        width: rail.width
        sourceComponent: modelData.kind === "folder" ? folderItem : guildItem

        Component {
            id: guildItem
            GuildItem {
                guildId: modelData.id
                mobile: rail.mobile
                icon: modelData.icon
                label: modelData.initials
                tip: modelData.name
                unread: !!modelData.unread
                mentions: modelData.mentions || 0
                selected: backend.selectedGuildId === modelData.id
                onClicked: window.openGuild(modelData.id)
            }
        }

        Component {
            id: folderItem
            FolderItem {
                mobile: rail.mobile
                folder: modelData
            }
        }
    }

    footer: Item {
        width: rail.width
        height: 60

        GuildItem {
            y: 6
            width: rail.width
            mobile: rail.mobile
            glyph: "Add"
            tip: "Join a Server"
            selected: false
            onClicked: if (window.openJoinServer) window.openJoinServer()
        }
    }
}
