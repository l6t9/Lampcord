package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.ExperimentalSettingsImplementation
import com.russhwolf.settings.Settings
import java.util.prefs.Preferences
import java.util.Base64
import java.util.Properties
import java.io.File
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec
import java.security.MessageDigest
import com.sun.jna.Platform
import com.sun.jna.platform.win32.Crypt32Util
import me.lampu.lampcord.shared.utils.XdgUtils

@OptIn(ExperimentalSettingsImplementation::class)
actual fun createSettings(): Settings {
    val configDir = XdgUtils.getXdgConfigHome()
    val appDir = File(configDir, "lampcord").apply { mkdirs() }
    val settingsFile = File(appDir, "settings.properties")
    
    val fileSettings = FileSettings(settingsFile)
    
    if (!fileSettings.getBoolean("migrated_from_prefs", false)) {
        val prefs = Preferences.userRoot().node("me.lampu.lampcord")
        val keys = prefs.keys()
        if (keys.isNotEmpty()) {
            keys.forEach { key ->
                val value = prefs.get(key, null)
                if (value != null) {
                    fileSettings.putString(key, value)
                }
            }
            fileSettings.putBoolean("migrated_from_prefs", true)
        }
    }
    
    return SecureJvmSettings(fileSettings)
}

private class FileSettings(private val file: File) : Settings {
    private val properties = Properties()

    init {
        if (file.exists()) {
            try {
                file.inputStream().use { properties.load(it) }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    private fun save() {
        try {
            file.parentFile?.mkdirs()
            file.outputStream().use { properties.store(it, "Lampcord Settings") }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    override val keys: Set<String> get() = properties.stringPropertyNames()
    override val size: Int get() = properties.size

    override fun clear() {
        properties.clear()
        save()
    }

    override fun hasKey(key: String): Boolean = properties.containsKey(key)

    override fun remove(key: String) {
        properties.remove(key)
        save()
    }

    override fun getBoolean(key: String, defaultValue: Boolean): Boolean =
        properties.getProperty(key)?.toBoolean() ?: defaultValue

    override fun getBooleanOrNull(key: String): Boolean? =
        properties.getProperty(key)?.toBoolean()

    override fun putBoolean(key: String, value: Boolean) {
        properties.setProperty(key, value.toString())
        save()
    }

    override fun getDouble(key: String, defaultValue: Double): Double =
        properties.getProperty(key)?.toDoubleOrNull() ?: defaultValue

    override fun getDoubleOrNull(key: String): Double? =
        properties.getProperty(key)?.toDoubleOrNull()

    override fun putDouble(key: String, value: Double) {
        properties.setProperty(key, value.toString())
        save()
    }

    override fun getFloat(key: String, defaultValue: Float): Float =
        properties.getProperty(key)?.toFloatOrNull() ?: defaultValue

    override fun getFloatOrNull(key: String): Float? =
        properties.getProperty(key)?.toFloatOrNull()

    override fun putFloat(key: String, value: Float) {
        properties.setProperty(key, value.toString())
        save()
    }

    override fun getInt(key: String, defaultValue: Int): Int =
        properties.getProperty(key)?.toIntOrNull() ?: defaultValue

    override fun getIntOrNull(key: String): Int? =
        properties.getProperty(key)?.toIntOrNull()

    override fun putInt(key: String, value: Int) {
        properties.setProperty(key, value.toString())
        save()
    }

    override fun getLong(key: String, defaultValue: Long): Long =
        properties.getProperty(key)?.toLongOrNull() ?: defaultValue

    override fun getLongOrNull(key: String): Long? =
        properties.getProperty(key)?.toLongOrNull()

    override fun putLong(key: String, value: Long) {
        properties.setProperty(key, value.toString())
        save()
    }

    override fun getString(key: String, defaultValue: String): String =
        properties.getProperty(key) ?: defaultValue

    override fun getStringOrNull(key: String): String? =
        properties.getProperty(key)

    override fun putString(key: String, value: String) {
        properties.setProperty(key, value)
        save()
    }
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
        var info = System.getProperty("user.name") + System.getProperty("os.name")
        if (Platform.isLinux()) {
            val machineId = runCatching {
                listOf("/etc/machine-id", "/var/lib/dbus/machine-id")
                    .map { File(it) }
                    .find { it.exists() }
                    ?.readText()?.trim()
            }.getOrNull()

            if (machineId != null) {
                info += machineId
            } else {
                info += System.getenv("HOSTNAME") ?: "unknown"
            }
        } else {
            info += System.getenv("HOSTNAME") ?: (System.getenv("COMPUTERNAME") ?: "unknown")
        }
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
