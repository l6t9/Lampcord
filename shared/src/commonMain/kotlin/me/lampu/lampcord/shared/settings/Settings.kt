package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.Settings as KmpSettings
import com.russhwolf.settings.set
import androidx.compose.runtime.mutableStateOf
import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

class Settings(private val settings: KmpSettings) {

    var discordToken by preference("discord_token", "")
    var savedAccountsJson by preference("saved_accounts", "[]")
    var pureBlack by preferenceBoolean("pure_black", false)
    var themeMode by preferenceEnum("theme_mode", ThemeMode.AUTO)
    var themePaletteStyle by preferenceEnum("theme_palette_style", ThemePaletteStyle.TONAL_SPOT)
    var appFont by preferenceEnum("app_font", FontOption.SYSTEM)
    var accentColor by preference("accent_color", "#6750A4")
    var materialYou by preferenceBoolean("material_you", true)
    var showHiddenChannels by preferenceBoolean("show_hidden_channels", false)
    var syncAppearance by preferenceBoolean("sync_appearance", true)
    var fontScale by preferenceFloat("font_scale", 1.0f)
    var customFontPath by preference("custom_font_path", "")
    var chatBackground by preference("chat_background", "")
    var activeThemeJson by preference("active_theme_json", "")
    var installedThemesJson by preference("installed_themes_json", "[]")
    var searchHistoryJson by preference("search_history", "[]")
    var tapTap by preferenceEnum("taptap_action", TapTapAction.REPLY_OR_EDIT)
    var tapTapEmoji by preference("taptap_emoji", "")

    var chatGestures by preferenceEnum("chat_gestures", ChatGestures.SWIPE_TO_MEMBERS)
    var animateStickers by preferenceEnum("animate_stickers", StickerAnimation.ALWAYS)
    var panelAnimation by preferenceEnum("panel_animation", PanelAnimation.MINIMAL)
    var reduceMotion by preferenceBoolean("reduce_motion", false)
    var desktopLowMemoryMode by preferenceBoolean("desktop_low_memory_mode", false)
    var verboseLogging by preferenceBoolean("verbose_logging", false)
    var enableSystemWindowFrame by preferenceBoolean("enable_system_window_frame", false)
    var waylandDefaultFrameApplied by preferenceBoolean("wayland_default_frame_applied", false)
    var macDefaultFrameApplied by preferenceBoolean("mac_default_frame_applied", false)
    var disableWaylandScaling by preferenceBoolean("disable_wayland_scaling", false)

    // Theme Settings
    var transparencyMode by preferenceEnum("transparency_mode", TransparencyMode.NONE)
    var enableCustomFonts by preferenceBoolean("enable_custom_fonts", true)
    var enableCustomSounds by preferenceBoolean("enable_custom_sounds", true)

    // Chatbox Customization
    var chatboxBackgroundOpacity by preferenceFloat("chatbox_background_opacity", 1.0f)
    var chatboxBorderRadius by preferenceInt("chatbox_border_radius", 16)
    var chatboxHeight by preferenceInt("chatbox_height", 40)
    var chatboxHideUploadButton by preferenceBoolean("chatbox_hide_upload_button", false)
    var chatboxHideEmojiButton by preferenceBoolean("chatbox_hide_emoji_button", false)
    var chatboxHideVoiceButton by preferenceBoolean("chatbox_hide_voice_button", true)
    var chatboxFontSize by preferenceFloat("chatbox_font_size", 1.0f)
    var chatboxShowAvatar by preferenceBoolean("chatbox_show_avatar", false)

    // Free Nitro Emojis
    var freeNitroEmojis by preferenceBoolean("free_nitro_emojis", true)
    var realmojis by preferenceBoolean("realmojis", true)
    var compoundRealmojis by preferenceBoolean("compound_realmojis", true)
    var useWebpEmojis by preferenceBoolean("use_webp_emojis", true)

    // Clean Channels
    var cleanChannelsRemoveEmojis by preferenceBoolean("clean_channels_remove_emojis", true)
    var cleanChannelsHideSymbols by preferenceBoolean("clean_channels_hide_symbols", true)
    var cleanChannelsNormalizeLetters by preferenceBoolean("clean_channels_normalize_letters", true)
    var cleanChannelsCapitalizeCategories by preferenceBoolean("clean_channels_capitalize_categories", true)

    // Message Logger
    var messageLoggerEnabled by preferenceBoolean("message_logger_enabled", false)
    var messageLoggerIgnoreBots by preferenceBoolean("message_logger_ignore_bots", false)
    var messageLoggerIgnoreSelf by preferenceBoolean("message_logger_ignore_self", false)

    // Notifications
    var notificationsEnabled by preferenceBoolean("notifications_enabled", true)
    var showMessagePreview by preferenceBoolean("show_message_preview", true)
    var showInAppNotifications by preferenceBoolean("show_in_app_notifications", true)
    var notificationSound by preferenceBoolean("notification_sound", true)
    var autoStartOnBoot by preferenceBoolean("auto_start_on_boot", true)
    var silentBackgroundService by preferenceBoolean("silent_background_service", true)

