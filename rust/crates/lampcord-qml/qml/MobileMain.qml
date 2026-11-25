import QtCore
import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Mobile entry point. The desktop Main.qml is unchanged; the Rust launcher
// selects this file on Android (or when LAMPCORD_QML_MOBILE is set).
ApplicationWindow {
    id: window

    visible: true
    visibility: ApplicationWindow.FullScreen
    color: Theme.surfaceContainerHigh
    font.family: Theme.font

    // Qt's Android style leaves status/navigation bar insets to the app.
    readonly property int statusBarInset: Qt.platform.os === "android" ? 24 : 0
    readonly property int navBarInset: Qt.platform.os === "android" ? 16 : 0

    LampcordBackend {
        id: backend
    }

    Timer {
        interval: 33
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: backend.poll()
    }

    Settings {
        id: memory
        category: "navigation"
        property string lastGuild: ""
        property string lastChannels: "{}"
        property bool memberPanel: true
        property bool showHome: true
        property bool showDms: true
        property bool showSearch: true
        property bool showMentions: true
        property bool showYou: true

        // Phoenix-style shell defaults: full-width overlapping panels with the
        // expressive spring. These are Settings values so they persist like the
        // Compose navigation settings.
        property string panelType: "overlapping"
        property string panelAnimation: "expressive"
        property bool reduceMotion: false
        property bool hideNavLabels: false
    }

    readonly property var guilds: JSON.parse(backend.guildsJson)
    readonly property var rail: JSON.parse(backend.railJson)
    readonly property var dms: JSON.parse(backend.dmsJson)
    readonly property var channels: JSON.parse(backend.channelsJson)
    readonly property bool signedIn: backend.hasToken
    readonly property bool loaded: guilds.length > 0 || backend.connection === "ready"

    function rememberChannel(guildId, channelId) {
        const map = JSON.parse(memory.lastChannels)
        map[guildId] = channelId
        memory.lastChannels = JSON.stringify(map)
    }

    function openGuild(guildId) {
        memory.lastGuild = guildId
        backend.selectGuild(guildId)
    }

    function openChannel(channelId) {
        rememberChannel(backend.selectedGuildId !== "" ? backend.selectedGuildId : "home", channelId)
        backend.selectChannel(channelId)
    }

    function openHome() {
        memory.lastGuild = ""
        backend.selectGuild("")
        const remembered = JSON.parse(memory.lastChannels)["home"]
        if (remembered && dms.some(dm => dm.id === remembered))
            backend.selectChannel(remembered)
    }

    function openDm(channelId) {
        memory.lastGuild = ""
        if (backend.selectedGuildId !== "")
            backend.selectGuild("")
        openChannel(channelId)
    }

    onGuildsChanged: {
        if (backend.selectedGuildId !== "" || memory.lastGuild === "")
            return
        if (guilds.some(g => g.id === memory.lastGuild))
            backend.selectGuild(memory.lastGuild)
    }

    property bool homeRestored: false
    onDmsChanged: {
        if (homeRestored || dms.length === 0)
            return
        homeRestored = true
        if (memory.lastGuild === "" && backend.selectedChannelId === "")
            openHome()
    }

    onChannelsChanged: {
        if (backend.selectedChannelId !== "" || channels.length === 0)
            return
        const remembered = JSON.parse(memory.lastChannels)[backend.selectedGuildId]
        const target = channels.find(c => c.id === remembered)
            || channels.find(c => !c.category && (c.kind === 0 || c.kind === 5))
        if (target)
            backend.selectChannel(target.id)
    }

    property real lastBackAt: 0

    function exitOrWarn() {
        const now = Date.now()
        if (now - window.lastBackAt < 2000)
            Qt.quit()
        window.lastBackAt = now
        backToast.show("Press back again to exit")
    }

    function handleBack() {
        if (mainView.item && mainView.item.handleBack && mainView.item.handleBack())
            return
        exitOrWarn()
    }

    // Qt 6.12 sends back presses to the background unless a window-level
    // shortcut accepts them. A Shortcut works regardless of which touch-driven
    // item currently has active focus.
    Shortcut {
        sequences: ["Back"]
        enabled: Qt.platform.os === "android"
        onActivated: window.handleBack()
    }

    Rectangle {
        id: backToast
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: window.navBarInset + 80
        z: 100
        radius: 20
        color: Theme.surfaceContainerHighest
        opacity: 0
        visible: opacity > 0

        function show(message) {
            label.text = message
            opacity = 1
            timer.restart()
        }

        Text {
            id: label
            anchors.margins: 14
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 13
        }

        Behavior on opacity { NumberAnimation { duration: Theme.quick } }

        Timer {
            id: timer
            interval: 2000
            onTriggered: backToast.opacity = 0
        }
    }

    Loader {
        id: mainView
        anchors.fill: parent
        sourceComponent: !window.signedIn ? loginView : window.loaded ? mobileView : connectingView
    }

    Component {
        id: loginView
        LoginScreen {}
    }

    Component {
        id: connectingView
        ConnectingScreen {}
    }

    Component {
        id: mobileView
        MobileBaseplate {}
    }
}
