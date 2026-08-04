package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.PreferencesSettings
import com.russhwolf.settings.Settings
import java.util.prefs.Preferences

actual fun createSettings(): Settings {
    val delegate = Preferences.userRoot().node("me.lampu.lampcord")
    return PreferencesSettings(delegate)
}
