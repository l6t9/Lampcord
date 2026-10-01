package me.lampu.lampcord.shared.state.handlers

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageAcknowledge
import me.lampu.lampcord.shared.model.MessageReactionAdd
import me.lampu.lampcord.shared.model.MessageReactionRemove
import me.lampu.lampcord.shared.model.MessageReactionRemoveAll
import me.lampu.lampcord.shared.model.MessageReactionRemoveEmoji
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.utils.Logging

class MessageEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val messageStore: MessageStore,
    private val messageLogger: MessageLogger,
    private val readStateStore: ReadStateStore,
    private val entityStore: EntityStore,
    private val guildStore: GuildStore,
    private val navigationStore: NavigationStore,
    private val finderStore: FinderStore,
    private val channelApi: ChannelApi,
    private val typingStore: TypingStore,
    private val scope: CoroutineScope
) : GatewayEventHandler {
    private val fetchingDmChannels = mutableSetOf<String>()
    override val supportedEvents = setOf(
        "MESSAGE_CREATE", 
        "MESSAGE_UPDATE", 
        "MESSAGE_DELETE", 
        "MESSAGE_ACK",
        "MESSAGE_REACTION_ADD",
        "MESSAGE_REACTION_REMOVE",
        "MESSAGE_REACTION_REMOVE_ALL",
        "MESSAGE_REACTION_REMOVE_EMOJI"
    )

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "MESSAGE_CREATE" -> handleMessageCreate(data)
            "MESSAGE_UPDATE" -> handleMessageUpdate(data)
            "MESSAGE_DELETE" -> handleMessageDelete(data)
            "MESSAGE_ACK" -> handleMessageAck(data)
            "MESSAGE_REACTION_ADD" -> handleReactionAdd(data)
            "MESSAGE_REACTION_REMOVE" -> handleReactionRemove(data)
            "MESSAGE_REACTION_REMOVE_ALL" -> handleReactionRemoveAll(data)
            "MESSAGE_REACTION_REMOVE_EMOJI" -> handleReactionRemoveEmoji(data)
        }
    }

    private fun handleMessageCreate(data: JsonElement) {
        val message = try {
            json.decodeFromJsonElement<Message>(data)
        } catch (e: Exception) {
            me.lampu.lampcord.shared.utils.Logging.e("MessageHandler", "Failed to parse MESSAGE_CREATE", e)
            return
        }
        messageLogger.logMessage(message)
        
        message.author?.let { userStore.handleUserUpdate(it) }
        message.author?.let {
            typingStore.handleUserSentMessage(message.channel_id, it.id)
        }
        message.guild_id?.let { guildId ->
            message.member?.let { member ->
                val userId = message.author?.id ?: return@let
                userStore.cacheMember(guildId, userId, member.copy(user = message.author))
            }
        }
        
        finderStore.addRecent(message.channel_id)

        val currentChannel = entityStore.channels.value[message.channel_id]
        val currentUser = userStore.currentUser.value
        val author = message.author
        val recipientUser = if (author != null && author.id != currentUser?.id) author else null
        val recipientList = recipientUser?.let { listOf(it) }
        val recipientIdList = recipientUser?.let { listOf(it.id) }

        if (currentChannel != null) {
            val updatedRecipients = if (!currentChannel.recipients.isNullOrEmpty()) {
                currentChannel.recipients
            } else {
                recipientList
            }
            val updatedRecipientIds = if (!currentChannel.recipient_ids.isNullOrEmpty()) {
                currentChannel.recipient_ids
            } else {
                recipientIdList
            }
            val updated = currentChannel.copy(
                last_message_id = kotlinx.serialization.json.JsonPrimitive(message.id),
                recipients = updatedRecipients,
                recipient_ids = updatedRecipientIds
            )
            entityStore.updateChannel(updated)
            if (updated.guild_id == null && (updated.type == 1 || updated.type == 3)) {
                guildStore.handleChannelCreateOrUpdate(updated)
                if (updated.recipients.isNullOrEmpty()) {
                    hydrateDmChannel(message.channel_id, message.id)
                }
            }
        } else if (message.guild_id == null) {
            val dummyChannel = me.lampu.lampcord.shared.model.Channel(
                id = message.channel_id,
                type = 1,
                last_message_id = kotlinx.serialization.json.JsonPrimitive(message.id),
                recipients = recipientList,
                recipient_ids = recipientIdList
            )
            guildStore.handleChannelCreateOrUpdate(dummyChannel)
            if (dummyChannel.recipients.isNullOrEmpty()) {
                hydrateDmChannel(message.channel_id, message.id)
            }
        }

        if (message.author?.id == userStore.currentUser.value?.id) {
            messageStore.draftMessages.remove(message.channel_id)
        }

        messageStore.handleMessageCreate(message)
        
        if (navigationStore.shouldAutoAcknowledge(message.channel_id)) {
            scope.launch {
                readStateStore.ackMessage(message.channel_id, message.id)
            }
        } else {
            val currentUserId = userStore.currentUser.value?.id
            val myRoles = message.guild_id?.let { userStore.getMember(it, currentUserId ?: "")?.roles } ?: emptyList()
            readStateStore.handleMessageCreate(message, currentUserId, myRoles)
        }
    }

    private fun handleMessageUpdate(data: JsonElement) {
        val message = json.decodeFromJsonElement<Message>(data)
        val jsonObject = data as? kotlinx.serialization.json.JsonObject ?: return
        messageStore.handleMessageUpdate(message, jsonObject)
    }

    private fun handleMessageDelete(data: JsonElement) {
        val obj = data as? kotlinx.serialization.json.JsonObject ?: return
        val id = obj["id"]?.jsonPrimitive?.content ?: return
        val channelId = obj["channel_id"]?.jsonPrimitive?.content ?: return
        messageLogger.logDelete(channelId, id)
        messageStore.handleMessageDelete(id)
        readStateStore.handleMessageDelete(channelId, id)
    }

    private fun handleMessageAck(data: JsonElement) {
        val ack = json.decodeFromJsonElement<MessageAcknowledge>(data)
        readStateStore.handleMessageAck(ack)
    }

    private fun handleReactionAdd(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<MessageReactionAdd>(data)
            messageStore.handleReactionAdd(update)
        } catch (e: Exception) { }
    }

    private fun handleReactionRemove(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<MessageReactionRemove>(data)
            messageStore.handleReactionRemove(update)
        } catch (e: Exception) { }
    }

    private fun handleReactionRemoveAll(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<MessageReactionRemoveAll>(data)
            messageStore.handleReactionRemoveAll(update)
        } catch (e: Exception) { }
    }

    private fun handleReactionRemoveEmoji(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<MessageReactionRemoveEmoji>(data)
            messageStore.handleReactionRemoveEmoji(update)
        } catch (e: Exception) { }
    }

    private fun hydrateDmChannel(channelId: String, lastMessageId: String) {
        val channel = entityStore.channels.value[channelId]
        if (channel != null && !channel.recipients.isNullOrEmpty()) return
        if (!fetchingDmChannels.add(channelId)) return
        scope.launch {
            try {
                val privateChannels = channelApi.getPrivateChannels()
                val real = privateChannels.find { it.id == channelId } ?: channelApi.getChannel(channelId)
                if (real != null) {
                    val withLast = real.copy(last_message_id = kotlinx.serialization.json.JsonPrimitive(lastMessageId))
                    guildStore.handleChannelCreateOrUpdate(withLast)
                }
            } finally {
                fetchingDmChannels.remove(channelId)
            }
        }
    }
}
