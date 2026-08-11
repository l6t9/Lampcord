package me.lampu.lampcord.shared.state.handlers

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageAcknowledge
import me.lampu.lampcord.shared.state.*

class MessageEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val messageStore: MessageStore,
    private val readStateStore: ReadStateStore,
    private val navigationStore: NavigationStore,
    private val finderStore: FinderStore,
    private val scope: CoroutineScope
) : GatewayEventHandler {
    override val supportedEvents = setOf("MESSAGE_CREATE", "MESSAGE_UPDATE", "MESSAGE_DELETE", "MESSAGE_ACK")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "MESSAGE_CREATE" -> handleMessageCreate(data)
            "MESSAGE_UPDATE" -> handleMessageUpdate(data)
            "MESSAGE_DELETE" -> handleMessageDelete(data)
            "MESSAGE_ACK" -> handleMessageAck(data)
        }
    }

    private fun handleMessageCreate(data: JsonElement) {
        val message = json.decodeFromJsonElement<Message>(data)
        
        // Cache author and member in UserStore (StoreUsers / StoreMembers)
        message.author?.let { userStore.handleUserUpdate(it) }
        message.guild_id?.let { guildId ->
            message.member?.let { member ->
                val userId = message.author?.id ?: return@let
                userStore.cacheMember(guildId, userId, member.copy(user = message.author))
            }
        }
        
        finderStore.addRecent(message.channel_id)

        if (navigationStore.selectedChannel?.id == message.channel_id || 
            navigationStore.selectedThread?.id == message.channel_id) {
            messageStore.handleMessageCreate(message)
            scope.launch {
                readStateStore.ackMessage(message.channel_id, message.id)
            }
        }
    }

    private fun handleMessageUpdate(data: JsonElement) {
        val message = json.decodeFromJsonElement<Message>(data)
        val jsonObject = data as? kotlinx.serialization.json.JsonObject ?: return
        messageStore.handleMessageUpdate(message, jsonObject)
    }

    private fun handleMessageDelete(data: JsonElement) {
        // ...
    }

    private fun handleMessageAck(data: JsonElement) {
        val ack = json.decodeFromJsonElement<MessageAcknowledge>(data)
        readStateStore.handleMessageAck(ack)
    }
}
