package com.example.materialcord.shared.settings

import com.russhwolf.settings.PreferencesSettings
import com.russhwolf.settings.Settings
import java.util.prefs.Preferences

actual fun createSettings(): Settings {
    val delegate = Preferences.userRoot().node("com.example.materialcord")
    return PreferencesSettings(delegate)
}
