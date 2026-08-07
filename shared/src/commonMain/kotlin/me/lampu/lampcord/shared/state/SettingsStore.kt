package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.UserSettings
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class SettingsStore(
    private val discordClient: DiscordClient
) {
    var userSettings by mutableStateOf<UserSettings?>(null)
    
    private var _pureBlack by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.pureBlack)
    var pureBlack: Boolean
        get() = _pureBlack
        set(value) {
            _pureBlack = value
            me.lampu.lampcord.shared.settings.Settings.shared.pureBlack = value
        }

    private var _themeMode by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.themeMode)
    var themeMode: String
        get() = _themeMode
        set(value) {
            _themeMode = value
            me.lampu.lampcord.shared.settings.Settings.shared.themeMode = value
        }

    private var _showHiddenChannels by mutableStateOf(me.lampu.lampcord.shared.settings.Settings.shared.showHiddenChannels)
    var showHiddenChannels: Boolean
        get() = _showHiddenChannels
        set(value) {
            _showHiddenChannels = value
            me.lampu.lampcord.shared.settings.Settings.shared.showHiddenChannels = value
        }
    
    private val scope = CoroutineScope(Dispatchers.Main)
    private var pendingUpdateJob: Job? = null
    private var hasPendingChanges = false
    
    enum class UpdateType {
        INFREQUENT,
        FREQUENT,
        AUTOMATED,
        DAILY
    }

    fun handleUserSettingsUpdate(newSettings: UserSettings) {
        userSettings = userSettings?.copy(
            status = newSettings.status ?: userSettings?.status,
            theme = newSettings.theme ?: userSettings?.theme,
            locale = newSettings.locale ?: userSettings?.locale,
            developer_mode = newSettings.developer_mode ?: userSettings?.developer_mode,
            guild_positions = if (newSettings.guild_positions.isNotEmpty()) newSettings.guild_positions else userSettings?.guild_positions ?: emptyList(),
            guild_folders = if (newSettings.guild_folders.isNotEmpty()) newSettings.guild_folders else userSettings?.guild_folders ?: emptyList(),
            custom_status = newSettings.custom_status ?: userSettings?.custom_status
        ) ?: newSettings
    }

    fun updateUserSetting(update: (UserSettings) -> UserSettings, type: UpdateType = UpdateType.FREQUENT) {
        val current = userSettings ?: return
        userSettings = update(current)
        requestSettingsModify(type)
    }

    private fun requestSettingsModify(type: UpdateType) {
        hasPendingChanges = true
        pendingUpdateJob?.cancel()
        
        val delayMs = when (type) {
            UpdateType.INFREQUENT -> 0L
            UpdateType.FREQUENT -> 10000L
            UpdateType.AUTOMATED -> 30000L
            UpdateType.DAILY -> 86400000L
        }
        
        if (delayMs == 0L) {
            performUpdate()
        } else {
            pendingUpdateJob = scope.launch {
                delay(delayMs)
                performUpdate()
            }
        }
    }

    private fun performUpdate() {
        if (!hasPendingChanges) return
        hasPendingChanges = false
        val settings = userSettings ?: return
        
        scope.launch {
            try {
                discordClient.updateUserSettings(settings)
            } catch (e: Exception) {
                println("Failed to update user settings: ${e.message}")
            }
        }
    }
}
