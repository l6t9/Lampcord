package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.ProtoReader2
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.model.TextReplaceRule
import me.lampu.lampcord.shared.settings.ThemeMode
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlin.time.Duration.Companion.milliseconds

class SettingsStore(
    private val userApi: UserApi
) {
    var userSettings by mutableStateOf<UserSettings?>(null)

    private var _language by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.language)
    var language: String
        get() = _language
        set(value) {
            _language = value
            me.lampu.lampcord.shared.settings.Settings.shared.language = value
        }

    private val json = Json { ignoreUnknownKeys = true }

    var textReplaceRules by mutableStateOf(
        try {
            json.decodeFromString<List<TextReplaceRule>>(me.lampu.lampcord.shared.settings.Settings.shared.textReplaceJson)
        } catch (e: Exception) { emptyList() }
    )

    fun updateTextReplaceRules(rules: List<TextReplaceRule>) {
        textReplaceRules = rules
        me.lampu.lampcord.shared.settings.Settings.shared.textReplaceJson = json.encodeToString(rules)
    }
    
    private var _pureBlack by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.pureBlack)
    var pureBlack: Boolean
        get() = _pureBlack
        set(value) {
            _pureBlack = value
            me.lampu.lampcord.shared.settings.Settings.shared.pureBlack = value
        }

    private var _themeMode by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.themeMode)
    var themeMode: me.lampu.lampcord.shared.settings.ThemeMode
        get() = _themeMode
        set(value) {
            _themeMode = value
            me.lampu.lampcord.shared.settings.Settings.shared.themeMode = value
        }

    private var _themePaletteStyle by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.themePaletteStyle)
    var themePaletteStyle: me.lampu.lampcord.shared.settings.ThemePaletteStyle
        get() = _themePaletteStyle
        set(value) {
            _themePaletteStyle = value
            me.lampu.lampcord.shared.settings.Settings.shared.themePaletteStyle = value
        }

    private var _appFont by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.appFont)
    var appFont: me.lampu.lampcord.shared.settings.FontOption
        get() = _appFont
        set(value) {
            _appFont = value
            me.lampu.lampcord.shared.settings.Settings.shared.appFont = value
        }

    private var _accentColor by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.accentColor)
    var accentColor: String
        get() = _accentColor
        set(value) {
            _accentColor = value
            me.lampu.lampcord.shared.settings.Settings.shared.accentColor = value
        }

    private var _materialYou by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.materialYou)
    var materialYou: Boolean
        get() = _materialYou
        set(value) {
            _materialYou = value
            me.lampu.lampcord.shared.settings.Settings.shared.materialYou = value
        }

    private var _showHiddenChannels by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showHiddenChannels)
    var showHiddenChannels: Boolean
        get() = _showHiddenChannels
        set(value) {
            _showHiddenChannels = value
            me.lampu.lampcord.shared.settings.Settings.shared.showHiddenChannels = value
        }

    private var _syncAppearance by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.syncAppearance)
    var syncAppearance: Boolean
        get() = _syncAppearance
        set(value) {
            _syncAppearance = value
            me.lampu.lampcord.shared.settings.Settings.shared.syncAppearance = value
            // Turning sync on should adopt Discord's current theme immediately,
            // not wait for the next USER_SETTINGS_UPDATE.
            if (value) applyDiscordAppearance(userSettings?.theme)
        }

    /** Applies Discord's theme to the local appearance settings when syncing is enabled. */
    fun applyDiscordAppearance(theme: String?) {
        mapDiscordAppearance(theme, syncAppearance)?.let { appearance ->
            themeMode = appearance.themeMode
            pureBlack = appearance.pureBlack
        }
    }

    private var _panelAnimation by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.panelAnimation)
    var panelAnimation: me.lampu.lampcord.shared.settings.PanelAnimation
        get() = _panelAnimation
        set(value) {
            _panelAnimation = value
            me.lampu.lampcord.shared.settings.Settings.shared.panelAnimation = value
        }

    private var _panelType by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.panelType)
    var panelType: me.lampu.lampcord.shared.settings.PanelType
        get() = _panelType
        set(value) {
            _panelType = value
            me.lampu.lampcord.shared.settings.Settings.shared.panelType = value
        }

    private var _hideNavLabels by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.hideNavLabels)
    var hideNavLabels: Boolean
        get() = _hideNavLabels
        set(value) {
            _hideNavLabels = value
            me.lampu.lampcord.shared.settings.Settings.shared.hideNavLabels = value
        }

    private var _showChatSearch by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showChatSearch)
    var showChatSearch: Boolean
        get() = _showChatSearch
        set(value) {
            _showChatSearch = value
            me.lampu.lampcord.shared.settings.Settings.shared.showChatSearch = value
        }

    private var _showChatPins by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showChatPins)
    var showChatPins: Boolean
        get() = _showChatPins
        set(value) {
            _showChatPins = value
            me.lampu.lampcord.shared.settings.Settings.shared.showChatPins = value
        }

    private var _showCallButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showCallButton)
    var showCallButton: Boolean
        get() = _showCallButton
        set(value) {
            _showCallButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.showCallButton = value
        }

    private var _showNavHome by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavHome)
    var showNavHome: Boolean
        get() = _showNavHome
        set(value) {
            _showNavHome = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavHome = value
        }

    private var _showNavDm by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavDm)
    var showNavDm: Boolean
        get() = _showNavDm
        set(value) {
            _showNavDm = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavDm = value
        }

    private var _showNavFriends by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavFriends)
    var showNavFriends: Boolean
        get() = _showNavFriends
        set(value) {
            _showNavFriends = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavFriends = value
        }

    private var _showNavSearch by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavSearch)
    var showNavSearch: Boolean
        get() = _showNavSearch
        set(value) {
            _showNavSearch = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavSearch = value
        }

    private var _showNavMentions by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavMentions)
    var showNavMentions: Boolean
        get() = _showNavMentions
        set(value) {
            _showNavMentions = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavMentions = value
        }

    private var _showNavSettings by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showNavSettings)
    var showNavSettings: Boolean
        get() = _showNavSettings
        set(value) {
            _showNavSettings = value
            me.lampu.lampcord.shared.settings.Settings.shared.showNavSettings = value
        }

    private var _useYouBar by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.useYouBar)
    var useYouBar: Boolean
        get() = _useYouBar
        set(value) {
            _useYouBar = value
            me.lampu.lampcord.shared.settings.Settings.shared.useYouBar = value
        }

    private var _navTabsOrderJson by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.navTabsOrderJson)
    var navTabsOrderJson: String
        get() = _navTabsOrderJson
        set(value) {
            _navTabsOrderJson = value
            me.lampu.lampcord.shared.settings.Settings.shared.navTabsOrderJson = value
        }

    private var _secretTabEnabled by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.secretTabEnabled)
    var secretTabEnabled: Boolean
        get() = _secretTabEnabled
        set(value) {
            _secretTabEnabled = value
            me.lampu.lampcord.shared.settings.Settings.shared.secretTabEnabled = value
        }

    private var _profile3y3 by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.profile3y3)
    var profile3y3: Boolean
        get() = _profile3y3
        set(value) {
            _profile3y3 = value
            me.lampu.lampcord.shared.settings.Settings.shared.profile3y3 = value
        }

    private var _userBg by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.userBg)
    var userBg: Boolean
        get() = _userBg
        set(value) {
            _userBg = value
            me.lampu.lampcord.shared.settings.Settings.shared.userBg = value
        }

    private var _userPfp by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.userPfp)
    var userPfp: Boolean
        get() = _userPfp
        set(value) {
            _userPfp = value
            me.lampu.lampcord.shared.settings.Settings.shared.userPfp = value
        }

    private var _checkForUpdates by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.checkForUpdates)
    var checkForUpdates: Boolean
        get() = _checkForUpdates
        set(value) {
            _checkForUpdates = value
            me.lampu.lampcord.shared.settings.Settings.shared.checkForUpdates = value
        }

    private var _updateNotifications by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.updateNotifications)
    var updateNotifications: Boolean
        get() = _updateNotifications
        set(value) {
            _updateNotifications = value
            me.lampu.lampcord.shared.settings.Settings.shared.updateNotifications = value
        }

    private var _chatboxBackgroundOpacity by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxBackgroundOpacity)
    var chatboxBackgroundOpacity: Float
        get() = _chatboxBackgroundOpacity
        set(value) {
            _chatboxBackgroundOpacity = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxBackgroundOpacity = value
        }

    private var _chatboxBorderRadius by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxBorderRadius)
    var chatboxBorderRadius: Int
        get() = _chatboxBorderRadius
        set(value) {
            _chatboxBorderRadius = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxBorderRadius = value
        }

    private var _chatboxHideUploadButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideUploadButton)
    var chatboxHideUploadButton: Boolean
        get() = _chatboxHideUploadButton
        set(value) {
            _chatboxHideUploadButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideUploadButton = value
        }

    private var _chatboxHideEmojiButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideEmojiButton)
    var chatboxHideEmojiButton: Boolean
        get() = _chatboxHideEmojiButton
        set(value) {
            _chatboxHideEmojiButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideEmojiButton = value
        }

    private var _mobileShowChannelListButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.mobileShowChannelListButton)
    var mobileShowChannelListButton: Boolean
        get() = _mobileShowChannelListButton
        set(value) {
            _mobileShowChannelListButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.mobileShowChannelListButton = value
        }

    private var _mobileShowMemberListButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.mobileShowMemberListButton)
    var mobileShowMemberListButton: Boolean
        get() = _mobileShowMemberListButton
        set(value) {
            _mobileShowMemberListButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.mobileShowMemberListButton = value
        }

    private var _chatboxHideVoiceButton by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideVoiceButton)
    var chatboxHideVoiceButton: Boolean
        get() = _chatboxHideVoiceButton
        set(value) {
            _chatboxHideVoiceButton = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxHideVoiceButton = value
        }

    private var _chatboxShowAvatar by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxShowAvatar)
    var chatboxShowAvatar: Boolean
        get() = _chatboxShowAvatar
        set(value) {
            _chatboxShowAvatar = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxShowAvatar = value
        }

    private var _silentTyping by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.silentTyping)
    var silentTyping: Boolean
        get() = _silentTyping
        set(value) {
            _silentTyping = value
            me.lampu.lampcord.shared.settings.Settings.shared.silentTyping = value
        }

    private var _silentTypingButtonEnabled by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.silentTypingButtonEnabled)
    var silentTypingButtonEnabled: Boolean
        get() = _silentTypingButtonEnabled
        set(value) {
            _silentTypingButtonEnabled = value
            me.lampu.lampcord.shared.settings.Settings.shared.silentTypingButtonEnabled = value
            if (!value) silentTyping = false
        }

    private var _chatboxFontSize by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatboxFontSize)
    var chatboxFontSize: Float
        get() = _chatboxFontSize
        set(value) {
            _chatboxFontSize = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatboxFontSize = value
        }

    private var _fontScale by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.fontScale)
    var fontScale: Float
        get() = _fontScale
        set(value) {
            _fontScale = value
            me.lampu.lampcord.shared.settings.Settings.shared.fontScale = value
        }

    private var _customFontPath by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.customFontPath)
    var customFontPath: String
        get() = _customFontPath
        set(value) {
            _customFontPath = value
            me.lampu.lampcord.shared.settings.Settings.shared.customFontPath = value
        }

    private var _customFontName by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.customFontName)
    var customFontName: String
        get() = _customFontName
        set(value) {
            _customFontName = value
            me.lampu.lampcord.shared.settings.Settings.shared.customFontName = value
        }

    private var _chatBackground by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatBackground)
    var chatBackground: String
        get() = _chatBackground
        set(value) {
            _chatBackground = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatBackground = value
        }

    private var _chatBubbles by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatBubbles)
    var chatBubbles: Boolean
        get() = _chatBubbles
        set(value) {
            _chatBubbles = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatBubbles = value
        }

    private var _compactMode by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.compactMode)
    var compactMode: Boolean
        get() = _compactMode
        set(value) {
            _compactMode = value
            me.lampu.lampcord.shared.settings.Settings.shared.compactMode = value
            updateUserSettings(UserSettings.Partial(message_display_compact = value))
        }

    private var _showContextMenuMessage by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showContextMenuMessage)
    var showContextMenuMessage: Boolean
        get() = _showContextMenuMessage
        set(value) {
            _showContextMenuMessage = value
            me.lampu.lampcord.shared.settings.Settings.shared.showContextMenuMessage = value
        }

    private var _messageSpacingMode by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.messageSpacingMode)
    var messageSpacingMode: me.lampu.lampcord.shared.settings.MessageSpacingMode
        get() = _messageSpacingMode
        set(value) {
            _messageSpacingMode = value
            me.lampu.lampcord.shared.settings.Settings.shared.messageSpacingMode = value
            // The legacy hardcoded payload reset the rest of the appearance proto
            // (including Discord's theme). Patch only ui_density in a fresh copy.
            val density = when (value) {
                me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> 1
                me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> 4
                me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> 2
            }

            spacingUpdateJob?.cancel()
            spacingUpdateJob = scope.launch {
                val current = userApi.getUserSettingsProto(type = 1) ?: return@launch
                val payload = UserSettingsProtoEditor.withUiDensity(current, density) ?: return@launch
                userApi.updateUserSettingsProto(payload, type = 1)
            }
        }



    private var _notificationsEnabled by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.notificationsEnabled)
    var notificationsEnabled: Boolean
        get() = _notificationsEnabled
        set(value) {
            _notificationsEnabled = value
            me.lampu.lampcord.shared.settings.Settings.shared.notificationsEnabled = value
        }

    private var _showMessagePreview by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showMessagePreview)
    var showMessagePreview: Boolean
        get() = _showMessagePreview
        set(value) {
            _showMessagePreview = value
            me.lampu.lampcord.shared.settings.Settings.shared.showMessagePreview = value
        }

    private var _showInAppNotifications by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showInAppNotifications)
    var showInAppNotifications: Boolean
        get() = _showInAppNotifications
        set(value) {
            _showInAppNotifications = value
            me.lampu.lampcord.shared.settings.Settings.shared.showInAppNotifications = value
        }

    private var _notificationSound by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.notificationSound)
    var notificationSound: Boolean
        get() = _notificationSound
        set(value) {
            _notificationSound = value
            me.lampu.lampcord.shared.settings.Settings.shared.notificationSound = value
        }

    private var _incomingCallSound by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.incomingCallSound)
    var incomingCallSound: Boolean
        get() = _incomingCallSound
        set(value) {
            _incomingCallSound = value
            me.lampu.lampcord.shared.settings.Settings.shared.incomingCallSound = value
        }

    private var _noiseCancellation by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.noiseCancellation)
    var noiseCancellation: Boolean
        get() = _noiseCancellation
        set(value) {
            _noiseCancellation = value
            me.lampu.lampcord.shared.settings.Settings.shared.noiseCancellation = value
        }

    private var _httpProxyUrl by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.httpProxyUrl)
    var httpProxyUrl: String
        get() = _httpProxyUrl
        set(value) {
            _httpProxyUrl = value
            me.lampu.lampcord.shared.settings.Settings.shared.httpProxyUrl = value
        }
    
    private val scope = CoroutineScope(Dispatchers.Main)
    private var pendingUpdateJob: Job? = null
    private var spacingUpdateJob: Job? = null
    private var hasPendingChanges = false
    
    enum class UpdateType {
        INFREQUENT,
        FREQUENT,
        AUTOMATED,
        DAILY
    }

    fun handleUserSettingsUpdate(newSettings: UserSettings) {
        userSettings = userSettings?.copy(
            status = newSettings.status ?: userSettings?.status,
            theme = newSettings.theme ?: userSettings?.theme,
            locale = newSettings.locale ?: userSettings?.locale,
            developer_mode = newSettings.developer_mode ?: userSettings?.developer_mode,
            guild_positions = newSettings.guild_positions.ifEmpty { userSettings?.guild_positions ?: emptyList() },
            guild_folders = newSettings.guild_folders.ifEmpty { userSettings?.guild_folders ?: emptyList() },
            custom_status = newSettings.custom_status ?: userSettings?.custom_status,
            inline_attachment_media = newSettings.inline_attachment_media ?: userSettings?.inline_attachment_media,
            inline_embed_media = newSettings.inline_embed_media ?: userSettings?.inline_embed_media,
            render_embeds = newSettings.render_embeds ?: userSettings?.render_embeds,
            render_reactions = newSettings.render_reactions ?: userSettings?.render_reactions,
            animate_emoji = newSettings.animate_emoji ?: userSettings?.animate_emoji,
            animate_stickers = newSettings.animate_stickers ?: userSettings?.animate_stickers,
            explicit_content_filter = newSettings.explicit_content_filter ?: userSettings?.explicit_content_filter,
            allow_accessibility_detection = newSettings.allow_accessibility_detection ?: userSettings?.allow_accessibility_detection,
            contact_sync_enabled = newSettings.contact_sync_enabled ?: userSettings?.contact_sync_enabled,
            default_guilds_restricted = newSettings.default_guilds_restricted ?: userSettings?.default_guilds_restricted,
            friend_discovery_flags = newSettings.friend_discovery_flags ?: userSettings?.friend_discovery_flags,
            show_current_game = newSettings.show_current_game ?: userSettings?.show_current_game,
            blocked_message_bar = newSettings.blocked_message_bar ?: userSettings?.blocked_message_bar,
            message_display_compact = newSettings.message_display_compact ?: userSettings?.message_display_compact
        ) ?: newSettings

        // Discord remains the source of truth for the theme only while sync is on.
        applyDiscordAppearance(newSettings.theme)

        newSettings.message_display_compact?.let { compact ->
            compactMode = compact
        }
    }


    fun updateUserSetting(update: (UserSettings) -> UserSettings, type: UpdateType = UpdateType.FREQUENT) {
        val current = userSettings ?: return
        userSettings = update(current)
        requestSettingsModify(type)
    }

    private fun requestSettingsModify(type: UpdateType) {
        hasPendingChanges = true
        pendingUpdateJob?.cancel()
        
        val delayMs = when (type) {
            UpdateType.INFREQUENT -> 0L
            UpdateType.FREQUENT -> 10000L
            UpdateType.AUTOMATED -> 30000L
            UpdateType.DAILY -> 86400000L
        }
        
        if (delayMs == 0L) {
            performUpdate()
        } else {
            pendingUpdateJob = scope.launch {
                delay(delayMs.milliseconds)
                performUpdate()
            }
        }
    }

    private val settingsUpdateGuard = SettingsUpdateGuard()

    fun updateUserSettings(partial: UserSettings.Partial) {
        val revision = settingsUpdateGuard.begin()
        val previous = userSettings
        // Optimistically apply for immediate UI feedback (e.g. status changes).
        userSettings = previous?.merge(partial) ?: userSettings
        scope.launch {
            val ok = try {
                userApi.updateUserSettings(partial)
            } catch (e: Exception) {
                false
            }
            // A failed older request must not roll back a newer update.
            if (!ok && settingsUpdateGuard.isLatest(revision)) {
                userSettings = previous
            }
        }
    }

    fun handlePartialUpdate(partial: UserSettings.Partial) {
        userSettings = userSettings?.merge(partial)
    }


    private fun performUpdate() {
        if (!hasPendingChanges) return
        hasPendingChanges = false
        val settings = userSettings ?: return
        
        scope.launch {
            try {
                userApi.updateUserSettings(UserSettings.Partial(
                    theme = settings.theme,
                    developer_mode = settings.developer_mode,
                    render_embeds = settings.render_embeds,
                    render_reactions = settings.render_reactions,
                    inline_embed_media = settings.inline_embed_media,
                    inline_attachment_media = settings.inline_attachment_media,
                    locale = settings.locale,
                    status = settings.status,
                    show_current_game = settings.show_current_game,
                    explicit_content_filter = settings.explicit_content_filter,
                    animate_emoji = settings.animate_emoji,
                    allow_accessibility_detection = settings.allow_accessibility_detection,
                    animate_stickers = settings.animate_stickers,
                    contact_sync_enabled = settings.contact_sync_enabled,
                    friend_discovery_flags = settings.friend_discovery_flags,
                    custom_status = settings.custom_status
                ))

            } catch (e: Exception) {
                println("Failed to update user settings: ${e.message}")
            }
        }
    }

    companion object {
        const val DISCORD_THEME_DARK = "dark"
        const val DISCORD_THEME_LIGHT = "light"
        const val DISCORD_THEME_DARKER = "darker"
        /** Discord's Onyx/Midnight client theme, which is Lampcord's Pure Black look. */
        const val DISCORD_THEME_PURE_BLACK = "midnight"

        /** Pure mapper: no global state, so both gateway paths and tests can use it. */
        internal fun mapDiscordAppearance(theme: String?, syncAppearance: Boolean): DiscordAppearance? {
            if (!syncAppearance) return null
            return when (theme?.trim()?.lowercase()) {
                DISCORD_THEME_DARK -> DiscordAppearance(ThemeMode.DARK, false)
                DISCORD_THEME_LIGHT -> DiscordAppearance(ThemeMode.LIGHT, false)
                DISCORD_THEME_DARKER, DISCORD_THEME_PURE_BLACK -> DiscordAppearance(ThemeMode.DARK, true)
                else -> null
            }
        }

        /** Raw value sent to Discord when appearance sync is enabled. */
        fun discordThemeRawValue(themeMode: ThemeMode, pureBlack: Boolean): String = when {
            pureBlack -> DISCORD_THEME_PURE_BLACK
            themeMode == ThemeMode.LIGHT -> DISCORD_THEME_LIGHT
            else -> DISCORD_THEME_DARK
        }
    }
}

