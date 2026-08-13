package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.Settings as KmpSettings
import com.russhwolf.settings.set
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
    var searchHistoryJson by preference("search_history", "[]")
    var tapTap by preferenceBoolean("tap_tap", true)
    var chatGestures by preferenceEnum("chat_gestures", ChatGestures.SWIPE_TO_MEMBERS)
    var animateStickers by preferenceEnum("animate_stickers", StickerAnimation.ALWAYS)

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
    var messageLoggerEnabled by preferenceBoolean("message_logger_enabled", true)
    var messageLoggerIgnoreBots by preferenceBoolean("message_logger_ignore_bots", false)
    var messageLoggerIgnoreSelf by preferenceBoolean("message_logger_ignore_self", false)

    // Notifications
    var notificationsEnabled by preferenceBoolean("notifications_enabled", true)
    var showMessagePreview by preferenceBoolean("show_message_preview", true)
    var showInAppNotifications by preferenceBoolean("show_in_app_notifications", true)
    var notificationSound by preferenceBoolean("notification_sound", true)
    var autoStartOnBoot by preferenceBoolean("auto_start_on_boot", true)
    var silentBackgroundService by preferenceBoolean("silent_background_service", true)
    var pushRelayServerUrl by preference("push_relay_server_url", "")
    var fcmToken by preference("fcm_token", "")


    // Other Enhancements
    var bypassUploadLimit by preferenceBoolean("bypass_upload_limit", true)
    var messageSpacingMode by preferenceEnum("message_spacing_mode", MessageSpacingMode.DEFAULT)
    var compactMode by preferenceBoolean("compact_mode", false)
    var chatBubbles by preferenceBoolean("chat_bubbles", false)
    var silentTyping by preferenceBoolean("silent_typing", false)
    var hideBlockedMessages by preferenceBoolean("hide_blocked_messages", false)
    var showPermissions by preferenceBoolean("show_permissions", true)

    fun getLastChannel(guildId: String): String? {
        val id = settings.getString("last_channel_$guildId", "")
        return if (id.isBlank()) null else id
    }

    fun setLastChannel(guildId: String, channelId: String) {
        settings["last_channel_$guildId"] = channelId
    }

    private inline fun <reified T : Enum<T>> preferenceEnum(key: String, defaultValue: T): ReadWriteProperty<Any?, T> =
        object : ReadWriteProperty<Any?, T> {
            override fun getValue(thisRef: Any?, property: KProperty<*>): T {
                val name = settings.getString(key, defaultValue.name)
                return enumValues<T>().find { it.name == name } ?: defaultValue
            }

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: T) {
                settings[key] = value.name
            }
        }

    private fun preference(key: String, defaultValue: String): ReadWriteProperty<Any?, String> =
        object : ReadWriteProperty<Any?, String> {
            override fun getValue(thisRef: Any?, property: KProperty<*>): String =
                settings.getString(key, defaultValue)

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: String) {
                settings[key] = value
            }
        }

    private fun preferenceBoolean(key: String, defaultValue: Boolean): ReadWriteProperty<Any?, Boolean> =
        object : ReadWriteProperty<Any?, Boolean> {
            override fun getValue(thisRef: Any?, property: KProperty<*>): Boolean =
                settings.getBoolean(key, defaultValue)

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: Boolean) {
                settings[key] = value
            }
        }

    private fun preferenceFloat(key: String, defaultValue: Float): ReadWriteProperty<Any?, Float> =
        object : ReadWriteProperty<Any?, Float> {
            override fun getValue(thisRef: Any?, property: KProperty<*>): Float =
                settings.getFloat(key, defaultValue)

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: Float) {
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

