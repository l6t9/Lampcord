import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Lampcord

// First mobile slice: the Compose MobileBaseplate's navigation shape —
// start drawer (server rail + channels), centre chat, bottom navigation —
// rebuilt from the existing desktop QML components.
Item {
    id: root
    anchors.fill: parent

    property string route: "chat"

    readonly property var guild: window.guilds.find(g => g.id === backend.selectedGuildId) || null
    readonly property var channel: window.channels.find(c => c.id === backend.selectedChannelId) || null

    readonly property string title: {
        if (root.route === "you") return "You"
        if (root.route === "search") return "Search"
        if (root.route === "mentions") return "Mentions"
        if (root.channel) return root.channel.name
        if (root.guild) return root.guild.name
        return "Direct Messages"
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
            spacing: 4

            IconButton {
                glyph: "Rounded.Menu"
                tip: "Servers and channels"
                onClicked: navDrawer.open()
            }

            Text {
                Layout.fillWidth: true
                leftPadding: 8
                text: root.title
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 17
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }

            IconButton {
                visible: backend.selectedChannelId !== ""
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
            anchors.fill: parent
            sourceComponent: {
                if (root.route === "you") return accountComponent
                if (root.route === "chat" || root.route === "dm") return chatComponent
                return pendingComponent
            }
        }
    }

    Component {
        id: chatComponent
        ChatView {
            anchors.fill: parent
        }
    }

    Component {
        id: accountComponent
        AccountPanel {
            anchors.fill: parent
        }
    }

    Component {
        id: pendingComponent
        Item {
            Column {
                anchors.centerIn: parent
                spacing: 12

                Icon {
                    anchors.horizontalCenter: parent.horizontalCenter
                    name: root.route === "search" ? "Rounded.Search"
                        : root.route === "mentions" ? "Rounded.AlternateEmail" : "Rounded.Home"
                    size: 48
                    color: Theme.outline
                }
                Text {
                    text: root.title + " is not ported to QML yet"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 14
                }
            }
        }
    }

    // --------------------------------------------------------------- drawers

    Drawer {
        id: navDrawer
        edge: Qt.LeftEdge
        width: Math.min(320, root.width * 0.86)
        height: root.height
        modal: true
        overviewMode: true

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
        height: root.height
        modal: true
        overviewMode: true

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
            if (memory.memberPanel && !memberDrawer.open) memberDrawer.open()
            if (!memory.memberPanel && memberDrawer.open) memberDrawer.close()
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

        ListModel {
            id: navModel
            ListElement { routeKey: "home"; glyph: "Home"; label: "Home" }
            ListElement { routeKey: "dm"; glyph: "ChatBubble"; label: "Messages" }
            ListElement { routeKey: "search"; glyph: "Search"; label: "Search" }
            ListElement { routeKey: "mentions"; glyph: "AlternateEmail"; label: "Mentions" }
            ListElement { routeKey: "you"; glyph: "AccountCircle"; label: "You" }
        }

        Row {
            anchors.fill: parent
            anchors.bottomMargin: window.navBarInset

            Repeater {
                model: navModel

                delegate: AbstractButton {
                    width: parent.width / navModel.count
                    height: parent.height
                    down: false

                    readonly property bool selected: root.route === model.routeKey

                    onClicked: root.selectRoute(model.routeKey)

                    Column {
                        anchors.centerIn: parent
                        spacing: 2

                        Icon {
                            anchors.horizontalCenter: parent.horizontalCenter
                            name: selected ? model.glyph : "Rounded." + model.glyph
                            size: 22
                            color: selected ? Theme.primary : Theme.surfaceVariantText
                        }
                        Text {
                            text: model.label
                            color: selected ? Theme.primary : Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 11
                        }
                    }
                }
            }
        }
    }

    function selectRoute(key) {
        route = key
        navDrawer.close()
        memberDrawer.close()
        if (key === "home" || key === "dm")
            window.openHome()
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
        if (route !== "chat") {
            route = "chat"
            return true
        }
        return false
    }
}
