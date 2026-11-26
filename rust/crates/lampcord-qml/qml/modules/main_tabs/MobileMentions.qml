import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// The mobile mentions inbox. The core fetches a page from the REST gateway
// into the snapshot; this view renders it and opens the source conversation.
Item {
    id: root

    signal openedDm()

    readonly property var rows: JSON.parse(backend.mentionsJson)

    function channelLabel(channelId) {
        const dm = window.dms.find(item => item.id === channelId)
        if (dm)
            return dm.name
        const channel = window.channels.find(item => item.id === channelId && !item.category)
        return channel ? "#" + channel.name : "Unknown channel"
    }

    function openMention(row) {
        const channel = window.channels.find(item => item.id === row.channelId)
        if (channel && channel.guildId !== "")
            window.openGuild(channel.guildId)
        else
            window.openHome()
        window.openChannel(row.channelId)
        root.openedDm()
    }

    Component.onCompleted: backend.fetchMentions()

    Rectangle {
        anchors.fill: parent
        color: Theme.surface
    }

    Rectangle {
        id: toolbar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 52
        color: Theme.surface

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: root.rows.length === 0 ? "Mentions" : root.rows.length + " recent mentions"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 13
        }

        TextButton {
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            text: "Refresh"
            onClicked: backend.fetchMentions()
        }
    }

    ListView {
        id: list
        anchors.top: toolbar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        model: root.rows
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        bottomMargin: 12
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: Rectangle {
            required property var modelData
            width: list.width
            height: content.implicitHeight + 20
            color: area.pressed ? Theme.alpha(Theme.surfaceVariant, 0.16) : "transparent"

            Column {
                id: content
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: 10
                spacing: 4

                Row {
                    width: parent.width
                    spacing: 8

                    UserAvatar {
                        anchors.verticalCenter: parent.verticalCenter
                        size: 24
                        name: modelData.author
                        source: modelData.avatar
                        decoration: modelData.decoration
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.author
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.channelLabel(modelData.channelId)
                        color: Theme.primary
                        font.family: Theme.font
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }
                }

                Text {
                    width: parent.width
                    text: modelData.content.replace(/\s+/g, " ")
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    wrapMode: Text.Wrap
                    maximumLineCount: 3
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                id: area
                anchors.fill: parent
                onClicked: root.openMention(modelData)
            }

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: Theme.alpha(Theme.outlineVariant, 0.4)
            }
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: 12
        visible: list.count === 0

        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: "Rounded.AlternateEmail"
            size: 48
            color: Theme.outline
        }
        Text {
            text: "No mentions yet"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 15
        }
        TextButton {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Check again"
            onClicked: backend.fetchMentions()
        }
    }
}
