import QtCore
import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Local finder for mobile. The core does not expose the REST gateway search
// to QML yet, so this searches guilds, channels, DMs and the messages already
// loaded into the client. It keeps a small local history like the Compose
// search sheet does.
Item {
    id: root

    signal opened()

    property string query: ""

    Settings {
        id: history
        category: "search"
        property string recentJson: "[]"
    }

    function channelLabel(channelId) {
        const dm = window.dms.find(item => item.id === channelId)
        if (dm)
            return dm.name
        const channel = window.channels.find(item => item.id === channelId && !item.category)
        return channel ? "#" + channel.name : "Unknown channel"
    }

    function buildResults() {
        const q = root.query.trim().toLowerCase()
        const out = []
        const add = (kind, title, subtitle, icon, id, guildId, channelId) => {
            out.push({
                kind: kind, title: title, subtitle: subtitle, icon: icon,
                id: id, guildId: guildId || "", channelId: channelId || ""
            })
        }

        if (q === "") {
            const recent = JSON.parse(history.recentJson)
            for (const text of recent)
                add("recent", text, "Recent search", "Rounded.Search", "", "", text)
            for (const dm of window.dms.slice(0, 5))
                add("dm", dm.name, "Direct message", "Rounded.ChatBubble", dm.id, "", dm.id)
            for (const channel of window.channels.filter(c => !c.category).slice(0, 5))
                add("channel", "#" + channel.name, "Channel", "Rounded.Tag", channel.id, channel.guildId, channel.id)
            return out
        }

        for (const guild of window.guilds) {
            if (guild.name.toLowerCase().indexOf(q) !== -1)
                add("guild", guild.name, "Server", "Rounded.Group", guild.id, guild.id, "")
        }
        for (const channel of window.channels) {
            if (channel.category || channel.name.toLowerCase().indexOf(q) === -1)
                continue
            add("channel", "#" + channel.name, "Channel", "Rounded.Tag", channel.id, channel.guildId, channel.id)
        }
        for (const dm of window.dms) {
            if (dm.name.toLowerCase().indexOf(q) === -1)
                continue
            add("dm", dm.name, dm.group ? "Group message" : "Direct message",
                dm.group ? "Rounded.Group" : "Rounded.ChatBubble", dm.id, "", dm.id)
        }

        const messages = JSON.parse(backend.messagesJson)
        for (const message of messages) {
            if (message.content.toLowerCase().indexOf(q) === -1)
                continue
            const channel = window.channels.find(c => c.id === message.channelId)
            add("message", message.author + ": " + message.content.replace(/\s+/g, " ").slice(0, 80),
                root.channelLabel(message.channelId), "Rounded.ChatBubble",
                message.id, channel ? channel.guildId : "", message.channelId)
        }
        const mentions = JSON.parse(backend.mentionsJson)
        for (const message of mentions) {
            if (message.content.toLowerCase().indexOf(q) === -1)
                continue
            const channel = window.channels.find(c => c.id === message.channelId)
            add("message", message.author + ": " + message.content.replace(/\s+/g, " ").slice(0, 80),
                "Mention in " + root.channelLabel(message.channelId), "Rounded.AlternateEmail",
                message.id, channel ? channel.guildId : "", message.channelId)
        }
        return out.slice(0, 80)
    }

    readonly property var results: {
        // Explicit reads keep the binding alive when a snapshot arrives.
        const guilds = window.guilds
        const channels = window.channels
        const dms = window.dms
        const messages = backend.messagesJson
        const mentions = backend.mentionsJson
        const recent = history.recentJson
        const text = root.query
        return root.buildResults()
    }

    function remember(text) {
        const clean = text.trim()
        if (clean === "")
            return
        let recent = JSON.parse(history.recentJson).filter(item => item !== clean)
        recent.unshift(clean)
        history.recentJson = JSON.stringify(recent.slice(0, 8))
    }

    function openResult(row) {
        if (row.kind === "recent") {
            root.query = row.title
            return
        }
        if (row.kind === "guild") {
            root.remember(row.title)
            window.openGuild(row.id)
            root.opened()
            return
        }
        if (row.kind === "dm") {
            root.remember(row.title)
            window.openDm(row.id)
            root.opened()
            return
        }
        if (row.kind === "channel" || row.kind === "message") {
            root.remember(root.query || row.title)
            if (row.guildId !== "")
                window.openGuild(row.guildId)
            else
                window.openHome()
            window.openChannel(row.channelId)
            root.opened()
        }
    }

    function handleBack() {
        if (root.query !== "") {
            root.query = ""
            return true
        }
        return false
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.surface
    }

    Rectangle {
        id: searchBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 64
        color: Theme.surface

        Rectangle {
            anchors.fill: parent
            anchors.margins: 8
            radius: 20
            color: Theme.surfaceContainerHigh

            Icon {
                id: searchIcon
                anchors.left: parent.left
                anchors.leftMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                name: "Rounded.Search"
                size: 20
                color: Theme.surfaceVariantText
            }

            TextField {
                id: field
                anchors.left: searchIcon.right
                anchors.leftMargin: 10
                anchors.right: clearButton.left
                anchors.rightMargin: 4
                anchors.verticalCenter: parent.verticalCenter
                placeholderText: "Search chats and loaded messages"
                placeholderTextColor: Theme.alpha(Theme.surfaceVariantText, 0.65)
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                background: null
                text: root.query
                onTextChanged: root.query = text
                onAccepted: {
                    root.remember(text)
                    if (root.results.length > 0)
                        root.openResult(root.results[0])
                }
            }

            IconButton {
                id: clearButton
                anchors.right: parent.right
                anchors.rightMargin: 4
                anchors.verticalCenter: parent.verticalCenter
                width: 32
                height: 32
                visible: field.text !== ""
                glyph: "Rounded.Close"
                iconSize: 18
                onClicked: {
                    field.clear()
                    field.forceActiveFocus()
                }
            }
        }
    }

    Text {
        id: hint
        anchors.top: searchBar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        padding: 12
        text: root.query === "" ? "Recent and suggestions" : "Searching this device's loaded chats"
        color: Theme.surfaceVariantText
        font.family: Theme.font
        font.pixelSize: 12
    }

    ListView {
        id: list
        anchors.top: hint.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        model: root.results
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        bottomMargin: 12
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: Rectangle {
            required property var modelData
            width: list.width
            height: 64
            color: area.pressed ? Theme.alpha(Theme.surfaceVariant, 0.16) : "transparent"

            Row {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                spacing: 14

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: modelData.icon
                    size: 22
                    color: Theme.surfaceVariantText
                }

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 36
                    spacing: 2

                    Text {
                        width: parent.width
                        text: modelData.title
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        elide: Text.ElideRight
                    }
                    Text {
                        width: parent.width
                        text: modelData.subtitle
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }
                }
            }

            MouseArea {
                id: area
                anchors.fill: parent
                onClicked: root.openResult(modelData)
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
        spacing: 8
        visible: list.count === 0

        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: "Rounded.Search"
            size: 48
            color: Theme.outline
        }
        Text {
            text: "No results on this device"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
        }
    }
}
