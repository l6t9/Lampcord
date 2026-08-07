package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.ExperimentalSettingsImplementation
import com.russhwolf.settings.PreferencesSettings
import com.russhwolf.settings.Settings
import java.util.prefs.Preferences
import java.util.Base64
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec
import java.security.MessageDigest
import com.sun.jna.Platform
import com.sun.jna.platform.win32.Crypt32Util

@OptIn(ExperimentalSettingsImplementation::class)
actual fun createSettings(): Settings {
    val delegate = Preferences.userRoot().node("me.lampu.lampcord")
    val preferencesSettings = PreferencesSettings(delegate)
    return SecureJvmSettings(preferencesSettings)
}

private class SecureJvmSettings(private val delegate: Settings) : Settings by delegate {
    private val secureKeys = setOf("discord_token", "saved_accounts")

    override fun getString(key: String, defaultValue: String): String {
        val value = delegate.getString(key, "")
        if (value.isEmpty()) return defaultValue
        
        return if (key in secureKeys) {
            decrypt(value) ?: defaultValue
        } else {
            value
        }
    }

    override fun putString(key: String, value: String) {
        if (key in secureKeys) {
            delegate.putString(key, encrypt(value))
        } else {
            delegate.putString(key, value)
        }
    }

    private fun encrypt(value: String): String {
        return try {
            if (Platform.isWindows()) {
                val encrypted = Crypt32Util.cryptProtectData(value.toByteArray())
                Base64.getEncoder().encodeToString(encrypted)
            } else {
                // Fallback for macOS/Linux: Machine-bound AES
                encryptAes(value)
            }
        } catch (e: Exception) {
            value
        }
    }

    private fun decrypt(value: String): String? {
        return try {
            if (Platform.isWindows()) {
                val data = Base64.getDecoder().decode(value)
                String(Crypt32Util.cryptUnprotectData(data))
            } else {
                decryptAes(value)
            }
        } catch (e: Exception) {
            null
        }
    }

    private fun getMachineKey(): SecretKeySpec {
        val info = System.getProperty("user.name") + System.getProperty("os.name") + (System.getenv("HOSTNAME") ?: "unknown")
        val hash = MessageDigest.getInstance("SHA-256").digest(info.toByteArray())
        return SecretKeySpec(hash.copyOf(16), "AES")
    }

    private fun encryptAes(value: String): String {
        val cipher = Cipher.getInstance("AES/CBC/PKCS5Padding")
        val key = getMachineKey()
        cipher.init(Cipher.ENCRYPT_MODE, key)
        val iv = cipher.iv
        val encrypted = cipher.doFinal(value.toByteArray())
        return Base64.getEncoder().encodeToString(iv + encrypted)
    }

    private fun decryptAes(value: String): String? {
        val data = Base64.getDecoder().decode(value)
        if (data.size < 16) return null
        val iv = data.copyOfRange(0, 16)
        val encrypted = data.copyOfRange(16, data.size)
        val cipher = Cipher.getInstance("AES/CBC/PKCS5Padding")
        cipher.init(Cipher.DECRYPT_MODE, getMachineKey(), IvParameterSpec(iv))
        return String(cipher.doFinal(encrypted))
    }
}
