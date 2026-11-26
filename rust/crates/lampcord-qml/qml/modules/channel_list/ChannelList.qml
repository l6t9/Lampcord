import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    property bool mobile: false

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
        anchors.fill: parent
        visible: root.mobile
        color: Theme.surface
    }

    Rectangle {
        id: header
        width: parent.width
        height: root.mobile ? 56 : 48
        topLeftRadius: root.mobile ? 0 : 16
        topRightRadius: root.mobile ? 0 : 16
        color: root.mobile ? Theme.surface : "transparent"
        z: 1

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            visible: root.mobile
            color: Theme.outlineVariant
        }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: root.guild !== null ? 16 : 8
            spacing: 4

            Text {
                width: parent.width - (root.guild !== null ? 20 : (!root.mobile ? 36 : 0))
                anchors.verticalCenter: parent.verticalCenter
                text: root.guild ? root.guild.name : "Direct Messages"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: root.mobile ? 16 : root.guild ? 14 : 16
                font.letterSpacing: root.mobile ? 0 : root.guild ? 0.1 : 0.15
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.guild !== null
                name: serverMenu.visible ? "Close" : "KeyboardArrowDown"
                size: 16
                color: Theme.surfaceVariantText
            }
            IconButton {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.guild === null && !root.mobile
                width: 32
                height: 32
                glyph: "Person"
                iconSize: 20
                iconColor: window.showingFriends ? Theme.primary : Theme.surfaceVariantText
                tip: "Friends"
                onClicked: if (window.openFriends) window.openFriends()
            }
        }

        HoverHandler {
            enabled: root.guild !== null
            cursorShape: Qt.PointingHandCursor
        }
        TapHandler {
            enabled: root.guild !== null
            onTapped: serverMenu.visible ? serverMenu.close() : serverMenu.open()
        }

        LampMenu {
            id: serverMenu
            y: header.height + 4
            x: 8
            width: Math.max(204, header.width - 16)
            closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

            LampMenuItem {
                text: "Mark As Read"
                leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: {
                    if (root.guild)
                        backend.markGuildRead(root.guild.id)
                }
            }
            LampMenuItem {
                text: "Edit Profile"
                leading: Component { Icon { name: "AccountCircle"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: if (window.openSettings) window.openSettings("profile")
            }
            LampMenuItem {
                text: "Server Settings"
                leading: Component { Icon { name: "Rounded.Settings"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: {
                    if (root.guild && window.openServerSettings)
                        window.openServerSettings(root.guild.id, root.guild.name)
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
                text: "Leave Server"
                danger: true
                leading: Component { Icon { name: "Logout"; size: 17; color: Theme.error } }
                onTriggered: {
                    if (root.guild)
                        backend.leaveGuild(root.guild.id)
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
                text: "Copy ID"
                leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: {
                    if (root.guild && window.copyText)
                        window.copyText(root.guild.id)
                }
            }
        }
    }

    ParallelAnimation {
        id: listSwitchAnim
        NumberAnimation { target: list; property: "opacity"; from: 0.0; to: 1.0; duration: Theme.quick; easing.type: Easing.OutCubic }
        NumberAnimation { target: dmList; property: "opacity"; from: 0.0; to: 1.0; duration: Theme.quick; easing.type: Easing.OutCubic }
    }

    Connections {
        target: backend
        function onSelectedGuildIdChanged() { listSwitchAnim.restart() }
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
                    mobile: root.mobile
                    categoryId: modelData.id
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
                    mobile: root.mobile
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
        topMargin: 4
        bottomMargin: 8
        spacing: 2
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: DmRow {
            required property var modelData
            width: dmList.width
            mobile: root.mobile
            dm: modelData
            selected: !window.showingFriends && backend.selectedChannelId === modelData.id
            onClicked: window.openDm(modelData.id)
        }
    }
}
