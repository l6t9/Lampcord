pragma Singleton
import QtCore
import QtQuick
import Lampcord

// Material 3 Expressive (M3E) tokens, generated in Rust from Matugen, a custom
// seed colour, or the current wallpaper (see src/palette.rs).
// M3's "onX" roles are spelled "xText": QML would read a property named
// onSomething as a signal handler.
QtObject {
    id: theme

    property Settings prefs: Settings {
        category: "appearance"
        property string themeMode: "system" // system | dark | light
        property string paletteVariant: "tonal_spot"
        property string customSeed: ""
        property bool matugenSync: true
        property string fontOption: "system" // system | inter | maple_mono
        property bool userbgEnabled: true
        property bool userpfpEnabled: true
        property bool freeNitroEmojis: true
        property bool logDeletedMessages: false
        property bool desktopNotifications: true
        property bool inAppToasts: true
        property bool compactMessages: false
        property bool chatBubbles: false
        property bool pureBlack: false
        property bool showContextMenuMessage: false
        property bool showChatSearch: false
        property bool showChatPins: false
        property bool showCallButton: true
        property bool noiseCancellation: true
        property real chatboxFontSize: 1.0
        property real chatboxBackgroundOpacity: 1.0
        property int chatboxBorderRadius: 16
        property bool chatboxHideUploadButton: false
        property bool chatboxHideEmojiButton: false
        property bool chatboxShowAvatar: false
        property bool silentTypingButtonEnabled: false
        property bool profile3y3: false
        property bool favoritesServerEnabled: false
        property bool marqueeText: false
        property bool cleanChannelsRemoveEmojis: true
        property bool cleanChannelsHideSymbols: true
        property bool cleanChannelsNormalizeLetters: true
        property bool cleanChannelsCapitalizeCategories: true
        property bool realmojis: false
        property bool messageLoggerIgnoreBots: false
        property bool messageLoggerIgnoreSelf: false
        property bool showMessagePreview: true
        property bool unobtrusiveUnreads: false
        property bool messageSound: true
        property bool incomingCallSound: true
        property bool syncAppearance: true
        property string panelAnimation: "minimal"
        property string panelType: "center"
        property string tapTap: "reply_or_edit"
        property bool swipeToReplyEnabled: false
        property bool autoDisplayUploads: true
        property bool autoDisplayLinks: true
        property bool showEmbeds: true
        property bool showReactions: true
        property string messageSpacingMode: "default"
        property real fontScale: 1.0
        property bool animateEmoji: true
        property string animateStickers: "always"
    }

    // Hydrates Theme.prefs from ~/.config/lampcord/settings.properties so QML
    // respects the exact same settings as the Compose client.
    function loadFromLampcordConfig(configJsonStr) {
        let cfg = {}
        try { cfg = JSON.parse(configJsonStr) } catch (e) { return }
        const b = (k, def) => cfg[k] !== undefined ? cfg[k] === "true" : def
        const f = (k, def) => cfg[k] !== undefined && !isNaN(parseFloat(cfg[k])) ? parseFloat(cfg[k]) : def
        const i = (k, def) => cfg[k] !== undefined && !isNaN(parseInt(cfg[k])) ? parseInt(cfg[k]) : def
        if (cfg.theme_mode) {
            const m = cfg.theme_mode.toLowerCase()
            prefs.themeMode = m === "auto" ? "system" : m
        }
        if (cfg.theme_palette_style)
            prefs.paletteVariant = cfg.theme_palette_style.toLowerCase()
        if (cfg.app_font)
            prefs.fontOption = cfg.app_font.toLowerCase()
        if (cfg.accent_color !== undefined)
            prefs.customSeed = cfg.material_you === "true" ? "" : cfg.accent_color
        prefs.matugenSync = b("material_you", true)
        prefs.pureBlack = b("pure_black", false)
        prefs.chatBubbles = b("chat_bubbles", false)
        prefs.compactMessages = b("compact_mode", false)
        prefs.showContextMenuMessage = b("show_context_menu_message", false)
        prefs.showChatSearch = b("show_chat_search", false)
        prefs.showChatPins = b("show_chat_pins", false)
        prefs.showCallButton = b("show_call_button", true)
        prefs.noiseCancellation = b("noise_cancellation", true)
        prefs.chatboxFontSize = f("chatbox_font_size", 1.0)
        prefs.chatboxBackgroundOpacity = f("chatbox_background_opacity", 1.0)
        prefs.chatboxBorderRadius = i("chatbox_border_radius", 16)
        prefs.chatboxHideUploadButton = b("chatbox_hide_upload_button", false)
        prefs.chatboxHideEmojiButton = b("chatbox_hide_emoji_button", false)
        prefs.chatboxShowAvatar = b("chatbox_show_avatar", false)
        prefs.silentTypingButtonEnabled = b("silent_typing_button_enabled", false)
        prefs.profile3y3 = b("profile_3y3", false)
        prefs.userbgEnabled = b("user_bg", false)
        prefs.userpfpEnabled = b("user_pfp", false)
        prefs.favoritesServerEnabled = b("favorites_server_enabled", false)
        prefs.marqueeText = b("marquee_text", false)
        prefs.cleanChannelsRemoveEmojis = b("clean_channels_remove_emojis", true)
        prefs.cleanChannelsHideSymbols = b("clean_channels_hide_symbols", true)
        prefs.cleanChannelsNormalizeLetters = b("clean_channels_normalize_letters", true)
        prefs.cleanChannelsCapitalizeCategories = b("clean_channels_capitalize_categories", true)
        prefs.freeNitroEmojis = b("free_nitro_emojis", false)
        prefs.realmojis = b("realmojis", false)
        prefs.logDeletedMessages = b("message_logger_enabled", false)
        prefs.messageLoggerIgnoreBots = b("message_logger_ignore_bots", false)
        prefs.messageLoggerIgnoreSelf = b("message_logger_ignore_self", false)
        prefs.desktopNotifications = b("notifications_enabled", true)
        prefs.showMessagePreview = b("show_message_preview", true)
        prefs.unobtrusiveUnreads = b("unobtrusive_unreads", false)
        prefs.messageSound = b("notification_sound", true)
        prefs.incomingCallSound = b("incoming_call_sound", true)
        prefs.inAppToasts = b("show_in_app_notifications", true)
        prefs.syncAppearance = b("sync_appearance", true)
        if (cfg.panel_animation) prefs.panelAnimation = cfg.panel_animation.toLowerCase()
        if (cfg.panel_type) prefs.panelType = cfg.panel_type.toLowerCase()
        if (cfg.taptap_action) prefs.tapTap = cfg.taptap_action.toLowerCase()
        prefs.swipeToReplyEnabled = b("swipe_to_reply_enabled", false)
        if (cfg.message_spacing_mode) prefs.messageSpacingMode = cfg.message_spacing_mode.toLowerCase()
        prefs.fontScale = f("font_scale", 1.0)
        if (cfg.animate_stickers) prefs.animateStickers = cfg.animate_stickers.toLowerCase()
        applyPalette()
    }

    function applyPalette() {
        Palette.configure(prefs.paletteVariant, prefs.customSeed, prefs.matugenSync)
    }

    Component.onCompleted: applyPalette()

    readonly property bool dark: prefs.themeMode === "dark"
        ? true
        : prefs.themeMode === "light"
            ? false
            : Application.styleHints.colorScheme !== Qt.ColorScheme.Light
    readonly property var parsedPalette: JSON.parse(Palette.schemeJson)
    readonly property var scheme: parsedPalette[dark ? "dark" : "light"]
    readonly property string activeSeed: parsedPalette.seed || "#6750a4"

    readonly property color primary: scheme.primary
    readonly property color primaryText: scheme.onPrimary
    readonly property color primaryContainer: scheme.primaryContainer
    readonly property color primaryContainerText: scheme.onPrimaryContainer
    readonly property color secondary: scheme.secondary
    readonly property color secondaryText: scheme.onSecondary
    readonly property color secondaryContainer: scheme.secondaryContainer
    readonly property color secondaryContainerText: scheme.onSecondaryContainer
    readonly property color tertiary: scheme.tertiary
    readonly property color tertiaryContainer: scheme.tertiaryContainer
    readonly property color error: scheme.error
    readonly property color errorText: scheme.onError
    readonly property color errorContainer: scheme.errorContainer
    readonly property color errorContainerText: scheme.onErrorContainer

    readonly property color background: scheme.background
    readonly property color surface: scheme.surface
    readonly property color surfaceContainerLowest: scheme.surfaceContainerLowest
    readonly property color surfaceContainerLow: scheme.surfaceContainerLow
    readonly property color surfaceContainer: scheme.surfaceContainer
    readonly property color surfaceContainerHigh: scheme.surfaceContainerHigh
    readonly property color surfaceContainerHighest: scheme.surfaceContainerHighest
    readonly property color surfaceVariant: scheme.surfaceVariant
    readonly property color surfaceText: scheme.onSurface
    readonly property color surfaceVariantText: scheme.onSurfaceVariant
    readonly property color outline: scheme.outline
    readonly property color outlineVariant: scheme.outlineVariant

    readonly property string font: prefs.fontOption === "inter"
        ? "Inter"
        : prefs.fontOption === "maple_mono"
            ? "Maple Mono"
            : "sans-serif"
    readonly property string monoFont: prefs.fontOption === "maple_mono" ? "Maple Mono" : "monospace"

    // Motion: the Compose client's "default spatial" spring, approximated.
    readonly property int quick: 160
    readonly property int medium: 260

    // The wallpaper or Matugen file can change while we run.
    property Timer wallpaperWatch: Timer {
        interval: 2000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: Palette.refresh()
    }

    function alpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a)
    }

    // Compose's Color.lerp, used by the mobile panels to tint the root surface
    // while a side panel opens.
    function lerp(a, b, amount) {
        const t = Math.max(0, Math.min(1, amount))
        return Qt.rgba(a.r + (b.r - a.r) * t,
                       a.g + (b.g - a.g) * t,
                       a.b + (b.b - a.b) * t,
                       a.a + (b.a - a.a) * t)
    }
}
