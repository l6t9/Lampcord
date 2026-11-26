import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// GuildFolderItem: a 48px tile tinted with the folder colour showing up to
// four server previews; opening it drops a well behind the folder's
// servers.
Item {
    id: root

    property var folder
    property bool expanded: false
    property bool mobile: false
    readonly property var guilds: folder.guilds
    readonly property bool childSelected: guilds.some(g => g.id === backend.selectedGuildId)
    readonly property color tint: folder.color !== "" ? folder.color : Theme.primary

    implicitWidth: 72
    implicitHeight: 48 + (expanded ? children_.implicitHeight + 4 : 0)
    clip: expanded

    Behavior on implicitHeight { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutCubic } }

    // Well behind the open folder.
    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        width: 48
        height: parent.height
        radius: 16
        color: Theme.alpha(Theme.surfaceVariant, 0.5)
        opacity: root.expanded ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    }

    Rectangle {
        anchors.left: parent.left
        y: (48 - height) / 2
        visible: !root.expanded && (root.childSelected || !!root.folder.unread || (root.folder.mentions || 0) > 0)
        width: 4
        height: root.childSelected ? 38 : 8
        topRightRadius: 4
        bottomRightRadius: 4
        color: Theme.surfaceText
    }

    Rectangle {
        id: tile
        anchors.horizontalCenter: parent.horizontalCenter
        width: 48
        height: 48
        radius: 16
        color: root.mobile
            ? Theme.alpha(root.tint, root.expanded ? (hover.hovered ? 0.3 : 0.18) : (hover.hovered ? 0.45 : 0.25))
            : Theme.alpha(root.tint, root.expanded ? (hover.hovered ? 0.2 : 0.1) : (hover.hovered ? 0.35 : 0.2))
        Behavior on color { ColorAnimation { duration: 100 } }

        Icon {
            anchors.centerIn: parent
            visible: root.expanded
            name: "FolderOpen"
            size: 28
            color: Theme.primary
        }

        // Preview grid, laid out like FolderPreviewGrid.
        Repeater {
            model: root.expanded ? [] : root.guilds.slice(0, 4)

            delegate: Item {
                required property var modelData
                required property int index
                readonly property int count: Math.min(root.guilds.length, 4)
                readonly property var offsets: ({
                    1: [[0, 0]],
                    2: [[-10, 0], [10, 0]],
                    3: [[0, -10], [-10, 10], [10, 10]],
                    4: [[-10, -10], [10, -10], [-10, 10], [10, 10]]
                })[count]
                x: 14 + offsets[index][0]
                y: 14 + offsets[index][1]
                width: 20
                height: 20

                Rectangle {
                    anchors.fill: parent
                    radius: 10
                    color: Theme.surfaceVariant
                    visible: !preview.ready

                    Text {
                        anchors.centerIn: parent
                        text: modelData.name.length > 0 ? modelData.name[0] : "?"
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 8
                    }
                }
                RoundedImage {
                    id: preview
                    anchors.fill: parent
                    source: modelData.icon
                }
            }
        }

        HoverHandler {
            id: hover
            cursorShape: Qt.PointingHandCursor
        }
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            cursorShape: Qt.PointingHandCursor
            onClicked: mouse => {
                if (mouse.button === Qt.RightButton)
                    folderMenu.popup(mouse.x, mouse.y)
                else if (mouse.button === Qt.LeftButton)
                    root.expanded = !root.expanded
            }
        }

        LampMenu {
            id: folderMenu

            LampMenuItem {
                text: "Mark as Read"
                leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: {
                    for (const g of root.guilds)
                        backend.markGuildRead(g.id)
                }
            }
            LampMenuItem {
                text: "Folder Settings"
                leading: Component { Icon { name: "Settings"; size: 17; color: Theme.surfaceVariantText } }
                onTriggered: root.expanded = !root.expanded
            }
        }

        RichTooltip {
            anchor: tile
            visible: hover.hovered
            text: root.folder.name !== "" ? root.folder.name : "Folder"
        }

        MentionBadge {
            x: parent.width - width + 3
            y: parent.height - height + 3
            visible: !root.expanded && (root.folder.mentions || 0) > 0
            count: root.folder.mentions || 0
        }
    }

    Column {
        id: children_
        y: 48 + 4
        width: parent.width
        spacing: 8
        visible: root.expanded

        Repeater {
            model: root.guilds

            delegate: GuildItem {
                required property var modelData
                guildId: modelData.id
                width: children_.width
                mobile: root.mobile
                icon: modelData.icon
                label: modelData.initials
                tip: modelData.name
                unread: !!modelData.unread
                mentions: modelData.mentions || 0
                selected: backend.selectedGuildId === modelData.id
                onClicked: window.openGuild(modelData.id)
            }
        }
    }

    // Open on the folder holding the restored server.
    Component.onCompleted: if (childSelected) expanded = true
}