    // Other Enhancements
    var bypassUploadLimit by preferenceBoolean("bypass_upload_limit", true)
    var showContextMenuMessage by preferenceBoolean("show_context_menu_message", false)
    var messageSpacingMode by preferenceEnum("message_spacing_mode", MessageSpacingMode.DEFAULT)
    var compactMode by preferenceBoolean("compact_mode", false)
    var chatBubbles by preferenceBoolean("chat_bubbles", false)
    var silentTyping by preferenceBoolean("silent_typing", false)
    var silentTypingButtonEnabled by preferenceBoolean("silent_typing_button_enabled", false)
    var musicPresenceEnabled by preferenceBoolean("music_presence_enabled", true)
    var hideBlockedMessages by preferenceBoolean("hide_blocked_messages", false)
    var showPermissions by preferenceBoolean("show_permissions", true)
    var emojiUsageJson by preference("emoji_usage_v4", "{}")
    var stickerUsageJson by preference("sticker_usage_v1", "{}")
    var favoriteEmojisJson by preference("favorite_emojis_v1", "[]")
    var videoVolume by preferenceFloat("video_volume", 0.2f)
    var localProfileOverrides by preference("local_profile_overrides_v1", "{}")
    var profile3y3 by preferenceBoolean("profile_3y3", true)
    var userBg by preferenceBoolean("user_bg", true)
    var userPfp by preferenceBoolean("user_pfp", true)
    var hideNavLabels by preferenceBoolean("hide_nav_labels", false)
    var showChatSearch by preferenceBoolean("show_chat_search", false)
    var showChatPins by preferenceBoolean("show_chat_pins", false)
    var showCallButton by preferenceBoolean("show_call_button", true)
    var showNavHome by preferenceBoolean("show_nav_home", true)
    var showNavFriends by preferenceBoolean("show_nav_friends", true)
    var showNavSearch by preferenceBoolean("show_nav_search", true)
    var showNavMentions by preferenceBoolean("show_nav_mentions", true)
    var showNavSettings by preferenceBoolean("show_nav_settings", true)
    var navTabsOrderJson by preference("nav_tabs_order", "[\"home\",\"friends\",\"search\",\"mentions\",\"settings\"]")
    var secretTabEnabled by preferenceBoolean("secret_tab_enabled", false)
    var textReplaceJson by preference("text_replace_v1", "[]")

    // Voice Settings
    var noiseCancellation by preferenceBoolean("noise_cancellation", true)

    fun getLastChannel(guildId: String): String? {
        val id = settings.getString("last_channel_$guildId", "")
        return if (id.isBlank()) null else id
    }

    fun setLastChannel(guildId: String, channelId: String) {
        settings["last_channel_$guildId"] = channelId
    }

    fun getLastGuild(): String? {
        val id = settings.getString("last_guild", "")
        return if (id.isBlank()) null else id
    }

    fun setLastGuild(guildId: String) {
        settings["last_guild"] = guildId
    }

    fun clearLastGuild() {
        settings.remove("last_guild")
    }

    private inline fun <reified T : Enum<T>> preferenceEnum(key: String, defaultValue: T): ReadWriteProperty<Any?, T> =
        object : ReadWriteProperty<Any?, T> {
            private val value = mutableStateOf(
                settings.getString(key, defaultValue.name)
                    .let { name -> enumValues<T>().find { it.name == name } ?: defaultValue }
            )

            override fun getValue(thisRef: Any?, property: KProperty<*>): T = value.value

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: T) {
                this.value.value = value
                settings[key] = value.name
            }
        }

    private fun preference(key: String, defaultValue: String): ReadWriteProperty<Any?, String> =
        object : ReadWriteProperty<Any?, String> {
            private val value = mutableStateOf(settings.getString(key, defaultValue))

            override fun getValue(thisRef: Any?, property: KProperty<*>): String = value.value

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: String) {
                this.value.value = value
                settings[key] = value
            }
        }

    private fun preferenceBoolean(key: String, defaultValue: Boolean): ReadWriteProperty<Any?, Boolean> =
        object : ReadWriteProperty<Any?, Boolean> {
            private val state = mutableStateOf(settings.getBoolean(key, defaultValue))

            override fun getValue(thisRef: Any?, property: KProperty<*>): Boolean = state.value

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: Boolean) {
                state.value = value
                settings[key] = value
            }
        }

    private fun preferenceFloat(key: String, defaultValue: Float): ReadWriteProperty<Any?, Float> =
        object : ReadWriteProperty<Any?, Float> {
            private val state = mutableStateOf(settings.getFloat(key, defaultValue))

            override fun getValue(thisRef: Any?, property: KProperty<*>): Float = state.value

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: Float) {
                state.value = value
                settings[key] = value
            }
        }

    private fun preferenceInt(key: String, defaultValue: Int): ReadWriteProperty<Any?, Int> =
        object : ReadWriteProperty<Any?, Int> {
            private val state = mutableStateOf(settings.getInt(key, defaultValue))

            override fun getValue(thisRef: Any?, property: KProperty<*>): Int = state.value

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: Int) {
                state.value = value
                settings[key] = value
            }
        }
    
    companion object {
        val shared: Settings by lazy { Settings(createSettings()) }
    }
}

enum class MessageSpacingMode {
    COMPACT,
    DEFAULT,
    SPACIOUS
}

