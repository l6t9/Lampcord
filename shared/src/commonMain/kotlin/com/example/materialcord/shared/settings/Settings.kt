package com.example.materialcord.shared.settings

import com.russhwolf.settings.Settings as KmpSettings
import com.russhwolf.settings.set
import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

class Settings(private val settings: KmpSettings) {

    var discordToken by preference("discord_token", "")

    private fun preference(key: String, defaultValue: String): ReadWriteProperty<Any?, String> =
        object : ReadWriteProperty<Any?, String> {
            override fun getValue(thisRef: Any?, property: KProperty<*>): String =
                settings.getString(key, defaultValue)

            override fun setValue(thisRef: Any?, property: KProperty<*>, value: String) {
                settings[key] = value
            }
        }
    
    companion object {
        val shared: Settings by lazy { Settings(createSettings()) }
    }
}
