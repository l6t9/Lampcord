package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.PresenceUpdate

class PresenceStore(private val discordClient: DiscordClient) {
    private val _presences = MutableStateFlow<Map<String, PresenceUpdate>>(emptyMap())
    val presences: StateFlow<Map<String, PresenceUpdate>> = _presences.asStateFlow()

    fun handlePresenceUpdate(update: PresenceUpdate) {
        val userId = update.user?.id ?: update.user_id ?: return
        _presences.update { it + (userId to update) }
    }

    fun clear() {
        _presences.value = emptyMap()
    }

    fun getUserStatus(userId: String, currentUserId: String?, currentUserStatus: String?): String {
        if (userId == currentUserId) return currentUserStatus ?: "online"
        return _presences.value[userId]?.status ?: "offline"
    }

    suspend fun updateStatus(status: String): Boolean {
        return discordClient.updateStatus(status)
    }

    suspend fun updateCustomStatus(text: String?): Boolean {
        return discordClient.updateCustomStatus(text)
    }

    fun isStatusVisible(user: me.lampu.lampcord.shared.model.User, presence: PresenceUpdate?, isStreaming: Boolean): Boolean {
        val flags = (user.public_flags ?: 0) or (user.flags ?: 0)
        return if ((flags and 524288) != 0) {
            presence != null && presence.status != "offline" && presence.status != "invisible"
        } else {
            presence != null || isStreaming
        }
    }
}
