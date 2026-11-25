import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Effects
import Lampcord

// Phoenix-style mobile shell. One state drives three overlapping panels:
//   panel =  1  sidebar (start)
//   panel =  0  chat / current route (center)
//   panel = -1  members (end)
// The bottom bar is an overlay and is hidden while the centre chat is open,
// exactly like the Compose mobile baseplate.
Item {
    id: root
    anchors.fill: parent

    property string route: "chat"
    property string tab: "home"
    property var stack: [{ route: "chat", tab: "home" }]

    property int panel: 0
    property real panelOffset: 0
    property int lastPanel: 0
    property real navAmount: 0
    property bool dragging: false

    readonly property var guild: window.guilds.find(g => g.id === backend.selectedGuildId) || null
    readonly property var dm: window.dms.find(d => d.id === backend.selectedChannelId) || null
    readonly property var channel: root.dm || window.channels.find(c => c.id === backend.selectedChannelId) || null

    readonly property bool overlapping: memory.panelType === "overlapping"
    readonly property bool expressive: memory.panelAnimation === "expressive"
    readonly property bool instantMotion: memory.reduceMotion
    readonly property real railWidth: 72
    // Phoenix drawer width: locked at 332dp on wide screens, otherwise
    // min(372dp, width - 32). The channel list is the drawer minus the rail.
    readonly property real drawerWidth: Math.min(372, Math.max(root.railWidth, root.width - 32))
    readonly property real channelListWidth: Math.max(0, root.drawerWidth - root.railWidth)
    readonly property real panelWidth: overlapping ? root.width : root.drawerWidth
    readonly property real targetOffset: panel === 1 ? panelWidth : panel === -1 ? -panelWidth : 0
    readonly property real panelProgress: root.width <= 0 ? 0
        : Math.max(-1, Math.min(1, panelOffset / panelWidth))
    readonly property real panelAbs: Math.abs(panelProgress)
    readonly property real endProgress: Math.max(0, -panelProgress)
    readonly property real shrinkFactor: root.overlapping && root.panelProgress < 0
        ? 0 : root.expressive ? 0.06 : 0.04

    readonly property bool chatRoute: root.route === "chat"
    readonly property bool tabRoute: ["chat", "dm", "search", "mentions"].indexOf(root.route) !== -1
    readonly property real navHeight: 80 + window.navBarInset
    readonly property real visibleNavHeight: navHeight * navAmount

    readonly property color panelColor: Theme.dark ? Theme.surfaceContainerLowest : Theme.surfaceContainer
    readonly property color activePanelColor: {
        if (!root.chatRoute)
            return Theme.surfaceContainerLow
        if (panelProgress > 0)
            return Theme.lerp(Theme.surface, root.panelColor, panelProgress)
        if (panelProgress < 0 && !root.overlapping)
            return Theme.lerp(Theme.surface, root.panelColor, -panelProgress)
        return Theme.surface
    }

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

    readonly property int selectedTab: tabs.findIndex(item => {
        if (item.routeKey === "home")
            return root.chatRoute && root.tab === "home"
        if (item.routeKey === "dm")
            return root.chatRoute && root.tab === "dm"
        return !root.chatRoute && root.tab === item.routeKey
    })

    readonly property string title: {
        if (root.route === "dm") return "Direct Messages"
        if (root.route === "you") return "You"
        if (root.route === "search") return "Search"
        if (root.route === "mentions") return "Mentions"
        if (root.dm) return root.dm.name
        if (root.channel) return root.channel.name
        if (root.guild) return root.guild.name
        return "Direct Messages"
    }

    readonly property int totalMentions: {
        let count = 0
        for (const dm of window.dms)
            count += dm.mentions || 0
        for (const item of window.channels)
            count += item.mentions || 0
        return count
    }

    // ------------------------------------------------------------------ state

    function syncNav() {
        let target = 0
        if (root.route === "chat")
            target = root.panel === 1 ? 1 : 0
        else if (root.tabRoute)
            target = 1
        root.navAmount = target
    }

    function setPanel(value) {
        if (value !== 0)
            lastPanel = value
        root.panel = value
        root.panelOffset = root.targetOffset
        if (value !== 0 && Qt.platform.os === "android") {
            Qt.inputMethod.hide()
            root.forceActiveFocus()
        }
        syncNav()
    }

    function openStart() { setPanel(1) }
    function openEnd() {
        if (root.chatRoute)
            setPanel(-1)
    }
    function closePanel() { setPanel(0) }

    function pushRoute(key) {
        root.stack = root.stack.concat([{ route: key, tab: root.tab }])
        root.route = key
        closePanel()
    }

    function replaceRoute(key, newTab) {
        if (newTab && newTab !== "")
            root.tab = newTab
        root.stack = [{ route: key, tab: root.tab }]
        root.route = key
        closePanel()
    }

    function selectRoute(key) {
        if (key === "home") {
            if (root.route !== "chat")
                root.replaceRoute("chat", "home")
            else
                root.tab = "home"
            if (root.panel === 1)
                root.closePanel()
            else
                root.openStart()
            return
        }
        root.replaceRoute(key, key)
    }

    function handleBack() {
        if (root.panel !== 0) {
            root.closePanel()
            return true
        }
        const view = contentLoader.item
        if (view && view.handleBack && view.handleBack())
            return true
        if (root.stack.length > 1) {
            const previous = root.stack[root.stack.length - 2]
            root.stack = root.stack.slice(0, -1)
            root.route = previous.route
            root.tab = previous.tab
            return true
        }
        if (root.route !== "chat") {
            root.replaceRoute("chat")
            return true
        }
        return false
    }

    function beginDrag(base) {
        root.dragging = true
        root.suppressClick = false
        root.dragBase = base
        root.dragStartTime = Date.now()
        root.dragDelta = 0
    }

    function updateDragDelta(delta) {
        if (!root.dragging)
            return
        root.dragDelta = delta
        if (Math.abs(delta) > 4)
            root.suppressClick = true

        const wanted = root.dragBase + delta
        if (root.panel === 1) {
            root.panelOffset = Math.max(0, Math.min(panelWidth, wanted))
            root.navAmount = Math.max(0, Math.min(1, root.panelOffset / panelWidth))
        } else if (root.panel === -1) {
            root.panelOffset = Math.min(0, Math.max(-panelWidth, wanted))
            root.navAmount = 0
        } else {
            root.panelOffset = Math.max(-panelWidth, Math.min(panelWidth, wanted))
            root.navAmount = root.panelOffset > 0
                ? Math.max(0, Math.min(1, root.panelOffset / panelWidth)) : 0
        }
    }

    function finishDrag(velocity) {
        if (!root.dragging)
            return
        const delta = root.dragDelta
        const elapsed = Math.max(1, Date.now() - root.dragStartTime)
        const measured = Math.abs(velocity) > 1 ? velocity : delta / elapsed * 1000
        root.dragging = false

        // Phoenix main-tabs gesture: activate after 12dp, snap at half the
        // drawer width, and commit a fling at 240dp/s. Starting from Center,
        // either direction is allowed; an open panel only closes.
        let target = 0
        if (root.panel === 1) {
            target = (delta < -panelWidth * 0.5 || (measured < -240 && delta < -20)) ? 0 : 1
        } else if (root.panel === -1) {
            target = (delta > panelWidth * 0.5 || (measured > 240 && delta > 20)) ? 0 : -1
        } else if (measured > 240 && delta > 0 || root.panelOffset > panelWidth * 0.5) {
            target = 1
        } else if (measured < -240 && delta < 0 || root.panelOffset < -panelWidth * 0.5) {
            target = -1
        }
        setPanel(target)
    }

    property real dragBase: 0
    property real dragDelta: 0
    property real dragStartTime: 0
    property bool suppressClick: false

    onPanelChanged: root.panelOffset = root.targetOffset
    onRouteChanged: syncNav()
    onWidthChanged: if (!dragging) root.panelOffset = root.targetOffset
    Component.onCompleted: {
        root.panelOffset = root.targetOffset
        syncNav()
    }

    Connections {
        target: backend
        function onSelectedChannelIdChanged() {
            if (root.panel === 1)
                root.closePanel()
        }
    }

    Behavior on panelOffset {
        enabled: !root.dragging
        // Android Phoenix uses 250ms open / 200ms close timing with a clamped
        // spring, so a cubic settle is both closer and noticeably snappier
        // than Qt's long spring tail.
        NumberAnimation {
            duration: root.panel === 0 ? 200 : 250
            easing.type: Easing.OutCubic
        }
    }

    Behavior on navAmount {
        enabled: !root.dragging
        NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
    }

    // --------------------------------------------------------------- surfaces

    Rectangle {
        anchors.fill: parent
        color: root.activePanelColor
    }

    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: window.statusBarInset
        color: Theme.surfaceContainer
        z: 20
    }

    // ------------------------------------------------------------ start panel

    Item {
        id: startPanel
        y: window.statusBarInset
        width: root.panelWidth
        height: root.height - window.statusBarInset
        z: 1
        visible: root.panelProgress > 0.001 || root.panel === 1
        enabled: visible
        opacity: visible ? 1 : 0
        transformOrigin: Item.Top
        scale: root.expressive && !root.instantMotion
            ? 0.92 + (Math.max(0, root.panelProgress) * 0.08) : 1
        transform: Translate {
            x: root.expressive && !root.instantMotion
                ? (Math.max(0, root.panelProgress) - 1) * root.panelWidth * 0.3 : 0
        }

        Rectangle {
            anchors.fill: parent
            color: Theme.surfaceContainerLowest
        }

        Row {
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.bottomMargin: root.visibleNavHeight
            width: root.drawerWidth

            GuildRail {
                width: root.railWidth
                height: parent.height
                mobile: true
            }

            Rectangle {
                width: root.channelListWidth
                height: parent.height
                radius: root.overlapping ? 0 : 16
                color: Theme.surface
                clip: true

                ChannelList {
                    anchors.fill: parent
                    mobile: true
                }
            }
        }
    }

    // ------------------------------------------------------------ center card

    Item {
        id: centerShell
        y: window.statusBarInset
        width: root.width
        height: root.height - window.statusBarInset
        z: 2
        x: {
            if (root.panel === 1)
                return Math.max(0, root.panelOffset)
            if (root.panel === -1 && !root.overlapping)
                return Math.min(0, root.panelOffset)
            return 0
        }
        scale: root.instantMotion ? 1 : 1 - (root.panelAbs * root.shrinkFactor)
        transformOrigin: {
            if (root.panelProgress > 0) return Item.TopRight
            if (root.panelProgress < 0) return Item.TopLeft
            return Item.Top
        }

        RectangularShadow {
            anchors.fill: centerCard
            radius: centerCard.radius
            blur: root.expressive ? 12 : 6
            offset.y: 3
            color: Qt.rgba(0, 0, 0, 0.3)
            visible: root.panelAbs > 0.01
        }

        Rectangle {
            id: centerCard
            anchors.fill: parent
            // Phoenix app frame: square while centered, then the chat card
            // rounds up to 24dp as the start drawer slides in.
            radius: {
                if (root.instantMotion || root.overlapping && root.panelProgress < 0)
                    return 0
                return Math.min(Math.max(0, root.panelOffset), 24)
            }
            color: root.chatRoute ? Theme.surfaceContainerLow : Theme.surface
            border.width: 1
            border.color: Theme.alpha(Theme.outlineVariant, 0.5)
            clip: true

            // ------------------------------------------------------- top app bar

            Rectangle {
                id: topBar
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                height: root.chatRoute ? 56 : 0
                visible: height > 0
                color: Theme.surfaceContainer
                z: 2

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: Theme.outlineVariant
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 4
                    anchors.rightMargin: 4
                    spacing: 2

                    IconButton {
                        glyph: "Rounded.ArrowBack"
                        iconColor: Theme.surfaceVariantText
                        tip: root.guild ? "Channels in " + root.guild.name : "Direct messages"
                        onClicked: root.openStart()
                    }

                    Item { width: 4; height: 1 }

                    Row {
                        Layout.fillWidth: true
                        Layout.preferredHeight: parent.height
                        spacing: 8

                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: root.dm ? !root.dm.group : root.channel && root.channel.kind === 5
                            name: root.dm ? "Group" : root.channel && root.channel.kind === 5 ? "Campaign" : "Tag"
                            size: 22
                            color: Theme.surfaceVariantText
                        }
                        Avatar {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: root.dm && !root.dm.group
                            size: 28
                            name: root.dm ? root.dm.name : ""
                            source: root.dm ? root.dm.avatar : ""
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 36
                            text: root.title
                            color: Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 18
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight

                            TapHandler {
                                onTapped: root.openEnd()
                            }
                        }
                    }

                    IconButton {
                        visible: backend.selectedChannelId !== ""
                        glyph: "Rounded.Search"
                        iconColor: Theme.surfaceVariantText
                        tip: "Search in channel"
                        onClicked: root.pushRoute("search")
                    }
                    IconButton {
                        id: pinsButton
                        visible: backend.selectedChannelId !== ""
                        glyph: "PushPin"
                        iconColor: Theme.surfaceVariantText
                        tip: "Pinned messages"
                        onClicked: {
                            const view = contentLoader.item
                            if (view && view.openPins)
                                view.openPins()
                        }
                    }
                    IconButton {
                        visible: backend.selectedChannelId !== ""
                        glyph: "Rounded.Group"
                        iconColor: root.panel === -1 ? Theme.primary : Theme.surfaceVariantText
                        tip: root.panel === -1 ? "Close member list" : "Open member list"
                        onClicked: root.panel === -1 ? root.closePanel() : root.openEnd()
                    }
                }
            }

            // --------------------------------------------------------- content

            Item {
                anchors.top: topBar.bottom
                anchors.bottom: parent.bottom
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottomMargin: root.chatRoute ? 0 : root.visibleNavHeight
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

            // ----------------------------------------------------------- scrim

            Rectangle {
                anchors.fill: parent
                z: 10
                visible: root.panelAbs > 0.01
                color: Qt.rgba(0, 0, 0, 0.5 * root.panelAbs)

                MouseArea {
                    anchors.fill: parent
                    enabled: parent.visible
                    onClicked: {
                        if (!root.suppressClick)
                            root.closePanel()
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------- end panel

    Item {
        id: endPanel
        y: window.statusBarInset
        width: root.panelWidth
        height: root.height - window.statusBarInset
        z: 3
        x: {
            const inset = root.overlapping ? 0 : 8
            const base = root.width - width - inset
            if (!root.overlapping)
                return base + (root.expressive ? root.panelWidth * (1 - root.endProgress) * 0.3 : 0)
            return base + (root.panelWidth * (1 - root.endProgress))
        }
        visible: root.panelProgress < -0.001 || root.panel === -1
        enabled: visible
        opacity: visible ? 1 : 0

        RectangularShadow {
            anchors.fill: endSurface
            radius: 0
            blur: 8
            offset.x: -3
            color: Qt.rgba(0, 0, 0, 0.35)
            visible: root.overlapping && parent.visible
        }

        Rectangle {
            id: endSurface
            anchors.fill: parent
            radius: root.overlapping ? 0 : 16
            color: root.overlapping ? Theme.surface : Theme.background
            clip: true

            MemberPanel {
                anchors.fill: parent
                mobile: true
                backVisible: root.overlapping
                onBackClicked: root.closePanel()
            }
        }
    }

    // -------------------------------------------------------- panel gestures

    // Kotlin attaches Modifier.draggable(Orientation.Horizontal) to the whole
    // shell. DragHandler is the QML equivalent: it stays passive until a real
    // horizontal move happens, so message scrolling and taps still reach the
    // views below. It sits above the panels so a drag can also close them.
    Item {
        anchors.fill: parent
        anchors.topMargin: window.statusBarInset
        anchors.bottomMargin: root.visibleNavHeight
        z: 5
        visible: root.chatRoute

        DragHandler {
            id: panelDrag
            target: null
            acceptedButtons: Qt.LeftButton
            acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchScreen
            xAxis.enabled: true
            yAxis.enabled: false
            dragThreshold: 12
            grabPermissions: PointerHandler.CanTakeOverFromItems |
                              PointerHandler.CanTakeOverFromHandlersOfSameType

            onActiveChanged: {
                if (active)
                    root.beginDrag(root.panelOffset)
                else
                    root.finishDrag(centroid.velocity.x)
            }

            onTranslationChanged: {
                if (active)
                    root.updateDragDelta(activeTranslation.x)
            }
        }
    }

    // ------------------------------------------------------------- nav overlay

    Item {
        id: bottomNav
        width: parent.width
        height: root.navHeight
        y: root.height - height + (1 - root.navAmount) * height
        z: 12
        visible: root.navAmount > 0.001
        opacity: root.navAmount

        RectangularShadow {
            anchors.fill: navSurface
            radius: 0
            blur: 10
            offset.y: -3
            color: Qt.rgba(0, 0, 0, 0.22)
        }

        Rectangle {
            id: navSurface
            anchors.fill: parent
            color: Theme.surface
        }

        Item {
            id: navItems
            anchors.fill: parent
            anchors.bottomMargin: window.navBarInset
            readonly property real itemWidth: width / Math.max(1, root.tabs.length)

            Row {
                anchors.fill: parent

                Repeater {
                    model: root.tabs

                    delegate: AbstractButton {
                        required property var modelData
                        required property int index
                        width: navItems.itemWidth
                        height: parent.height
                        down: false

                        readonly property bool selected: index === root.selectedTab

                        onClicked: root.selectRoute(modelData.routeKey)

                        Column {
                            anchors.centerIn: parent
                            spacing: 2

                            Icon {
                                anchors.horizontalCenter: parent.horizontalCenter
                                name: selected ? modelData.glyph : "Rounded." + modelData.glyph
                                size: 24
                                color: selected ? Theme.primary : Theme.surfaceVariantText
                            }
                            Text {
                                visible: !memory.hideNavLabels || selected
                                text: modelData.label
                                color: selected ? Theme.primary : Theme.surfaceVariantText
                                font.family: Theme.font
                                font.pixelSize: 12
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
    }

    // ------------------------------------------------------------------ views

    Component {
        id: chatComponent
        ChatView {
            anchors.fill: parent
            compact: true
            floatingComposer: true
        }
    }

    Component {
        id: dmsComponent
        MobileDms {
            anchors.fill: parent
            onOpenServers: root.openStart()
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
}
