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

    private fun flattenOne(presences: Collection<PresenceUpdate>): PresenceUpdate =
        presences.find { it.status == "online" }
            ?: presences.find { it.status == "idle" }
            ?: presences.find { it.status == "dnd" }
            ?: presences.firstOrNull()
            ?: PresenceUpdate(status = "offline")

    private fun rebuildFlattened() {
        flattenedPresences.value = _presences.value.mapValues { (_, guildMap) -> flattenOne(guildMap.values) }
    }

    private fun updateFlattened(changed: Set<String>) {
        if (changed.isEmpty()) return
        val source = _presences.value
        flattenedPresences.update { current ->
            val next = current.toMutableMap()
            for (userId in changed) {
                val guildMap = source[userId]
                if (guildMap == null) next.remove(userId) else next[userId] = flattenOne(guildMap.values)
            }
            next
        }
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
        rebuildFlattened()

        ready.sessions?.let { sessions ->
            val userId = ready.user?.id ?: return@let
            handleSessions(userId, sessions)
        }
    }

    fun handlePresenceUpdate(update: PresenceUpdate) {
        applyPresences(listOf(update))
    }

    fun applyPresences(updates: List<PresenceUpdate>) {
        if (updates.isEmpty()) return
        val changed = HashSet<String>(updates.size * 2)
        _presences.update { current ->
            val next = current.toMutableMap()
            for (update in updates) {
                val userId = update.user?.id ?: update.user_id ?: continue
                val guildId = update.guild_id ?: "global"
                val userMap = next[userId]?.toMutableMap() ?: HashMap(4)
                if (update.status == "offline") userMap.remove(guildId) else userMap[guildId] = update
                if (userMap.isEmpty()) next.remove(userId) else next[userId] = userMap
                changed.add(userId)
            }
            next
        }
        updateFlattened(changed)
    }

    fun handleSessions(userId: String, sessions: List<Session>) {
        val activeSession = sessions.find { it.active } ?: sessions.firstOrNull() ?: return
        val existing = _presences.value[userId]?.get("global")
        
        val clientStatus = me.lampu.lampcord.shared.model.ClientStatus(
            desktop = sessions.find { it.client_info?.client == "desktop" }?.status,
            mobile = sessions.find { it.client_info?.client == "mobile" }?.status,
            web = sessions.find { it.client_info?.client == "web" }?.status
        )

        val newPresence = (existing ?: PresenceUpdate(user_id = userId)).copy(
            status = activeSession.status ?: existing?.status ?: "online",
            activities = activeSession.activities,
            client_status = clientStatus
        )
        _presences.update { current ->
            val userMap = current[userId]?.toMutableMap() ?: mutableMapOf()
            userMap["global"] = newPresence
            current + (userId to userMap)
        }
        updateFlattened(setOf(userId))
    }

    fun clear() {
        _presences.value = emptyMap()
        flattenedPresences.value = emptyMap()
    }

    fun getUserStatus(userId: String, currentUserId: String?, currentUserStatus: String?): String {
        return getUserStatus(userId, null, currentUserId, currentUserStatus)
    }

    fun getUserStatus(userId: String, presence: PresenceUpdate?, currentUserId: String?, currentUserStatus: String?): String {
        val p = presence ?: flattenedPresences.value[userId]
        
        if (p != null) {
            val status = presenceStatus(p)
            if (userId == currentUserId && (status == "streaming" || status == "mobile")) {
                return status
            }
        }
        
        if (userId == currentUserId) return currentUserStatus ?: "online"
        if (p == null) return "offline"
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
