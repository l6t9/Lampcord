import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Lampcord

Popup {
    id: root

    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 0
    width: Math.min(860, parent ? parent.width - 48 : 860)
    height: Math.min(620, parent ? parent.height - 48 : 620)
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    property string guildId: ""
    property string guildName: ""
    property string tab: "channels"

    readonly property var roles: {
        try { return JSON.parse(backend.guildRolesJson) } catch (e) { return [] }
    }
    readonly property var emojis: {
        try { return JSON.parse(backend.guildEmojisJson) } catch (e) { return [] }
    }
    readonly property var bans: {
        try { return JSON.parse(backend.guildBansJson) } catch (e) { return [] }
    }

    function openForGuild(id, name) {
        guildId = id
        guildName = name
        backend.fetchGuildSettings(id)
        open()
    }

    enter: Transition {
        ParallelAnimation {
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.quick }
            NumberAnimation { property: "scale"; from: 0.96; to: 1; duration: Theme.quick; easing.type: Easing.OutCubic }
        }
    }

    Overlay.modal: Rectangle { color: Theme.alpha("#000000", 0.55) }

    background: Rectangle {
        radius: 24
        color: Theme.surfaceContainerLow
        border.width: 1
        border.color: Theme.outlineVariant
        clip: true
    }

    contentItem: RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 200
            Layout.fillHeight: true
            color: Theme.surfaceContainer
            topLeftRadius: 24
            bottomLeftRadius: 24

            Column {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 4

                Text {
                    width: parent.width
                    leftPadding: 12
                    topPadding: 8
                    bottomPadding: 12
                    text: root.guildName || "Server Settings"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 16
                    font.weight: Font.Bold
                    elide: Text.ElideRight
                }

                Repeater {
                    model: [
                        { key: "overview", label: "Overview", icon: "Info" },
                        { key: "channels", label: "Channels", icon: "Tag" },
                        { key: "roles", label: "Roles", icon: "Group" },
                        { key: "emojis", label: "Emojis", icon: "Mood" },
                        { key: "bans", label: "Bans", icon: "Block" }
                    ]

                    delegate: Rectangle {
                        required property var modelData
                        readonly property bool active: root.tab === modelData.key
                        width: parent.width
                        height: 38
                        radius: 19
                        color: active ? Theme.secondaryContainer : tHover.hovered ? Theme.alpha(Theme.surfaceText, 0.06) : "transparent"

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            spacing: 10
                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: modelData.icon
                                size: 18
                                color: active ? Theme.secondaryContainerText : Theme.surfaceVariantText
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: modelData.label
                                color: active ? Theme.secondaryContainerText : Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 14
                                font.weight: active ? Font.DemiBold : Font.Normal
                            }
                        }

                        HoverHandler { id: tHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: root.tab = modelData.key }
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            IconButton {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 12
                z: 2
                glyph: "Close"
                tip: "Close"
                onClicked: root.close()
            }

            Flickable {
                anchors.fill: parent
                anchors.margins: 24
                contentHeight: pane.implicitHeight + 32
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

                Column {
                    id: pane
                    width: parent.width - 12
                    spacing: 14

                    Column {
                        width: parent.width
                        spacing: 12
                        visible: root.tab === "overview"

                        Text {
                            text: "Server Overview"
                            color: Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 20
                            font.weight: Font.Bold
                        }
                        Text {
                            text: "Name: " + root.guildName
                            color: Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 14
                        }
                        Text {
                            text: "Server ID: " + root.guildId
                            color: Theme.surfaceVariantText
                            font.family: Theme.monoFont
                            font.pixelSize: 12
                        }
                        TextButton {
                            text: "Leave Server"
                            onClicked: {
                                backend.leaveGuild(root.guildId)
                                root.close()
                            }
                        }
                    }

                    ServerChannelsTab {
                        width: parent.width
                        visible: root.tab === "channels"
                        guildId: root.guildId
                    }

                    ServerRolesTab {
                        width: parent.width
                        visible: root.tab === "roles"
                        guildId: root.guildId
                        roles: root.roles
                    }

                    ServerEmojisTab {
                        width: parent.width
                        visible: root.tab === "emojis"
                        guildId: root.guildId
                        emojis: root.emojis
                    }

                    ServerBansTab {
                        width: parent.width
                        visible: root.tab === "bans"
                        guildId: root.guildId
                        bans: root.bans
                    }
                }
            }
        }
    }
}
