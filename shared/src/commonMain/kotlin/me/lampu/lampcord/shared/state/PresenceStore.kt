package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.PresenceUpdate

class PresenceStore(private val discordClient: DiscordClient) {
    val presences = mutableStateMapOf<String, PresenceUpdate>()

    fun handlePresenceUpdate(update: PresenceUpdate) {
        presences[update.user.id] = update
    }

    fun getUserStatus(userId: String, currentUserId: String?, currentUserStatus: String?): String {
        if (userId == currentUserId) return currentUserStatus ?: "online"
        return presences[userId]?.status ?: "offline"
    }

    suspend fun updateStatus(status: String): Boolean {
        return discordClient.updateStatus(status)
    }

    suspend fun updateCustomStatus(text: String?): Boolean {
        return discordClient.updateCustomStatus(text)
    }
}
