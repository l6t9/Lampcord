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
    private val entityStore: EntityStore,
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
        
        // Cache in entity store
        message.guild_id?.let { guildId ->
            message.member?.let { member ->
                entityStore.updateMember(guildId, member.copy(user = message.author))
            }
        }
        message.author?.let { entityStore.updateUser(it) }
        
        finderStore.addRecent(message.channel_id)

        if (navigationStore.selectedChannel?.id == message.channel_id || 
            navigationStore.selectedThread?.id == message.channel_id) {
            messageStore.handleMessageCreate(message)
            scope.launch {
                readStateStore.ackMessage(message.channel_id, message.id)
            }
        } else {
            // Mention handling logic (omitted for brevity, or moved to ReadStateStore)
            // ...
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
