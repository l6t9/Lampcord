package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.TypingStart
import me.lampu.lampcord.shared.state.*

class TypingEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val typingStore: TypingStore,
    private val currentUserIdProvider: () -> String?
) : GatewayEventHandler {
    override val supportedEvents = setOf("TYPING_START")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "TYPING_START" -> handleTypingStart(data)
        }
    }

    private fun handleTypingStart(data: JsonElement) {
        try {
            val typing = json.decodeFromJsonElement<TypingStart>(data)
            
            // Cache member/user if provided
            typing.guild_id?.let { guildId ->
                typing.member?.let { member ->
                    val userId = typing.user_id
                    userStore.cacheMember(guildId, userId, member)
                }
            }

            typingStore.handleTypingStart(typing.channel_id, typing.user_id, currentUserIdProvider())
        } catch (e: Exception) { }
    }
}
