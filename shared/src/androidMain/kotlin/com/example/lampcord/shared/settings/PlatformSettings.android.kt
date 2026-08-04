package com.example.lampcord.shared.settings

import android.content.Context
import com.russhwolf.settings.Settings
import com.russhwolf.settings.SharedPreferencesSettings

private var appContext: Context? = null

fun initSettings(context: Context) {
    appContext = context.applicationContext
}

actual fun createSettings(): Settings {
    val context = appContext ?: throw IllegalStateException("Settings not initialized with context")
    val delegate = context.getSharedPreferences("lampcord_settings", Context.MODE_PRIVATE)
    return SharedPreferencesSettings(delegate)
}
