@file:Suppress("DEPRECATION")

package me.lampu.lampcord.shared.settings

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.russhwolf.settings.Settings
import com.russhwolf.settings.SharedPreferencesSettings
import me.lampu.lampcord.shared.utils.Logging

private var appContext: Context? = null

fun initSettings(context: Context) {
    appContext = context.applicationContext
}

actual fun createSettings(): Settings {
    val context = appContext ?: throw IllegalStateException("Settings not initialized with context")
    val name = "lampcord_encrypted_settings"

    fun open(masterKey: MasterKey) = EncryptedSharedPreferences.create(
        context,
        name,
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    val delegate = try {
        open(masterKey)
    } catch (e: Exception) {
        Logging.e("Settings", "Failed to open encrypted settings, recreating store: ${e.message}")
        runCatching {
            context.deleteSharedPreferences(name)
            val prefsDir = java.io.File(context.applicationInfo.dataDir, "shared_prefs")
            java.io.File(prefsDir, "$name.xml").delete()
            java.io.File(prefsDir, "$name.xml.bak").delete()
        }
        open(masterKey)
    }

    return SharedPreferencesSettings(delegate)
}
