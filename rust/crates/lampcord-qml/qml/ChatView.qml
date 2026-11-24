import QtQuick
import QtQuick.Controls.Basic

Item {
    id: root

    readonly property var dm: window.dms.find(d => d.id === backend.selectedChannelId) || null
    readonly property var channel: dm || window.channels.find(c => c.id === backend.selectedChannelId) || null
    property bool loadingOlder: false
    // Mobile has its own top app bar, so it hides this desktop header.
    property bool compact: false

    function openPins() {
        backend.fetchPins()
        pins.width = Math.min(420, root.width - 16)
        if (compact) {
            pins.x = root.width - pins.width - 8
            pins.y = 8
        } else {
            pins.x = pinsButton.width - pins.width
            pins.y = pinsButton.height + 8
        }
        pins.open()
    }

    function handleBack() {
        if (pins.opened) {
            pins.close()
            return true
        }
        return composer.handleBack()
    }

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
                if (Object.keys(item).some(key => row[key] !== item[key]))
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

    // Header (TopAppBar, 56px).
    Item {
        id: header
        width: parent.width
        height: 56
        visible: !root.compact && root.channel !== null

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 16 - actions.width - 8
            spacing: 8

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm === null || root.dm.group
                name: root.dm ? "Group" : root.channel && root.channel.kind === 5 ? "Campaign" : "Tag"
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
                elide: Text.ElideRight
                text: root.channel ? root.channel.name : ""
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 18
                font.weight: Font.Bold
            }
        }

        Row {
            id: actions
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter

            // Calls need voice in the core; the button is here but off.
            IconButton {
                visible: root.dm !== null
                enabled: false
                glyph: "Call"
                iconSize: 22
                tip: "Voice calls aren't available yet"
            }
            IconButton {
                id: pinsButton
                glyph: "PushPin"
                iconSize: 22
                iconColor: pins.opened ? Theme.primary : Theme.surfaceVariantText
                tip: "Pins"
                onClicked: pins.opened ? pins.close() : root.openPins()

                PinsPopup {
                    id: pins
                    x: pinsButton.width - width
                    y: pinsButton.height + 8
                    onAboutToShow: width = Math.min(420, root.width - 16)
                }
            }
            IconButton {
                glyph: root.dm && !root.dm.group ? "AccountCircle" : "Group"
                iconSize: 24
                iconColor: memory.memberPanel ? Theme.primary : Theme.surfaceVariantText
                tip: "Toggle Member List"
                onClicked: memory.memberPanel = !memory.memberPanel
            }
        }
    }

    ListView {
        id: list
        anchors.top: root.compact ? parent.top : header.bottom
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
            messageId: model.id
            authorId: model.authorId
            mine: model.authorId !== "" && model.authorId === backend.currentUserId

            onActionReply: id => composer.replyTo(id, model.author, model.content)
            onActionEdit: (id, content) => composer.editMessage(id, content)
            onActionDelete: id => backend.deleteMessage(id)
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
