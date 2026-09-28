@file:Suppress("DEPRECATION")

package me.lampu.lampcord.shared.settings

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.russhwolf.settings.Settings
import com.russhwolf.settings.SharedPreferencesSettings

private var appContext: Context? = null

fun initSettings(context: Context) {
    appContext = context.applicationContext
}

actual fun createSettings(): Settings {
    val context = appContext ?: throw IllegalStateException("Settings not initialized with context")

    val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    val delegate = EncryptedSharedPreferences.create(
        context,
        "lampcord_encrypted_settings",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    return SharedPreferencesSettings(delegate)
}
