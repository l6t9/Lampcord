import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    readonly property var dm: window.dms.find(d => d.id === backend.selectedChannelId) || null
    readonly property var channel: dm || window.channels.find(c => c.id === backend.selectedChannelId) || null
    property bool loadingOlder: false
    // Mobile has its own top app bar, so it hides this desktop header.
    property bool compact: false
    // The mobile composer floats over the chat surface with M3 side/bottom insets.
    property bool floatingComposer: false
    property string reactTargetId: ""

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

    function openSearch() {
        searchPanel.width = Math.min(440, root.width - 16)
        searchPanel.x = Math.max(8, root.width - searchPanel.width - 16)
        searchPanel.y = header.height + 8
        searchPanel.open()
    }

    function openThreads() {
        if (window.toggleThreads)
            window.toggleThreads()
    }

    function handleBack() {
        if (pins.opened) {
            pins.close()
            return true
        }
        if (searchPanel.opened) {
            searchPanel.close()
            return true
        }
        if (reactPicker.opened) {
            reactPicker.close()
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
    // Date separators are computed in local time (matching MessageDelegate.stamp).
    function sync() {
        const chronological = JSON.parse(backend.messagesJson)
        let prevDay = ""
        for (let k = 0; k < chronological.length; k++) {
            const item = chronological[k]
            const d = new Date(item.time)
            const dayKey = d.getFullYear() + "-" + d.getMonth() + "-" + d.getDate()
            const newDay = prevDay === "" || dayKey !== prevDay
            item.showDate = newDay
            item.dateLabel = newDay ? Qt.formatDate(d, "MMMM d, yyyy") : ""
            if (newDay)
                item.grouped = false
            prevDay = dayKey
        }
        // Track whether the next message continues the same bubble group.
        for (let k = 0; k < chronological.length; k++) {
            const cur = chronological[k]
            const nxt = k + 1 < chronological.length ? chronological[k + 1] : null
            cur.hasNextSameUser = !!(
                nxt
                && !nxt.showDate
                && nxt.grouped
                && cur.system === ""
                && nxt.system === ""
                && cur.authorId !== ""
                && cur.authorId === nxt.authorId
            )
        }
        const incoming = chronological.reverse().map(item => {
            // ListModel role names cannot shadow an Item's `id`, so the
            // delegate reads `messageId` instead of `model.id`.
            const row = Object.assign({}, item)
            row.messageId = item.id
            return row
        })
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

    ParallelAnimation {
        id: channelSwitchAnim
        NumberAnimation { target: root; property: "opacity"; from: 0.0; to: 1.0; duration: Theme.quick; easing.type: Easing.OutCubic }
        NumberAnimation { target: list; property: "x"; from: 28; to: 0; duration: 220; easing.type: Easing.OutCubic }
    }

    Connections {
        target: backend
        function onMessagesJsonChanged() { root.sync() }
        function onSelectedChannelIdChanged() {
            messages.clear()
            list.positionViewAtBeginning()
            composer.reset()
            if (backend.selectedChannelId !== "")
                channelSwitchAnim.restart()
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
            width: parent.width - 16 - actions.width - 12
            spacing: 8

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm === null || root.dm.group
                name: root.dm ? "Group"
                    : root.channel && root.channel.kind === 5 ? "Campaign"
                    : root.channel && root.channel.kind === 2 ? "VolumeUp"
                    : root.channel && root.channel.kind === 13 ? "Podcasts"
                    : root.channel && root.channel.kind === 15 ? "Rounded.Forum"
                    : "Tag"
                size: 26
                color: Theme.surfaceVariantText
            }
            UserAvatar {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm !== null && !root.dm.group
                size: 28
                name: root.dm ? root.dm.name : ""
                source: root.dm ? root.dm.avatar : ""
                decoration: root.dm ? root.dm.decoration : ""
                status: root.dm ? root.dm.status : ""
                ring: Theme.surface
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 36
                spacing: 0

                Text {
                    width: parent.width
                    elide: Text.ElideRight
                    text: root.channel ? root.channel.name : ""
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 17
                    font.weight: Font.Bold
                }
                Text {
                    width: parent.width
                    visible: root.channel && root.channel.topic && root.channel.topic !== "" ? true : false
                    elide: Text.ElideRight
                    text: root.channel && root.channel.topic ? root.channel.topic : ""
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 11
                }
            }
        }

        Row {
            id: actions
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            // Calls need voice in the core; the button is here but off.
            IconButton {
                visible: root.dm !== null && Theme.prefs.showCallButton
                enabled: false
                glyph: "Call"
                iconSize: 22
                tip: "Voice calls aren't available yet"
            }
            IconButton {
                visible: Theme.prefs.showChatSearch && root.channel !== null
                glyph: "Search"
                iconSize: 22
                iconColor: searchPanel.opened ? Theme.primary : Theme.surfaceVariantText
                tip: "Search Messages"
                onClicked: searchPanel.opened ? searchPanel.close() : root.openSearch()
            }
            IconButton {
                id: threadsButton
                visible: root.dm === null && root.channel !== null && (root.channel.kind === 0 || root.channel.kind === 5 || root.channel.kind === 15)
                glyph: "Rounded.Topic"
                iconSize: 22
                iconColor: window.threadPanelVisible ? Theme.primary : Theme.surfaceVariantText
                tip: "Threads"
                onClicked: root.openThreads()
            }
            IconButton {
                id: pinsButton
                visible: Theme.prefs.showChatPins
                glyph: "PushPin"
                iconSize: 21
                iconColor: pins.opened ? Theme.primary : Theme.surfaceVariantText
                tip: "Pinned Messages"
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
                iconSize: 23
                iconColor: memory.memberPanel ? Theme.primary : Theme.surfaceVariantText
                tip: "Toggle Member List"
                onClicked: memory.memberPanel = !memory.memberPanel
            }
        }

        SearchPanel {
            id: searchPanel
            x: Math.max(8, root.width - width - 16)
            y: header.height + 8
        }

        Shortcut {
            sequences: [StandardKey.Find]
            onActivated: if (root.channel !== null) searchPanel.open()
        }
    }

    ListView {
        id: list
        anchors.top: root.compact ? parent.top : header.bottom
        // The floating composer overlays the list like Phoenix; keep a small
        // gap so the newest message is never hidden behind the pill.
        anchors.bottom: root.floatingComposer ? parent.bottom : composer.top
        anchors.bottomMargin: root.floatingComposer ? 96 + window.navBarInset : 8
        width: parent.width
        visible: root.channel !== null
        model: messages
        verticalLayoutDirection: ListView.BottomToTop
        clip: true
        reuseItems: true
        bottomMargin: root.floatingComposer ? 0 : 8
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: MessageDelegate {
            required property var model
            width: list.width
            mine: authorId !== "" && authorId === backend.currentUserId
            systemPrefix: model.systemPrefix || ""
            showDate: !!model.showDate
            dateLabel: model.dateLabel || ""
            hasNextSameUser: !!model.hasNextSameUser
            deleted: !!model.deleted
            mentioned: !!model.mentioned
            pinned: !!model.pinned
            clanTag: model.clanTag || ""
            clanBadge: model.clanBadge || ""
            reactions: model.reactions || "[]"
            stickers: model.stickers || "[]"
            embeds: model.embeds || "[]"
            forwarded: model.forwarded || "[]"

            onActionReply: id => composer.replyTo(id, author, content)
            onActionEdit: (id, content) => composer.editMessage(id, content)
            onActionDelete: id => backend.deleteMessage(id)
            onActionPin: (id, isPinned) => isPinned ? backend.unpinMessage(id) : backend.pinMessage(id)
            onActionReact: id => {
                root.reactTargetId = id
                reactPicker.x = Math.max(8, (root.width - reactPicker.width) / 2)
                reactPicker.y = Math.max(8, (root.height - reactPicker.height) / 2)
                reactPicker.open()
            }
            onActionToggleReaction: (id, key, hasMe) => {
                if (hasMe)
                    backend.removeReaction(id, key)
                else
                    backend.addReaction(id, key)
            }
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
            height: root.loadingOlder ? 48 : (welcomeColumn.implicitHeight + 32)

            LoadingIndicator {
                anchors.centerIn: parent
                visible: root.loadingOlder
                size: 32
            }

            Column {
                id: welcomeColumn
                visible: !root.loadingOlder && root.channel !== null
                x: 16
                y: 16
                width: parent.width - 32
                spacing: 0

                Rectangle {
                    width: 74
                    height: 74
                    radius: 37
                    color: Theme.surfaceContainerHigh

                    Icon {
                        anchors.centerIn: parent
                        name: root.dm !== null ? "Rounded.AlternateEmail" : "Tag"
                        size: 46
                        color: Theme.surfaceText
                    }
                }

                Item { width: 1; height: 14 }

                Text {
                    width: parent.width
                    text: root.dm !== null
                        ? "This is the start of your conversation."
                        : ("Welcome to #" + (root.channel ? root.channel.name : ""))
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 32
                    font.weight: Font.Bold
                    wrapMode: Text.Wrap
                }

                Item { width: 1; height: 8 }

                Text {
                    width: parent.width
                    text: root.dm !== null
                        ? "This is the very beginning of your direct message history."
                        : ("This is the start of the #" + (root.channel ? root.channel.name : "") + " channel.")
                    color: Theme.alpha(Theme.surfaceVariantText, 0.7)
                    font.family: Theme.font
                    font.pixelSize: 15
                    wrapMode: Text.Wrap
                }

                Item { width: 1; height: 16 }
            }
        }
    }

    EmojiPicker {
        id: reactPicker
        onPicked: emoji => {
            if (root.reactTargetId !== "" && emoji !== "") {
                // Convert custom emoji token `<:name:id>` to `name:id` for reaction API.
                const match = emoji.match(/^<a?:([^:>]+):(\d+)>$/)
                const key = match ? (match[1] + ":" + match[2]) : emoji
                backend.addReaction(root.reactTargetId, key)
            }
            reactPicker.close()
        }
    }

    Connections {
        target: window
        function onMentionUser(userId) {
            if (root.channel !== null)
                composer.insertMention(userId)
        }
    }

    Composer {
        id: composer
        anchors.bottom: parent.bottom
        width: parent.width
        visible: root.channel !== null
        floating: root.floatingComposer
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
