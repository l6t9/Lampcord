package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.*
import kotlinx.serialization.json.JsonPrimitive

class ReadStateStore(private val discordClient: DiscordClient) {
    val readStates = mutableStateMapOf<String, ReadState>()

    fun handleReady(ready: ReadyPayload) {
        readStates.clear()
        ready.read_state?.entries?.forEach { state ->
            readStates[state.id] = state
        }
    }

    fun clear() {
        readStates.clear()
    }

    fun handleMessageAck(ack: MessageAcknowledge) {
        val state = readStates[ack.channel_id]
        if (state != null) {
            readStates[ack.channel_id] = state.copy(
                last_message_id = JsonPrimitive(ack.message_id),
                mention_count = ack.mention_count ?: 0
            )
        } else {
            readStates[ack.channel_id] = ReadState(
                id = ack.channel_id,
                last_message_id = JsonPrimitive(ack.message_id),
                mention_count = ack.mention_count ?: 0
            )
        }
    }

    fun isUnread(channel: Channel): Boolean {
        val lastMsgId = channel.lastMessageId() ?: return false
        val state = readStates[channel.id] ?: return true
        val ackedId = state.lastMessageId() ?: "0"
        if (ackedId == "0") return lastMsgId != "0"
        return (lastMsgId.toLongOrNull() ?: 0L) > (ackedId.toLongOrNull() ?: 0L)
    }

    fun getMentionCount(channelId: String): Int {
        return readStates[channelId]?.mention_count ?: 0
    }

    suspend fun ackMessage(channelId: String, messageId: String) {
        val state = readStates[channelId]
        if (state != null) {
            val currentAckId = state.lastMessageId() ?: "0"
            if ((messageId.toLongOrNull() ?: 0L) > (currentAckId.toLongOrNull() ?: 0L)) {
                readStates[channelId] = state.copy(last_message_id = JsonPrimitive(messageId), mention_count = 0)
            }
        } else {
            readStates[channelId] = ReadState(id = channelId, last_message_id = JsonPrimitive(messageId), mention_count = 0)
        }
        discordClient.ackMessage(channelId, messageId)
    }
}
