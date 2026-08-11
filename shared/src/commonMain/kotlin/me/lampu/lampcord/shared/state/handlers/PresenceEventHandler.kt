package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.PresenceStore

class PresenceEventHandler(
    private val json: Json,
    private val presenceStore: PresenceStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("PRESENCE_UPDATE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "PRESENCE_UPDATE" -> handlePresenceUpdate(data)
        }
    }

    private fun handlePresenceUpdate(data: JsonElement) {
        try {
            val presence = json.decodeFromJsonElement<PresenceUpdate>(data)
            presenceStore.handlePresenceUpdate(presence)
        } catch (e: Exception) { }
    }
}
