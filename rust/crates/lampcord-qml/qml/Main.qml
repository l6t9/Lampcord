import QtCore
import QtQuick
import QtQuick.Controls.Basic
import Lampcord

ApplicationWindow {
    id: window

    width: 1280
    height: 800
    minimumWidth: 720
    minimumHeight: 480
    visible: true
    title: "Lampcord"
    color: Theme.surfaceContainerHigh
    font.family: Theme.font

    LampcordBackend {
        id: backend
    }

    // The core is pull-based: patches are cheap to ask for and empty when
    // nothing changed, so a short timer keeps the UI one frame behind.
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

    // Home is the DM list; it reopens the DM you were last in.
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

    // Restore the last server once the guild list arrives…
    onGuildsChanged: {
        if (backend.selectedGuildId !== "" || memory.lastGuild === "")
            return
        if (guilds.some(g => g.id === memory.lastGuild))
            backend.selectGuild(memory.lastGuild)
    }

    // …or the last DM, when home was where you left off.
    property bool homeRestored: false
    onDmsChanged: {
        if (homeRestored || dms.length === 0)
            return
        homeRestored = true
        if (memory.lastGuild === "" && backend.selectedChannelId === "")
            openHome()
    }

    // …and the last channel in it, else its first text channel.
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
        anchors.fill: parent
        sourceComponent: !window.signedIn ? loginView : window.loaded ? mainView : connectingView
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
        id: mainView
        DesktopBaseplate {}
    }
}