/** Discord theme mapped into Lampcord's local appearance model. */
internal data class DiscordAppearance(val themeMode: ThemeMode, val pureBlack: Boolean)

/**
 * Edits a settings-proto payload in place at the wire level so every unknown field
 * (including required_data_version and the rest of appearance) survives the patch.
 * Returns null when the payload cannot be parsed, in which case no patch is sent.
 */
internal object UserSettingsProtoEditor {
    private const val APPEARANCE_FIELD = 13
    private const val UI_DENSITY_FIELD = 12

    @OptIn(ExperimentalEncodingApi::class)
    fun withUiDensity(base64: String, density: Int): String? = try {
        val bytes = Base64.Default.decode(base64)
        val out = mutableListOf<Byte>()
        val reader = ProtoReader2(bytes)
        var foundAppearance = false

        while (reader.hasRemaining()) {
            val tagStart = reader.position
            val tag = reader.readTag() ?: return null
            reader.position = tagStart
            if (tag.fieldNumber == APPEARANCE_FIELD && tag.wireType == 2) {
                foundAppearance = true
                reader.readTag()
                val appearance = reader.readBytes()
                val updated = replaceDensity(appearance, density) ?: return null
                out += wrapBytesField(APPEARANCE_FIELD, updated).toList()
            } else {
                // position was reset to tagStart above: consume the tag before skipping its payload.
                reader.readTag()
                reader.skip(tag.wireType)
                out += bytes.copyOfRange(tagStart, reader.position).toList()
            }
        }

        if (!foundAppearance) {
            out += wrapBytesField(APPEARANCE_FIELD, replaceDensity(ByteArray(0), density) ?: return null).toList()
        }
        Base64.Default.encode(out.toByteArray())
    } catch (e: Exception) {
        null
    }

