import QtQuick
import QtQuick.Controls.Basic

Item {
    id: root

    readonly property var dm: window.dms.find(d => d.id === backend.selectedChannelId) || null
    readonly property var channel: dm || window.channels.find(c => c.id === backend.selectedChannelId) || null
    property bool loadingOlder: false

    // Newest first, matching the bottom-to-top list below.
    ListModel { id: messages }

    function idCompare(a, b) {
        return a.length !== b.length ? a.length - b.length : (a < b ? -1 : a > b ? 1 : 0)
    }

    // Merge the snapshot into the model in place so the list keeps its
    // scroll position (and delegates) across updates.
    function sync() {
        const incoming = JSON.parse(backend.messagesJson).reverse()
        let i = 0
        for (const item of incoming) {
            while (i < messages.count && idCompare(messages.get(i).id, item.id) > 0)
                messages.remove(i)
            if (i < messages.count && messages.get(i).id === item.id) {
                const row = messages.get(i)
                if (row.content !== item.content || row.grouped !== item.grouped
                        || row.author !== item.author || row.avatar !== item.avatar || row.edited !== item.edited)
                    messages.set(i, item)
            } else {
                messages.insert(i, item)
            }
            i++
        }
        if (messages.count > i)
            messages.remove(i, messages.count - i)
        loadingOlder = false
        if (Qt.application.state === Qt.ApplicationActive && list.atYEnd)
            backend.ackLatest()
    }

    Connections {
        target: backend
        function onMessagesJsonChanged() { root.sync() }
        function onSelectedChannelIdChanged() {
            messages.clear()
            list.positionViewAtBeginning()
            composer.reset()
        }
    }

    // Header (TopAppBar)
    Item {
        id: header
        width: parent.width
        height: 64
        visible: root.channel !== null

        Row {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 16
            spacing: 8

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm === null || root.dm.group
                name: root.dm ? "group" : root.channel && root.channel.kind === 5 ? "campaign" : "tag"
                size: 28
                color: Theme.surfaceVariantText
            }
            Avatar {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm !== null && !root.dm.group
                size: 28
                name: root.dm ? root.dm.name : ""
                source: root.dm ? root.dm.avatar : ""
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 36
                text: root.channel ? root.channel.name : ""
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 18
                font.weight: Font.Bold
                elide: Text.ElideRight
            }
        }
    }

    ListView {
        id: list
        anchors.top: header.bottom
        anchors.bottom: composer.top
        width: parent.width
        visible: root.channel !== null
        model: messages
        verticalLayoutDirection: ListView.BottomToTop
        clip: true
        reuseItems: true
        bottomMargin: 8
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: MessageDelegate {
            width: list.width
        }

        // Reaching the oldest loaded message pulls the previous page.
        onContentYChanged: {
            if (!root.loadingOlder && messages.count >= 50 && visibleArea.yPosition < 0.05) {
                root.loadingOlder = true
                backend.loadOlder()
            }
        }

        footer: Item {
            width: list.width
            height: root.loadingOlder ? 48 : 0
            LoadingIndicator {
                anchors.centerIn: parent
                visible: root.loadingOlder
                size: 32
            }
        }
    }

    Composer {
        id: composer
        anchors.bottom: parent.bottom
        width: parent.width
        visible: root.channel !== null
        placeholder: !root.channel ? "" : root.dm ? "Message @" + root.dm.name : "Message #" + root.channel.name
    }

    Text {
        anchors.centerIn: parent
        visible: root.channel === null
        text: backend.selectedGuildId === "" ? "Select a friend to start chatting" : "Pick a channel to start chatting"
        color: Theme.surfaceVariantText
        font.family: Theme.font
        font.pixelSize: 14
    }
}
