package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.*
import kotlinx.serialization.json.JsonPrimitive

class ReadStateStore(private val channelApi: ChannelApi) {
    private val _readStates = MutableStateFlow<Map<String, ReadState>>(emptyMap())
    val readStates: StateFlow<Map<String, ReadState>> = _readStates.asStateFlow()

    fun handleReady(ready: ReadyPayload) {
        val states = mutableMapOf<String, ReadState>()
        ready.read_state?.entries?.forEach { state ->
            states[state.id] = state
        }
        _readStates.value = states
    }

    fun clear() {
        _readStates.value = emptyMap()
    }

    fun handleMessageAck(ack: MessageAcknowledge) {
        _readStates.update { current ->
            val existing = current[ack.channel_id]
            val updated = existing?.copy(
                last_message_id = JsonPrimitive(ack.message_id),
                mention_count = ack.mention_count ?: 0
            )
                ?: ReadState(
                    id = ack.channel_id,
                    last_message_id = JsonPrimitive(ack.message_id),
                    mention_count = ack.mention_count ?: 0
                )
            current + (ack.channel_id to updated)
        }
    }

    fun isUnread(channel: Channel): Boolean {
        val lastMsgId = channel.lastMessageId() ?: return false
        val state = _readStates.value[channel.id] ?: return true
        val ackedId = state.lastMessageId() ?: "0"
        if (ackedId == "0") return lastMsgId != "0"
        return (lastMsgId.toLongOrNull() ?: 0L) > (ackedId.toLongOrNull() ?: 0L)
    }

    fun getMentionCount(channelId: String): Int {
        return _readStates.value[channelId]?.mention_count ?: 0
    }

    suspend fun ackMessage(channelId: String, messageId: String) {
        _readStates.update { current ->
            val existing = current[channelId]
            if (existing != null) {
                val currentAckId = existing.lastMessageId() ?: "0"
                if ((messageId.toLongOrNull() ?: 0L) > (currentAckId.toLongOrNull() ?: 0L)) {
                    current + (channelId to existing.copy(last_message_id = JsonPrimitive(messageId), mention_count = 0))
                } else current
            } else {
                current + (channelId to ReadState(id = channelId, last_message_id = JsonPrimitive(messageId), mention_count = 0))
            }
        }
        channelApi.ackMessage(channelId, messageId)
    }
}