    private fun replaceDensity(appearance: ByteArray, density: Int): ByteArray? = try {
        val out = mutableListOf<Byte>()
        val reader = ProtoReader2(appearance)
        var foundDensity = false

        while (reader.hasRemaining()) {
            val tagStart = reader.position
            val tag = reader.readTag() ?: return null
            reader.position = tagStart
            if (tag.fieldNumber == UI_DENSITY_FIELD) {
                if (tag.wireType != 0) return null
                foundDensity = true
                reader.readTag()
                reader.readVarint()
                writeTag(out, UI_DENSITY_FIELD, 0)
                writeVarint(out, density.toLong())
            } else {
                reader.readTag()
                reader.skip(tag.wireType)
                out += appearance.copyOfRange(tagStart, reader.position).toList()
            }
        }

        if (!foundDensity) {
            writeTag(out, UI_DENSITY_FIELD, 0)
            writeVarint(out, density.toLong())
        }
        out.toByteArray()
    } catch (e: Exception) {
        null
    }

    private fun wrapBytesField(field: Int, payload: ByteArray): ByteArray {
        val out = mutableListOf<Byte>()
        writeTag(out, field, 2)
        writeVarint(out, payload.size.toLong())
        out += payload.toList()
        return out.toByteArray()
    }

    private fun writeTag(out: MutableList<Byte>, field: Int, wireType: Int) {
        writeVarint(out, ((field shl 3) or wireType).toLong())
    }

    private fun writeVarint(out: MutableList<Byte>, value: Long) {
        var remaining = value
        while (remaining and 0x7FL.inv() != 0L) {
            out += ((remaining and 0x7F) or 0x80).toByte()
            remaining = remaining ushr 7
        }
        out += remaining.toByte()
    }
}
