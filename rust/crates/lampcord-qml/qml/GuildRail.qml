import QtQuick

ListView {
    id: rail

    model: window.rail
    spacing: 8
    topMargin: 8
    bottomMargin: 8
    clip: true
    boundsBehavior: Flickable.StopAtBounds

    header: Column {
        width: rail.width
        spacing: 8
        bottomPadding: 8

        GuildItem {
            width: rail.width
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
                icon: modelData.icon
                label: modelData.initials
                tip: modelData.name
                selected: backend.selectedGuildId === modelData.id
                onClicked: window.openGuild(modelData.id)
            }
        }

        Component {
            id: folderItem
            FolderItem {
                folder: modelData
            }
        }
    }
}
