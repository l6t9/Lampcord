package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.Settings as KmpSettings
import com.russhwolf.settings.set
import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

class Settings(private val settings: KmpSettings) {

    var discordToken by preference("discord_token", "")
    var savedAccountsJson by preference("saved_accounts", "[]")
    var pureBlack by preferenceBoolean("pure_black", false)
    var themeMode by preference("theme_mode", "auto")
    var showHiddenChannels by preferenceBoolean("show_hidden_channels", false)

    fun getLastChannel(guildId: String): String? {
        val id = settings.getString("last_channel_$guildId", "")
        return if (id.isBlank()) null else id
    }

    fun setLastChannel(guildId: String, channelId: String) {
        settings["last_channel_$guildId"] = channelId
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
    
    companion object {
        val shared: Settings by lazy { Settings(createSettings()) }
    }
}
