package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.model.ReadyPayload
import me.lampu.lampcord.shared.model.Session

class PresenceStore(private val discordClient: DiscordClient) {
    private val _presences = MutableStateFlow<Map<String, PresenceUpdate>>(emptyMap())
    val presences: StateFlow<Map<String, PresenceUpdate>> = _presences.asStateFlow()

    fun handleReady(ready: ReadyPayload) {
        val newPresences = mutableMapOf<String, PresenceUpdate>()
        ready.merged_presences?.friends?.forEach { update ->
            val userId = update.user?.id ?: update.user_id ?: return@forEach
            newPresences[userId] = update
        }
        ready.merged_presences?.guilds?.forEach { guildPresences ->
            guildPresences.forEach { update ->
                val userId = update.user?.id ?: update.user_id ?: return@forEach
                newPresences[userId] = update
            }
        }
        ready.merged_members?.forEach { members ->
            members.forEach { member ->
                val userId = member.user?.id ?: member.userId() ?: return@forEach
                member.presence?.let { p ->
                    val pWithId = if (p.user?.id == null && p.user_id == null) {
                        p.copy(user_id = userId)
                    } else p
                    newPresences[userId] = pWithId
                }
            }
        }
        _presences.value = newPresences
    }

    fun handlePresenceUpdate(update: PresenceUpdate) {
        val userId = update.user?.id ?: update.user_id ?: return
        _presences.update { it + (userId to update) }
    }

    fun handleSessions(userId: String, sessions: List<Session>) {
        val activeSession = sessions.find { it.active } ?: sessions.firstOrNull() ?: return
        val existing = _presences.value[userId]
        val newPresence = (existing ?: PresenceUpdate(user_id = userId)).copy(
            status = activeSession.status ?: existing?.status ?: "online",
            activities = activeSession.activities
        )
        _presences.update { it + (userId to newPresence) }
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
