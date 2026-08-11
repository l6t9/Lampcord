package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.model.UserGuildSettings
import me.lampu.lampcord.shared.model.ReadyPayload
import kotlin.time.Clock
import kotlin.time.Instant


class UserGuildSettingsStore {
    private val _userGuildSettings = MutableStateFlow<Map<String?, UserGuildSettings>>(emptyMap())
    val userGuildSettings: StateFlow<Map<String?, UserGuildSettings>> = _userGuildSettings.asStateFlow()

    fun handleReady(ready: ReadyPayload) {
        val map = if (ready.user_guild_settings?.partial == false) mutableMapOf() else _userGuildSettings.value.toMutableMap()
        ready.user_guild_settings?.entries?.forEach { settings ->
            map[settings.guild_id] = settings
        }
        _userGuildSettings.value = map
    }

    fun handlePartialUpdate(partial: UserGuildSettings.Partial) {
        val guildId = partial.guild_id ?: return
        _userGuildSettings.update { current ->
            val existing = current[guildId] ?: UserGuildSettings(guild_id = guildId)
            current + (guildId to existing.applyPartial(partial))
        }
    }

    private fun UserGuildSettings.applyPartial(partial: UserGuildSettings.Partial): UserGuildSettings {
        return this.copy(
            muted = partial.muted ?: this.muted,
            hide_muted_channels = partial.hide_muted_channels ?: this.hide_muted_channels,
            suppress_everyone = partial.suppress_everyone ?: this.suppress_everyone,
            suppress_roles = partial.suppress_roles ?: this.suppress_roles,
            message_notifications = partial.message_notifications ?: this.message_notifications,
            mobile_push = partial.mobile_push ?: this.mobile_push,
            channel_overrides = partial.channel_overrides ?: this.channel_overrides,
            flags = partial.flags ?: this.flags,
            mute_config = partial.mute_config ?: this.mute_config,
            notify_highlights = partial.notify_highlights ?: this.notify_highlights
        )
    }

    fun isChannelMuted(guildId: String?, channelId: String): Boolean {
        val guildSettings = _userGuildSettings.value[guildId] ?: return false
        val channelOverride = guildSettings.channel_overrides.find { it.channel_id == channelId }
        if (channelOverride == null) return false
        
        return channelOverride.muted && (channelOverride.mute_config?.end_time?.let {
            try { Instant.parse(it) > Clock.System.now() } catch(_: Exception) { false }
        } ?: true)
    }
    
    fun isGuildMuted(guildId: String?): Boolean {
        val settings = _userGuildSettings.value[guildId] ?: return false
        return settings.muted && (settings.mute_config?.end_time?.let {
            try { Instant.parse(it) > Clock.System.now() } catch(_: Exception) { false }
        } ?: true)
    }

    fun clear() {
        _userGuildSettings.value = emptyMap()
    }
}
