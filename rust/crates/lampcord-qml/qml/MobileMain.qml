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

    focus: true
    Keys.enabled: true
    Keys.onBackPressed: {
        if (mainView.item && mainView.item.handleBack && mainView.item.handleBack())
            event.accepted = true
    }
}
