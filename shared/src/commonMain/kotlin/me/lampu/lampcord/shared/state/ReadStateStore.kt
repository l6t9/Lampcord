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
    
    private val mentionedMessageIds = mutableMapOf<String, MutableSet<String>>()
    private val _recentIds = MutableStateFlow<Map<String, String>>(emptyMap())
    val recentIds: StateFlow<Map<String, String>> = _recentIds.asStateFlow()

    private val settings get() = me.lampu.lampcord.shared.settings.Settings.shared
    private val json = kotlinx.serialization.json.Json

    private var localAcks: MutableMap<String, String> = loadLocalAcks()

    private fun loadLocalAcks(): MutableMap<String, String> {
        val saved = settings.readStateAcks
        if (saved.isBlank()) return mutableMapOf()
        return try {
            json.decodeFromString<Map<String, String>>(saved).toMutableMap()
        } catch (_: Exception) {
            mutableMapOf()
        }
    }

    private fun persistLocalAcks() {
        settings.readStateAcks = json.encodeToString(localAcks)
    }

    fun handleReady(ready: ReadyPayload) {
        val states = mutableMapOf<String, ReadState>()
        ready.read_state?.entries?.forEach { state ->
            states[state.id] = state
        }

        // READY only carries read positions for channels Discord already tracks, so channels
        // read on this device can be missing from it entirely. Without the locally kept
        // baseline those channels fall back to "no read state", which reads as unread and
        // comes back on every launch.
        localAcks.forEach { (channelId, ackedId) ->
            val acked = ackedId.toLongOrNull() ?: return@forEach
            val remote = states[channelId]?.lastMessageId()?.toLongOrNull() ?: 0L
            if (acked > remote) {
                states[channelId] = ReadState(
                    id = channelId,
                    last_message_id = JsonPrimitive(ackedId),
                    mention_count = states[channelId]?.mention_count ?: 0,
                )
            }
        }

        _readStates.value = states
        val recent = mutableMapOf<String, String>()
        states.forEach { (channelId, state) ->
            state.lastMessageId()?.let { recent[channelId] = it }
        }
        _recentIds.value = recent
    }

    fun clear() {
        _readStates.value = emptyMap()
        mentionedMessageIds.clear()
        _recentIds.value = emptyMap()
    }

    private fun updateMostRecentId(channelId: String, messageId: String) {
        val id = messageId.toLongOrNull() ?: return
        val current = _recentIds.value[channelId]?.toLongOrNull() ?: 0L
        if (id > current) {
            _recentIds.value = _recentIds.value + (channelId to messageId)
        }
    }

    private fun ReadState?.withAck(messageId: String, mentionCount: Int): ReadState {
        val previous = this ?: ReadState(id = "", last_message_id = null, mention_count = 0)
        val previousId = previous.lastMessageId()?.toLongOrNull() ?: 0L
        val id = messageId.toLongOrNull() ?: 0L
        return when {
            id > previousId -> previous.copy(last_message_id = JsonPrimitive(messageId), mention_count = mentionCount)
            id == previousId -> previous.copy(mention_count = mentionCount)
            else -> previous
        }
    }

    fun handleMessageAck(ack: MessageAcknowledge) {
        _readStates.update { current ->
            val existing = current[ack.channel_id]

            val ids = mentionedMessageIds[ack.channel_id]
            ids?.let { set ->
                val toRemove = set.filter { (it.toLongOrNull() ?: 0L) <= (ack.message_id.toLongOrNull() ?: 0L) }
                set.removeAll(toRemove.toSet())
            }

            val updated = existing.withAck(ack.message_id, ack.mention_count ?: 0).copy(id = ack.channel_id)
            current + (ack.channel_id to updated)
        }
    }

    fun handleMessageCreate(message: Message, currentUserId: String?, myRoles: List<String> = emptyList()) {
        updateMostRecentId(message.channel_id, message.id)

        val hasMention = message.author?.id != currentUserId && (
            message.mention_everyone ||
            message.mentions.any { it.id == currentUserId } ||
            message.mention_roles.any { myRoles.contains(it) }
        )
        
        if (hasMention) {
            val state = _readStates.value[message.channel_id]
            val ackedId = state?.lastMessageId() ?: "0"
            if ((message.id.toLongOrNull() ?: 0L) <= (ackedId.toLongOrNull() ?: 0L)) return

            val ids = mentionedMessageIds.getOrPut(message.channel_id) { mutableSetOf() }
            if (ids.add(message.id)) {
                _readStates.update { current ->
                    val existing = current[message.channel_id]
                    val updated = existing?.copy(mention_count = (existing.mention_count) + 1)
                        ?: ReadState(id = message.channel_id, mention_count = 1)
                    current + (message.channel_id to updated)
                }
            }
        }
    }

    fun handleMessageDelete(channelId: String, messageId: String) {
        if (mentionedMessageIds[channelId]?.contains(messageId) == true) {
            mentionedMessageIds[channelId]?.remove(messageId)
            _readStates.update { current ->
                val existing = current[channelId]
                if (existing != null && existing.mention_count > 0) {
                    current + (channelId to existing.copy(mention_count = existing.mention_count - 1))
                } else current
            }
        }
    }

    fun isUnread(channel: Channel): Boolean {
        val channelMsgId = channel.lastMessageId()?.toLongOrNull() ?: 0L
        val recentMsgId = _recentIds.value[channel.id]?.toLongOrNull() ?: 0L
        val lastMsgId = maxOf(channelMsgId, recentMsgId)
        if (lastMsgId <= 0L) return false
        val ackedId = _readStates.value[channel.id]?.lastMessageId()?.toLongOrNull()
            ?: localAcks[channel.id]?.toLongOrNull()
            ?: 0L
        return lastMsgId > ackedId
    }

    fun getMentionCount(channelId: String): Int {
        return _readStates.value[channelId]?.mention_count ?: 0
    }

    private fun rememberAck(channelId: String, messageId: String) {
        val id = messageId.toLongOrNull() ?: return
        val current = localAcks[channelId]?.toLongOrNull() ?: 0L
        if (id <= current) return
        localAcks[channelId] = messageId
        persistLocalAcks()
    }

    suspend fun ackMessage(channelId: String, messageId: String) {
        rememberAck(channelId, messageId)
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

    suspend fun ackBulk(readStates: Map<String, String>): Boolean {
        val validStates = readStates.filterValues { it.toLongOrNull()?.let { id -> id > 0L } == true }
        if (validStates.isEmpty()) return true
        if (!channelApi.ackBulk(validStates)) return false

        applyAcknowledgements(validStates)
        return true
    }

    /** Applies confirmed acknowledgements without issuing another REST request. */
    fun applyAcknowledgements(readStates: Map<String, String>) {
        val validStates = readStates.filterValues { it.toLongOrNull()?.let { id -> id > 0L } == true }
        if (validStates.isEmpty()) return
        validStates.forEach { (channelId, messageId) -> rememberAck(channelId, messageId) }
        _readStates.update { current ->
            current + validStates.mapValues { (channelId, messageId) ->
                (current[channelId] ?: ReadState(id = channelId)).withAck(messageId, 0)
                    .copy(id = channelId)
            }
        }
    }

    /**
     * The newest message we know about in a channel, combining the channel's
     * own last_message_id with the ids of messages seen over the gateway.
     */
    fun mostRecentMessageId(channel: Channel): String? {
        val channelMsgId = channel.lastMessageId()?.toLongOrNull() ?: 0L
        val recentMsgId = _recentIds.value[channel.id]?.toLongOrNull() ?: 0L
        return maxOf(channelMsgId, recentMsgId)
            .takeIf { it > 0L }
            ?.toString()
    }
}
