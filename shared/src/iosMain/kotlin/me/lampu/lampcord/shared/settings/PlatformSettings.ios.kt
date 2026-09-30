package me.lampu.lampcord.shared.settings

import com.russhwolf.settings.ExperimentalSettingsImplementation
import com.russhwolf.settings.KeychainSettings
import com.russhwolf.settings.NSUserDefaultsSettings
import com.russhwolf.settings.Settings
import platform.Foundation.NSUserDefaults
import kotlin.experimental.ExperimentalObjCName

internal val secureSettingsKeys = setOf(
    "discord_token",
    "saved_accounts"
)

@OptIn(ExperimentalSettingsImplementation::class)
internal class IosSecureSettings(
    private val delegate: Settings
) : Settings by delegate {
    private val secure = KeychainSettings("me.lampu.lampcord.settings")

    init {
        migratePlaintextSecrets()
    }

    override val keys: Set<String>
        get() = delegate.keys + runCatching { secure.keys }.getOrDefault(emptySet())

    override val size: Int
        get() = keys.size

    override fun clear() {
        delegate.clear()
        runCatching { secure.clear() }
    }

    override fun remove(key: String) {
        delegate.remove(key)
        if (key in secureSettingsKeys) runCatching { secure.remove(key) }
    }

    override fun hasKey(key: String): Boolean =
        if (key in secureSettingsKeys) {
            runCatching { secure.hasKey(key) }.getOrDefault(false)
        } else {
            delegate.hasKey(key)
        }

    override fun putString(key: String, value: String) {
        if (key !in secureSettingsKeys) {
            delegate.putString(key, value)
            return
        }
        secure.putString(key, value)
        delegate.remove(key)
    }

    override fun getString(key: String, defaultValue: String): String =
        getStringOrNull(key) ?: defaultValue

    override fun getStringOrNull(key: String): String? =
        if (key in secureSettingsKeys) {
            runCatching { secure.getStringOrNull(key) }.getOrNull()
        } else {
            delegate.getStringOrNull(key)
        }

    private fun migratePlaintextSecrets() {
        secureSettingsKeys.forEach { key ->
            val plaintext = runCatching { delegate.getStringOrNull(key) }.getOrNull() ?: return@forEach
            if (plaintext.isEmpty()) return@forEach
            val alreadySecured = runCatching { secure.getStringOrNull(key) }.getOrNull()
            if (alreadySecured.isNullOrEmpty()) {
                runCatching { secure.putString(key, plaintext) }
            }
            delegate.remove(key)
        }
    }
}

@OptIn(ExperimentalObjCName::class)
@ObjCName("defaultKmpSettings")
internal fun iosDefaultKmpSettings(): Settings =
    IosSecureSettings(NSUserDefaultsSettings(NSUserDefaults.standardUserDefaults))

actual fun createSettings(): Settings = iosDefaultKmpSettings()