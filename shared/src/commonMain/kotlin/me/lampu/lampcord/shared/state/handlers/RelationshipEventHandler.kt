package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.model.Relationship
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.RelationshipStore

class RelationshipEventHandler(
    private val json: Json,
    private val relationshipStore: RelationshipStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("RELATIONSHIP_ADD", "RELATIONSHIP_REMOVE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "RELATIONSHIP_ADD" -> handleRelationshipAdd(data)
            "RELATIONSHIP_REMOVE" -> handleRelationshipRemove(data)
        }
    }

    private fun handleRelationshipAdd(data: JsonElement) {
        try {
            val rel = json.decodeFromJsonElement<Relationship>(data)
            relationshipStore.handleRelationshipAdd(rel)
        } catch (e: Exception) { }
    }

    private fun handleRelationshipRemove(data: JsonElement) {
        try {
            val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
            relationshipStore.handleRelationshipRemove(id)
        } catch (e: Exception) { }
    }
}
