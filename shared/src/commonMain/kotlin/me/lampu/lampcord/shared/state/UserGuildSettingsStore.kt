package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import me.lampu.lampcord.shared.model.UserGuildSettings
import me.lampu.lampcord.shared.model.ReadyPayload

class UserGuildSettingsStore {
    val userGuildSettings = mutableStateMapOf<String?, UserGuildSettings>()

    fun handleReady(ready: ReadyPayload) {
        ready.user_guild_settings?.forEach { settings ->
            userGuildSettings[settings.guild_id] = settings
        }
    }

    fun handleUpdate(settings: UserGuildSettings) {
        userGuildSettings[settings.guild_id] = settings
    }

    fun isChannelMuted(guildId: String?, channelId: String): Boolean {
        val guildSettings = userGuildSettings[guildId] ?: return false
        val channelOverride = guildSettings.channel_overrides.find { it.channel_id == channelId }
        return channelOverride?.muted ?: false
    }
    
    fun isGuildMuted(guildId: String?): Boolean {
        return userGuildSettings[guildId]?.muted ?: false
    }

    fun clear() {
        userGuildSettings.clear()
    }
}
