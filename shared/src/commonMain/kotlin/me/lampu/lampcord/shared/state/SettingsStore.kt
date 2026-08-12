package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.UserSettings
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

class SettingsStore(
    private val discordClient: DiscordClient
) {
    var userSettings by mutableStateOf<UserSettings?>(null)
    
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

    private var _chatBackground by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.chatBackground)
    var chatBackground: String
        get() = _chatBackground
        set(value) {
            _chatBackground = value
            me.lampu.lampcord.shared.settings.Settings.shared.chatBackground = value
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
    
    private val scope = CoroutineScope(Dispatchers.Main)
    private var pendingUpdateJob: Job? = null
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
            animate_emoji = newSettings.animate_emoji ?: userSettings?.animate_emoji,
            animate_stickers = newSettings.animate_stickers ?: userSettings?.animate_stickers,
            explicit_content_filter = newSettings.explicit_content_filter ?: userSettings?.explicit_content_filter,
            allow_accessibility_detection = newSettings.allow_accessibility_detection ?: userSettings?.allow_accessibility_detection,
            contact_sync_enabled = newSettings.contact_sync_enabled ?: userSettings?.contact_sync_enabled,
            default_guilds_restricted = newSettings.default_guilds_restricted ?: userSettings?.default_guilds_restricted,
            friend_discovery_flags = newSettings.friend_discovery_flags ?: userSettings?.friend_discovery_flags,
            show_current_game = newSettings.show_current_game ?: userSettings?.show_current_game,
            blocked_message_bar = newSettings.blocked_message_bar ?: userSettings?.blocked_message_bar
        ) ?: newSettings
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

    fun updateUserSettings(partial: UserSettings.Partial) {
        scope.launch {
            if (discordClient.updateUserSettings(partial)) {
                userSettings = userSettings?.merge(partial)
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
                discordClient.updateUserSettings(UserSettings.Partial(
                    theme = settings.theme,
                    developer_mode = settings.developer_mode,
                    render_embeds = settings.render_embeds,
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
}
