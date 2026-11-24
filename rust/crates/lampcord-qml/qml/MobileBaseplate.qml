import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Lampcord

// Mobile coordinator: a small navigation stack, the five Compose tabs, the
// start/member drawers and the chat surface. Each tab owns one route; opening
// a conversation from search/mentions replaces the stack with the chat route.
Item {
    id: root
    anchors.fill: parent

    property string route: "chat"
    property string tab: "home"
    property var stack: [{ route: "chat", tab: "home" }]

    readonly property var guild: window.guilds.find(g => g.id === backend.selectedGuildId) || null
    readonly property var channel: window.channels.find(c => c.id === backend.selectedChannelId) || null

    readonly property var tabs: {
        const out = []
        if (memory.showHome) out.push({ routeKey: "home", glyph: "Home", label: "Home" })
        if (memory.showDms) out.push({ routeKey: "dm", glyph: "ChatBubble", label: "Messages" })
        if (memory.showSearch) out.push({ routeKey: "search", glyph: "Search", label: "Search" })
        if (memory.showMentions) out.push({ routeKey: "mentions", glyph: "AlternateEmail", label: "Mentions" })
        if (memory.showYou) out.push({ routeKey: "you", glyph: "AccountCircle", label: "You" })
        if (out.length === 0) out.push({ routeKey: "you", glyph: "AccountCircle", label: "You" })
        return out
    }

    readonly property string title: {
        if (root.route === "dm") return "Direct Messages"
        if (root.route === "you") return "You"
        if (root.route === "search") return "Search"
        if (root.route === "mentions") return "Mentions"
        if (root.channel) return root.channel.name
        if (root.guild) return root.guild.name
        return "Direct Messages"
    }

    function pushRoute(key) {
        const next = { route: key, tab: root.tab }
        root.stack = root.stack.concat([next])
        root.route = key
    }

    function replaceRoute(key, newTab) {
        if (newTab && newTab !== "")
            root.tab = newTab
        root.stack = [{ route: key, tab: root.tab }]
        root.route = key
    }

    function selectRoute(key) {
        if (key === "home") {
            if (root.route !== "chat")
                root.replaceRoute("chat", "home")
            else
                root.tab = "home"
            navDrawer.open()
            return
        }
        root.replaceRoute(key, key)
        navDrawer.close()
        memberDrawer.close()
    }

    function handleBack() {
        if (navDrawer.open) {
            navDrawer.close()
            return true
        }
        if (memberDrawer.open) {
            memberDrawer.close()
            return true
        }
        const view = contentLoader.item
        if (view && view.handleBack && view.handleBack())
            return true
        if (root.stack.length > 1) {
            const next = root.stack[root.stack.length - 2]
            root.stack = root.stack.slice(0, -1)
            root.route = next.route
            root.tab = next.tab
            return true
        }
        if (root.route !== "chat") {
            root.replaceRoute("chat")
            return true
        }
        return false
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.surfaceContainerHigh
    }

    // ---------------------------------------------------------------- top bar

    Rectangle {
        id: topBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: window.statusBarInset + 56
        color: Theme.surfaceContainer
        z: 2

        RowLayout {
            anchors.fill: parent
            anchors.topMargin: window.statusBarInset
            anchors.leftMargin: 4
            anchors.rightMargin: 4
            spacing: 2

            IconButton {
                visible: root.stack.length > 1 || root.route !== "chat"
                glyph: "Rounded.ArrowBack"
                tip: "Back"
                onClicked: root.handleBack()
            }

            IconButton {
                glyph: "Rounded.Menu"
                tip: "Servers and channels"
                onClicked: navDrawer.open()
            }

            Text {
                Layout.fillWidth: true
                leftPadding: 6
                text: root.title
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 17
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }

            IconButton {
                visible: backend.selectedChannelId !== "" && root.route === "chat"
                glyph: "Rounded.Search"
                tip: "Search"
                onClicked: root.pushRoute("search")
            }

            IconButton {
                visible: backend.selectedChannelId !== "" && root.route === "chat"
                glyph: "PushPin"
                tip: "Pins"
                onClicked: {
                    const view = contentLoader.item
                    if (view && view.openPins)
                        view.openPins()
                }
            }

            IconButton {
                visible: backend.selectedChannelId !== "" && root.route === "chat"
                glyph: "Rounded.Group"
                tip: "Members"
                onClicked: memberDrawer.open()
            }
        }
    }

    // ---------------------------------------------------------------- content

    Rectangle {
        id: content
        anchors.top: topBar.bottom
        anchors.bottom: bottomNav.top
        anchors.left: parent.left
        anchors.right: parent.right
        color: Theme.surface
        clip: true

        Loader {
            id: contentLoader
            anchors.fill: parent
            sourceComponent: {
                if (root.route === "chat") return chatComponent
                if (root.route === "dm") return dmsComponent
                if (root.route === "search") return searchComponent
                if (root.route === "mentions") return mentionsComponent
                if (root.route === "you") return youComponent
                return chatComponent
            }
        }
    }

    Component {
        id: chatComponent
        ChatView {
            anchors.fill: parent
            compact: true
        }
    }

    Component {
        id: dmsComponent
        MobileDms {
            anchors.fill: parent
            onOpenServers: navDrawer.open()
            onOpenedDm: root.replaceRoute("chat", "dm")
        }
    }

    Component {
        id: searchComponent
        MobileSearch {
            anchors.fill: parent
            onOpened: root.replaceRoute("chat")
        }
    }

    Component {
        id: mentionsComponent
        MobileMentions {
            anchors.fill: parent
            onOpenedDm: root.replaceRoute("chat", "mentions")
        }
    }

    Component {
        id: youComponent
        MobileYou {
            anchors.fill: parent
        }
    }

    // --------------------------------------------------------------- drawers

    Drawer {
        id: navDrawer
        edge: Qt.LeftEdge
        width: Math.min(320, root.width * 0.86)
        y: window.statusBarInset
        height: root.height - window.statusBarInset - window.navBarInset
        modal: true

        background: Rectangle { color: Theme.surfaceContainerLow }

        contentItem: Item {
            Row {
                anchors.fill: parent

                GuildRail {
                    width: 72
                    height: parent.height
                }

                Rectangle {
                    width: parent.width - 72
                    height: parent.height
                    color: Theme.surface

                    ChannelList {
                        anchors.fill: parent
                    }
                }
            }
        }

        onOpened: {
            if (backend.selectedChannelId !== "")
                backend.selectChannel(backend.selectedChannelId)
        }
    }

    Drawer {
        id: memberDrawer
        edge: Qt.RightEdge
        width: Math.min(300, root.width * 0.86)
        y: window.statusBarInset
        height: root.height - window.statusBarInset - window.navBarInset
        modal: true

        background: Rectangle { color: Theme.surfaceContainerLow }

        contentItem: MemberPanel {
            anchors.fill: parent
        }

        onOpened: memory.memberPanel = true
        onClosed: memory.memberPanel = false
    }

    Connections {
        target: memory
        function onMemberPanelChanged() {
            if (memory.memberPanel && !memberDrawer.open)
                memberDrawer.open()
            if (!memory.memberPanel && memberDrawer.open)
                memberDrawer.close()
        }
    }

    // ---------------------------------------------------------- bottom nav

    Rectangle {
        id: bottomNav
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 64 + window.navBarInset
        color: Theme.surface
        z: 2

        Row {
            anchors.fill: parent
            anchors.bottomMargin: window.navBarInset

            Repeater {
                model: root.tabs

                delegate: AbstractButton {
                    required property var modelData
                    width: parent.width / root.tabs.length
                    height: parent.height
                    down: false

                    readonly property bool selected: {
                        if (modelData.routeKey === "home")
                            return root.tab === "home" && root.route === "chat"
                        return root.tab === modelData.routeKey
                    }

                    onClicked: root.selectRoute(modelData.routeKey)

                    Column {
                        anchors.centerIn: parent
                        spacing: 2

                        Icon {
                            anchors.horizontalCenter: parent.horizontalCenter
                            name: selected ? modelData.glyph : "Rounded." + modelData.glyph
                            size: 22
                            color: selected ? Theme.primary : Theme.surfaceVariantText
                        }
                        Text {
                            text: modelData.label
                            color: selected ? Theme.primary : Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 11
                        }
                    }

                    MentionBadge {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.top: parent.top
                        anchors.topMargin: 8
                        visible: modelData.routeKey === "dm" && root.totalMentions > 0
                        count: root.totalMentions
                    }
                }
            }
        }
    }

    readonly property int totalMentions: {
        let count = 0
        for (const dm of window.dms)
            count += dm.mentions || 0
        for (const channel of window.channels)
            count += channel.mentions || 0
        return count
    }
}
