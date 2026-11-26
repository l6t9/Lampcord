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

    readonly property int statusBarInset: 0
    readonly property int navBarInset: 0

    LampcordBackend {
        id: backend
        Component.onCompleted: {
            Theme.loadFromLampcordConfig(backend.configJson)
            backend.logDeletedMessages = Theme.prefs.logDeletedMessages
            backend.desktopNotifications = Theme.prefs.desktopNotifications
        }
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

    Connections {
        target: backend
        function onToastSeqChanged() {
            toastBanner.show(backend.toastTitle, backend.toastBody, backend.toastChannelId)
        }
    }

    Settings {
        id: memory
        category: "navigation"
        property string lastGuild: ""
        property string lastChannels: "{}"
        property bool memberPanel: true
    }

    property bool showingFriends: false
    signal mentionUser(string userId)

    readonly property var guilds: JSON.parse(backend.guildsJson)
    readonly property var rail: JSON.parse(backend.railJson)
    readonly property var dms: JSON.parse(backend.dmsJson)
    readonly property var channels: JSON.parse(backend.channelsJson)
    readonly property var relationships: JSON.parse(backend.relationshipsJson)
    readonly property bool signedIn: backend.hasToken
    readonly property bool loaded: guilds.length > 0 || backend.connection === "ready"

    function rememberChannel(guildId, channelId) {
        const map = JSON.parse(memory.lastChannels)
        map[guildId] = channelId
        memory.lastChannels = JSON.stringify(map)
    }

    function openGuild(guildId) {
        showingFriends = false
        memory.lastGuild = guildId
        backend.selectGuild(guildId)
        if (guildId !== "")
            backend.fetchGuildSettings(guildId)
    }

    function openChannel(channelId) {
        showingFriends = false
        threadPanelVisible = false
        rememberChannel(backend.selectedGuildId !== "" ? backend.selectedGuildId : "home", channelId)
        backend.selectChannel(channelId)
    }

    property bool threadPanelVisible: false

    function toggleThreads() {
        threadPanelVisible = !threadPanelVisible
        if (threadPanelVisible)
            backend.fetchThreads()
    }

    // Home is the DM list; it reopens the DM you were last in.
    function openHome() {
        showingFriends = false
        memory.lastGuild = ""
        backend.selectGuild("")
        const remembered = JSON.parse(memory.lastChannels)["home"]
        if (remembered && dms.some(dm => dm.id === remembered))
            backend.selectChannel(remembered)
    }

    function openFriends() {
        memory.lastGuild = ""
        if (backend.selectedGuildId !== "")
            backend.selectGuild("")
        showingFriends = true
    }

    function openDm(channelId) {
        showingFriends = false
        memory.lastGuild = ""
        if (backend.selectedGuildId !== "")
            backend.selectGuild("")
        openChannel(channelId)
    }

    function openProfile(userId, name, avatar, decoration, status, activity, anchorX, anchorY) {
        profilePopup.openForUser(userId, name, avatar, decoration, status, activity, anchorX, anchorY)
    }

    function openSettings(category) {
        if (category)
            settingsScreen.category = category
        settingsScreen.open()
    }

    function openJoinServer() {
        joinServerDialog.open()
    }

    function openServerSettings(guildId, guildName) {
        serverSettingsDialog.openForGuild(guildId, guildName)
    }

    function openImage(url, filename) {
        mediaViewer.openImage(url, filename)
    }

    function openVideo(url, filename) {
        mediaViewer.openVideo(url, filename)
    }

    function copyText(text) {
        clipboardHelper.text = text
        clipboardHelper.selectAll()
        clipboardHelper.copy()
    }

    TextEdit {
        id: clipboardHelper
        visible: false
    }

    // Restore the last server once the guild list arrives…
    onGuildsChanged: {
        if (backend.selectedGuildId !== "" || memory.lastGuild === "")
            return
        if (guilds.some(g => g.id === memory.lastGuild)) {
            backend.selectGuild(memory.lastGuild)
            backend.fetchGuildSettings(memory.lastGuild)
        }
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

    ProfilePopup {
        id: profilePopup
        parent: Overlay.overlay
    }

    SettingsScreen {
        id: settingsScreen
        parent: Overlay.overlay
    }

    JoinServerDialog {
        id: joinServerDialog
        parent: Overlay.overlay
    }

    ServerSettingsDialog {
        id: serverSettingsDialog
        parent: Overlay.overlay
    }

    MediaViewer {
        id: mediaViewer
        parent: Overlay.overlay
    }

    ToastBanner {
        id: toastBanner
        parent: Overlay.overlay
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: 16
        z: 100
    }
}
