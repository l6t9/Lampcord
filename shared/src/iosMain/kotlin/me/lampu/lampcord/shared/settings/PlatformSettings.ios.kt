package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.ExperimentalSettingsImplementation
import com.russhwolf.settings.KeychainSettings
import com.russhwolf.settings.Settings

@OptIn(ExperimentalSettingsImplementation::class)
actual fun createSettings(): Settings {
    return KeychainSettings("me.lampu.lampcord")
}
