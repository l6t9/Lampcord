package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.MemberListUpdate
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.MemberListStore

class MemberListEventHandler(
    private val json: Json,
    private val memberListStore: MemberListStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("GUILD_MEMBER_LIST_UPDATE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_MEMBER_LIST_UPDATE" -> handleMemberListUpdate(data)
        }
    }

    private fun handleMemberListUpdate(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<MemberListUpdate>(data)
            memberListStore.handleMemberListUpdate(update)
        } catch (e: Exception) { }
    }
}
