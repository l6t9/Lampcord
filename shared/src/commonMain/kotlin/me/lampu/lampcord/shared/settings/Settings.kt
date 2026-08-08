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
