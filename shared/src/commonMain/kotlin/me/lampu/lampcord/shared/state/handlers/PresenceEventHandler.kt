package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.model.Session
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.UserStore

class PresenceEventHandler(
    private val json: Json,
    private val presenceStore: PresenceStore,
    private val userStore: UserStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("PRESENCE_UPDATE", "SESSIONS_REPLACE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "PRESENCE_UPDATE" -> handlePresenceUpdate(data)
            "SESSIONS_REPLACE" -> handleSessionsReplace(data)
        }
    }

    private fun handlePresenceUpdate(data: JsonElement) {
        try {
            val presence = json.decodeFromJsonElement<PresenceUpdate>(data)
            presenceStore.handlePresenceUpdate(presence)
            
            presence.user?.let { userStore.handleUserUpdate(it) }
        } catch (e: Exception) { }
    }

    private fun handleSessionsReplace(data: JsonElement) {
        try {
            val sessions = json.decodeFromJsonElement<List<Session>>(data)
            val userId = userStore.currentUser.value?.id ?: return
            presenceStore.handleSessions(userId, sessions)
        } catch (e: Exception) { }
    }
}
