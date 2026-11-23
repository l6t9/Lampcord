import QtQuick
import QtQuick.Controls.Basic

Item {
    id: root

    readonly property var guild: window.guilds.find(g => g.id === backend.selectedGuildId) || null
    property var collapsed: ({})

    // Categories the user folded away hide their channels.
    readonly property var rows: {
        const out = []
        let hidden = false
        for (const row of window.channels) {
            if (row.category) {
                hidden = !!collapsed[row.id]
                out.push(row)
            } else if (!hidden || row.id === backend.selectedChannelId) {
                out.push(row)
            }
        }
        return out
    }

    Rectangle {
        id: header
        width: parent.width
        height: 48
        // The panel's clip is rectangular, so the header rounds itself.
        topLeftRadius: 16
        topRightRadius: 16
        color: Theme.surface
        z: 1

        Row {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 16
            spacing: 4

            Text {
                width: parent.width - 20
                anchors.verticalCenter: parent.verticalCenter
                text: root.guild ? root.guild.name : "Direct Messages"
                color: Theme.surfaceText
                font.family: Theme.font
                // DM header is titleMedium, the server header titleSmall.
                font.pixelSize: root.guild ? 14 : 16
                font.letterSpacing: root.guild ? 0.1 : 0.15
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.guild !== null
                name: "expand"
                size: 16
            }
        }
    }

    ListView {
        id: list
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        width: parent.width
        visible: root.guild !== null
        model: root.rows
        bottomMargin: 8
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: Loader {
            required property var modelData
            width: list.width
            sourceComponent: modelData.category ? categoryRow : channelRow

            Component {
                id: categoryRow
                CategoryRow {
                    name: modelData.name
                    collapsed: !!root.collapsed[modelData.id]
                    onToggled: {
                        const next = Object.assign({}, root.collapsed)
                        next[modelData.id] = !next[modelData.id]
                        root.collapsed = next
                    }
                }
            }

            Component {
                id: channelRow
                ChannelRow {
                    channel: modelData
                    selected: backend.selectedChannelId === modelData.id
                    onClicked: window.openChannel(modelData.id)
                }
            }
        }
    }

    ListView {
        id: dmList
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        width: parent.width
        visible: root.guild === null
        model: window.dms
        topMargin: 8
        bottomMargin: 8
        spacing: 2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: DmRow {
            required property var modelData
            width: dmList.width
            dm: modelData
            selected: backend.selectedChannelId === modelData.id
            onClicked: window.openDm(modelData.id)
        }
    }
}
