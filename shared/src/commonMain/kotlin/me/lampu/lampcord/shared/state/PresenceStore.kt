package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.model.ReadyPayload
import me.lampu.lampcord.shared.model.Session

class PresenceStore(private val userApi: UserApi) {
    private val _presences = MutableStateFlow<Map<String, Map<String, PresenceUpdate>>>(emptyMap())
    
    // A simple cache for flattened presences to avoid recomputing too much
    private val flattenedPresences = MutableStateFlow<Map<String, PresenceUpdate>>(emptyMap())
    val presences: StateFlow<Map<String, PresenceUpdate>> = flattenedPresences.asStateFlow()
    val allPresences: StateFlow<Map<String, PresenceUpdate>> = flattenedPresences.asStateFlow()

    private fun updateFlattened() {
        val current = _presences.value
        val flattened = current.mapValues { (_, guildMap) ->
            // Priority: online > idle > dnd > offline
            val presences = guildMap.values
            presences.find { it.status == "online" }
                ?: presences.find { it.status == "idle" }
                ?: presences.find { it.status == "dnd" }
                ?: presences.firstOrNull()
                ?: PresenceUpdate(status = "offline")
        }
        flattenedPresences.value = flattened
    }

    fun handleReady(ready: ReadyPayload) {
        val newPresences = mutableMapOf<String, MutableMap<String, PresenceUpdate>>()
        
        ready.merged_presences?.friends?.forEach { update ->
            val userId = update.user?.id ?: update.user_id ?: return@forEach
            newPresences.getOrPut(userId) { mutableMapOf() }["global"] = update
        }
        
        ready.merged_presences?.guilds?.forEachIndexed { index, guildPresences ->
            val guild = ready.guilds.getOrNull(index) ?: return@forEachIndexed
            guildPresences.forEach { update ->
                val userId = update.user?.id ?: update.user_id ?: return@forEach
                newPresences.getOrPut(userId) { mutableMapOf() }[guild.id] = update
            }
        }
        
        ready.merged_members?.forEachIndexed { index, members ->
            val guild = ready.guilds.getOrNull(index) ?: return@forEachIndexed
            members.forEach { member ->
                val userId = member.user?.id ?: member.userId() ?: return@forEach
                member.presence?.let { p ->
                    val pWithId = if (p.user?.id == null && p.user_id == null) {
                        p.copy(user_id = userId)
                    } else p
                    newPresences.getOrPut(userId) { mutableMapOf() }[guild.id] = pWithId
                }
            }
        }
        
        _presences.value = newPresences
        updateFlattened()
    }

    fun handlePresenceUpdate(update: PresenceUpdate) {
        val userId = update.user?.id ?: update.user_id ?: return
        val guildId = update.guild_id ?: "global"
        _presences.update { current ->
            val userMap = current[userId]?.toMutableMap() ?: mutableMapOf()
            if (update.status == "offline") {
                userMap.remove(guildId)
            } else {
                userMap[guildId] = update
            }
            if (userMap.isEmpty()) current - userId else current + (userId to userMap)
        }
        updateFlattened()
    }

    fun handleSessions(userId: String, sessions: List<Session>) {
        val activeSession = sessions.find { it.active } ?: sessions.firstOrNull() ?: return
        val existing = _presences.value[userId]?.get("global")
        val newPresence = (existing ?: PresenceUpdate(user_id = userId)).copy(
            status = activeSession.status ?: existing?.status ?: "online",
            activities = activeSession.activities
        )
        _presences.update { current ->
            val userMap = current[userId]?.toMutableMap() ?: mutableMapOf()
            userMap["global"] = newPresence
            current + (userId to userMap)
        }
        updateFlattened()
    }

    fun clear() {
        _presences.value = emptyMap()
        flattenedPresences.value = emptyMap()
    }

    fun getUserStatus(userId: String, currentUserId: String?, currentUserStatus: String?): String {
        if (userId == currentUserId) return currentUserStatus ?: "online"
        val p = flattenedPresences.value[userId] ?: return "offline"
        return presenceStatus(p)
    }

    fun getUserStatus(userId: String, presence: PresenceUpdate?, currentUserId: String?, currentUserStatus: String?): String {
        if (userId == currentUserId) return currentUserStatus ?: "online"
        val p = presence ?: flattenedPresences.value[userId] ?: return "offline"
        return presenceStatus(p)
    }

    private fun presenceStatus(p: PresenceUpdate): String {
        if (p.activities.any { it.type == 1 }) return "streaming"

        if (p.status == "online") {
            val cs = p.client_status
            if (cs.mobile == "online" && cs.desktop != "online" && cs.web != "online") {
                return "mobile"
            }
        }

        return p.status
    }

    suspend fun updateStatus(status: String): Boolean {
        return userApi.updateStatus(status)
    }

    suspend fun updateCustomStatus(text: String?): Boolean {
        return userApi.updateCustomStatus(text)
    }

    fun isStatusVisible(user: me.lampu.lampcord.shared.model.User, presence: PresenceUpdate?): Boolean {
        val flags = (user.public_flags ?: 0) or (user.flags ?: 0)
        val status = presence?.status ?: "offline"
        val isStreaming = presence?.activities?.any { it.type == 1 } == true
        val isOnline = status != "offline" && status != "invisible"
        
        return if ((flags and 524288) != 0) { // FLAG_OFFLINE_VISIBLE
            isOnline
        } else {
            isOnline || isStreaming
        }
    }
}
