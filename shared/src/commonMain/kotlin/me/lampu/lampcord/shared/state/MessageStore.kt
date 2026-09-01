package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonObject
import me.lampu.lampcord.shared.model.AllowedMentions
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageReactionAdd
import me.lampu.lampcord.shared.model.MessageReactionRemove
import me.lampu.lampcord.shared.model.MessageReactionRemoveAll
import me.lampu.lampcord.shared.model.MessageReactionRemoveEmoji
import me.lampu.lampcord.shared.model.MessageReaction
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.ReactionCountDetails
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Backoff
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlin.time.Clock
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeoutOrNull

class MessageStore(
    private val messageApi: MessageApi,
    private val channelApi: ChannelApi,
    private val userStore: UserStore,
    private val errorStore: AppErrorStore,
    private val selectionStore: SelectionStore,
    private val messageLogger: MessageLogger,
    private val scope: CoroutineScope
) {
    private companion object {
        const val CACHE_MAX_CHANNELS = 50
    }

    // Exact replica of Discord's internal message storage philosophy:
    // LRU channel access + sorted lists (TreeMap equivalent) for messages
    private val channelAccessOrder = mutableListOf<String>()
    private val messageCache = mutableMapOf<String, List<Message>>()
    
    private val _allMessages = MutableStateFlow<Map<String, List<Message>>>(emptyMap())
    
    val messages: StateFlow<List<Message>> = combine(_allMessages, selectionStore.activeChannelIdFlow) { all, activeId ->
        if (activeId == null) emptyList() else all[activeId] ?: emptyList()
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    private val _isLoadingHistory = MutableStateFlow(false)
    val isLoadingHistory: StateFlow<Boolean> = _isLoadingHistory.asStateFlow()

    private val _hasMoreHistory = MutableStateFlow<Map<String, Boolean>>(emptyMap())
    val hasMoreHistory: StateFlow<Boolean> = combine(_hasMoreHistory, selectionStore.activeChannelIdFlow) { map, activeId ->
        if (activeId == null) true else map[activeId] ?: true
    }.stateIn(scope, SharingStarted.Eagerly, true)

    val draftMessages = mutableStateMapOf<String, String>()
    var replyingTo by mutableStateOf<Message?>(null)
    var editingMessage by mutableStateOf<Message?>(null)
    val pendingFiles = mutableStateListOf<PendingFile>()
    val loadingMessages = mutableStateListOf<String>()
    val pinnedMessages = mutableStateListOf<Message>()

    var scrollToMessageId by mutableStateOf<String?>(null)
    var highlightedMessageId by mutableStateOf<String?>(null)

    private val queues = mutableMapOf<String, MessageQueue>()
    private val queuesMutex = Mutex()
    private var historyLoadingJob: Job? = null

    // Nonce tracking to replace local messages with server ones (Discord logic)
    private val messageNonceIds = mutableMapOf<String, String>()

    private fun transformOutgoingContent(content: String): String = me.lampu.lampcord.shared.utils.FreeNitroEmojis.transformOutgoing(content)

    private fun preprocess(message: Message): Message = me.lampu.lampcord.shared.utils.FreeNitroEmojis.preprocessIncoming(message)

    init {
        scope.launch {
            try {
                val text = ResourceLoader.readText("files/loading_messages.txt")
                val lines = text?.split("\n")
                    ?.map { it.trim() }
                    ?.filter { it.isNotEmpty() && !it.startsWith("#") }
                if (!lines.isNullOrEmpty()) {
                    loadingMessages.addAll(lines)
                }
            } catch (e: Exception) {
                loadingMessages.add("how")
            }
        }
    }

    fun showPinnedMessages() {
        val channel = selectionStore.selectedChannel ?: return
        scope.launch {
            try {
                val msgs = channelApi.getPinnedMessages(channel.id)
                pinnedMessages.clear()
                pinnedMessages.addAll(msgs)
            } catch (e: Exception) { }
        }
    }

    fun handleConnected() {
        scope.launch {
            queuesMutex.withLock {
                queues.values.forEach { it.process() }
            }
        }
    }

    fun clear() {
        messageCache.clear()
        channelAccessOrder.clear()
        _allMessages.value = emptyMap()
        messageNonceIds.clear()
        _hasMoreHistory.value = emptyMap()
        _isLoadingHistory.value = false
        historyLoadingJob?.cancel()
    }

    fun loadLoggedMessages(channelId: String) {
        scope.launch {
            val logged = messageLogger.getLoggedMessages(channelId)
            if (logged.isNotEmpty()) {
                addMessages(channelId, logged)
            }
        }
    }

    private fun updateAllMessagesFlow() {
        _allMessages.value = messageCache.toMap()
    }

    fun hasMessages(channelId: String): Boolean {
        return messageCache[channelId]?.isNotEmpty() == true
    }

    private fun recordAccess(channelId: String) {
        channelAccessOrder.remove(channelId)
        channelAccessOrder.add(channelId)
        if (channelAccessOrder.size > CACHE_MAX_CHANNELS) {
            val oldest = channelAccessOrder.removeAt(0)
            messageCache.remove(oldest)
        }
    }

    fun addMessages(channelId: String, newMessages: List<Message>) {
        val channelMessages = messageCache[channelId]?.toMutableList() ?: mutableListOf()
        val existingIds = channelMessages.map { it.id }.toSet()
        
        var changed = false
        newMessages.forEach { msg ->
            if (msg.id !in existingIds) {
                val preprocessed = preprocess(msg)
                channelMessages.add(preprocessed)
                messageLogger.logMessage(preprocessed)
                changed = true
            }
        }

        if (changed) {
            val sorted = channelMessages.sortedByDescending { it.id }
            // Keep the complete loaded history. Trimming this list while paging older
            // messages removes the viewport's anchor and makes reverse-layout scrolling
            // jump back when the cache limit is crossed.
            messageCache[channelId] = sorted
            recordAccess(channelId)
            updateAllMessagesFlow()
        }
    }

    fun handleMessageCreate(message: Message) {
        Logging.d("MessageStore", "Handling MESSAGE_CREATE: ${message.id}")
        val preprocessed = preprocess(message)
        val channelId = message.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: mutableListOf()
        
        val nonce = preprocessed.nonce
        var replaced = false
        
        if (nonce != null) {
            if (preprocessed.isPending) {
                messageNonceIds[nonce] = preprocessed.id
            } else {
                val localId = messageNonceIds.remove(nonce)
                if (localId != null) {
                    val idx = channelMessages.indexOfFirst { it.id == localId }
                    if (idx != -1) {
                        channelMessages[idx] = preprocessed
                        replaced = true
                    }
                }
            }
        }

        if (!replaced) {
            val existingIdx = channelMessages.indexOfFirst { it.id == preprocessed.id }
            if (existingIdx != -1) {
                channelMessages[existingIdx] = preprocessed
            } else {
                channelMessages.add(0, preprocessed)
            }
        }
        
        val sorted = channelMessages.sortedByDescending { it.id }
        messageCache[channelId] = sorted
        recordAccess(channelId)
        updateAllMessagesFlow()
    }

    fun handleMessageUpdate(message: Message, dataObj: JsonObject) {
        Logging.d("MessageStore", "Handling MESSAGE_UPDATE: ${message.id}")
        val channelId = message.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        val index = channelMessages.indexOfFirst { it.id == message.id }
        if (index != -1) {
            val existing = channelMessages[index]
            val merged = existing.merge(dataObj)
            val preprocessed = preprocess(merged)
            messageLogger.logUpdate(preprocessed)
            channelMessages[index] = preprocessed
            messageCache[channelId] = channelMessages
            updateAllMessagesFlow()
        }
    }

    fun handleMessageDelete(id: String) {
        Logging.d("MessageStore", "Handling MESSAGE_DELETE: $id")
        var changed = false
        messageCache.forEach { (chanId, channelMessages) ->
            val newList = if (me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled) {
                channelMessages.map {
                    if (it.id == id) { changed = true; it.copy(isDeleted = true) } else it
                }
            } else {
                val filtered = channelMessages.filter { msg -> msg.id != id }
                if (filtered.size != channelMessages.size) changed = true
                filtered
            }
            if (changed) messageCache[chanId] = newList
        }
        if (changed) updateAllMessagesFlow()
    }

    fun handleReactionAdd(update: MessageReactionAdd) {
        val channelId = update.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        var msgChanged = false
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) {
                msgChanged = true
                val reactions = msg.reactions?.toMutableList() ?: mutableListOf()
                val index = reactions.indexOfFirst { 
                    (it.emoji.id != null && it.emoji.id == update.emoji.id) || 
                    (it.emoji.id == null && it.emoji.name == update.emoji.name) 
                }
                val currentUserId = userStore.currentUser.value?.id
                val isMe = update.user_id == currentUserId
                
                if (index != -1) {
                    val reaction = reactions[index]
                    reactions[index] = reaction.copy(
                        count = if (update.burst) reaction.count else reaction.count + 1,
                        count_details = if (update.burst) {
                            reaction.count_details.copy(burst = reaction.count_details.burst + 1)
                        } else {
                            reaction.count_details.copy(normal = reaction.count_details.normal + 1)
                        },
                        me = if (!update.burst) reaction.me || isMe else reaction.me,
                        me_burst = if (update.burst) reaction.me_burst || isMe else reaction.me_burst,
                        burst_count = if (update.burst) (reaction.burst_count ?: 0) + 1 else reaction.burst_count
                    )
                } else {
                    reactions.add(MessageReaction(
                        emoji = update.emoji,
                        count = if (update.burst) 0 else 1,
                        count_details = if (update.burst) {
                            ReactionCountDetails(burst = 1, normal = 0)
                        } else {
                            ReactionCountDetails(burst = 0, normal = 1)
                        },
                        me = if (!update.burst) isMe else false,
                        me_burst = if (update.burst) isMe else false,
                        burst_count = if (update.burst) 1 else 0
                    ))
                }
                msg.copy(reactions = reactions)
            } else msg
        }
        if (msgChanged) {
            messageCache[channelId] = newList
            updateAllMessagesFlow()
        }
    }

    fun handleReactionRemove(update: MessageReactionRemove) {
        val channelId = update.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        var msgChanged = false
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) {
                msgChanged = true
                val reactions = msg.reactions?.toMutableList() ?: return@map msg
                val index = reactions.indexOfFirst { 
                    (it.emoji.id != null && it.emoji.id == update.emoji.id) || 
                    (it.emoji.id == null && it.emoji.name == update.emoji.name) 
                }
                if (index != -1) {
                    val reaction = reactions[index]
                    val currentUserId = userStore.currentUser.value?.id
                    val isMe = update.user_id == currentUserId
                    
                    val newNormalCount = if (!update.burst) (reaction.count_details.normal - 1).coerceAtLeast(0) else reaction.count_details.normal
                    val newBurstCount = if (update.burst) (reaction.count_details.burst - 1).coerceAtLeast(0) else reaction.count_details.burst
                    
                    if (newNormalCount == 0 && newBurstCount == 0) {
                        reactions.removeAt(index)
                    } else {
                        reactions[index] = reaction.copy(
                            count = newNormalCount,
                            count_details = ReactionCountDetails(burst = newBurstCount, normal = newNormalCount),
                            me = if (!update.burst && isMe) false else reaction.me,
                            me_burst = if (update.burst && isMe) false else reaction.me_burst,
                            burst_count = newBurstCount
                        )
                    }
                }
                msg.copy(reactions = reactions)
            } else msg
        }
        if (msgChanged) {
            messageCache[channelId] = newList
            updateAllMessagesFlow()
        }
    }

    fun handleReactionRemoveAll(update: MessageReactionRemoveAll) {
        val channelId = update.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) msg.copy(reactions = emptyList()) else msg
        }
        messageCache[channelId] = newList
        updateAllMessagesFlow()
    }

    fun handleReactionRemoveEmoji(update: MessageReactionRemoveEmoji) {
        val channelId = update.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) {
                val reactions = msg.reactions?.filter { 
                    !((it.emoji.id != null && it.emoji.id == update.emoji.id) || 
                      (it.emoji.id == null && it.emoji.name == update.emoji.name)) 
                }
                msg.copy(reactions = reactions)
            } else msg
        }
        messageCache[channelId] = newList
        updateAllMessagesFlow()
    }

    fun loadMoreMessages(channelId: String, guildId: String?, threadId: String?) {
        if (_isLoadingHistory.value) return
        Logging.i("MessageStore", "Loading more messages for $channelId")
        val currentChannelMessages = messageCache[channelId] ?: emptyList()
        val hasMore = _hasMoreHistory.value[channelId] ?: true
        if (!hasMore) return
        
        val before = currentChannelMessages.lastOrNull()?.id ?: return
        _isLoadingHistory.value = true
        historyLoadingJob = scope.launch {
            try {
                val more = messageApi.getChannelMessagesPage(threadId ?: channelId, before = before)
                if (more == null) {
                    // Keep history enabled so the next scroll event can retry.
                    return@launch
                }
                if (more.isEmpty()) {
                    _hasMoreHistory.update { it + (channelId to false) }
                } else {
                    more.forEach { msg -> 
                        msg.author?.let { author ->
                            msg.member?.let { m -> userStore.cacheMember(guildId ?: "", author.id, m) }
                        }
                    }
                    addMessages(channelId, more)
                }
            } catch (e: Exception) {
                println("Error loading history: ${e.message}")
            } finally {
                _isLoadingHistory.value = false
            }
        }
    }

    fun forwardMessage(targetChannel: Channel, message: Message) {
        val user = userStore.currentUser.value ?: return
        sendMessage(
            channelId = targetChannel.id,
            content = "",
            currentUser = user,
            forwardFrom = message,
            guildId = targetChannel.guild_id
        )
    }

    fun sendMessageDraft(
        content: String,
        stickerIds: List<String>? = null,
        allowedMentions: AllowedMentions? = null,
        poll: Poll? = null
    ) {
        val channel = selectionStore.selectedThread ?: selectionStore.selectedChannel ?: return
        val user = userStore.currentUser.value ?: return

        sendMessage(
            channelId = channel.id,
            content = content,
            currentUser = user,
            replyTo = replyingTo?.id,
            files = pendingFiles.toList(),
            guildId = selectionStore.selectedGuild?.id,
            stickerIds = stickerIds,
            allowedMentions = allowedMentions,
            poll = poll
        )
        replyingTo = null
        pendingFiles.clear()
        draftMessages.remove(channel.id)
    }

    fun sendMessage(
        channelId: String,
        content: String,
        currentUser: User,
        replyTo: String? = null,
        files: List<PendingFile> = emptyList(),
        guildId: String? = null,
        forwardFrom: Message? = null,
        stickerIds: List<String>? = null,
        allowedMentions: AllowedMentions? = null,
        poll: Poll? = null
    ) {
        val nonce = me.lampu.lampcord.shared.utils.Snowflake.nextId()
        val tempMessage = Message(
            id = nonce,
            channel_id = channelId,
            content = content,
            author = currentUser,
            timestamp = Clock.System.now().toString(),
            nonce = nonce,
            isPending = true,
            guild_id = guildId,
            poll = poll
        )
        
        handleMessageCreate(tempMessage)
        
        val task = MessageTask(nonce, channelId, content, replyTo, files, forwardFrom, stickerIds, allowedMentions, poll)
        
        scope.launch {
            queuesMutex.withLock {
                val queue = queues.getOrPut(channelId) { MessageQueue(channelId) }
                queue.enqueue(task)
            }
        }
    }

    private inner class MessageQueue(val channelId: String) {
        private val queue = mutableListOf<MessageTask>()
        private var isProcessing = false
        private val backoff = Backoff()

        fun enqueue(task: MessageTask) {
            queue.add(task)
            if (!isProcessing) {
                process()
            }
        }

        fun process() {
            if (isProcessing || queue.isEmpty()) return
            isProcessing = true
            
            scope.launch {

                            try {
                            while (true) {
                                val task = if (queue.isNotEmpty()) queue[0] else null
                                if (task == null) break
                                
                                try {
                                    val contentToSend = transformOutgoingContent(task.content)
                                    Logging.d("MessageStore", "Sending message: nonce=${task.nonce} channel=${task.channelId}")

                            val message = withTimeoutOrNull(60000.milliseconds) {
                                messageApi.sendMessage(
                                    channelId = task.channelId,
                                    content = contentToSend,
                                    nonce = task.nonce,
                                    replyTo = task.replyTo,
                                    forwardFrom = task.forwardFrom,
                                    files = task.files.map { f -> f.name to f.data },
                                    stickerIds = task.stickerIds,
                                    allowedMentions = task.allowedMentions,
                                    poll = task.poll
                                )
                            }

                            if (message != null) {
                                handleMessageCreate(message)
                                if (queue.isNotEmpty() && queue[0].nonce == task.nonce) queue.removeAt(0)
                                backoff.succeed()
                            } else {
                                handleFailure(task, "Failed to send")
                                if (queue.isNotEmpty() && queue[0].nonce == task.nonce) queue.removeAt(0)
                            }
                        } catch (e: Exception) {
                            handleFailure(task, e.message ?: "Error")
                            if (queue.isNotEmpty() && queue[0].nonce == task.nonce) queue.removeAt(0)
                        }
                        
                        val delayMs = backoff.nextDelay()
                        delay(delayMs)
                    }
                } finally {
                    isProcessing = false
                }
            }
        }

        private fun handleFailure(task: MessageTask, error: String) {
            val channelMessages = messageCache[task.channelId]?.toMutableList() ?: return
            val index = channelMessages.indexOfFirst { it.nonce == task.nonce }
            if (index != -1) {
                channelMessages[index] = channelMessages[index].copy(sendError = error, isPending = true)
                messageCache[task.channelId] = channelMessages
                updateAllMessagesFlow()
            }
            errorStore.pushError("Message failed: $error")
        }

        fun cancel(nonce: String) {
            queue.removeAll { it.nonce == nonce }
        }
        
        fun retry(task: MessageTask) {
            if (!queue.any { it.nonce == task.nonce }) {
                enqueue(task)
            }
        }
    }

    fun retryMessage(message: Message) {
        val channelId = message.channel_id
        val nonce = message.nonce ?: return
        
        scope.launch {
            queuesMutex.withLock {
                val queue = queues[channelId] ?: return@launch
                val task = MessageTask(nonce, channelId, message.content, null, emptyList(), null)
                
                val channelMessages = messageCache[channelId]?.toMutableList() ?: return@launch
                val index = channelMessages.indexOfFirst { it.nonce == nonce }
                if (index != -1) {
                    channelMessages[index] = channelMessages[index].copy(sendError = null, isPending = true)
                    messageCache[channelId] = channelMessages
                    updateAllMessagesFlow()
                }
                
                queue.retry(task)
            }
        }
    }

    fun deletePendingMessage(message: Message) {
        val channelId = message.channel_id
        val nonce = message.nonce ?: return
        val channelMessages = messageCache[channelId] ?: return
        val newList = channelMessages.filter { msg -> msg.nonce != nonce }
        messageCache[channelId] = newList
        updateAllMessagesFlow()
        scope.launch {
            queuesMutex.withLock {
                queues[channelId]?.cancel(nonce)
            }
        }
    }

    fun sendTyping(channelId: String) {
        // Silent typing only suppresses our outbound signal. Incoming typing
        // events are handled independently by TypingStore and remain visible.
        if (Settings.shared.silentTyping) return
        scope.launch {
            try {
                channelApi.triggerTyping(channelId)
            } catch (e: Exception) { }
        }
    }

    fun editMessage(message: Message, content: String) {
        val channelId = message.channel_id
        val id = message.id
        
        val current = messageCache[channelId]?.toMutableList() ?: return
        val idx = current.indexOfFirst { it.id == id }
        if (idx == -1) return
        
        val oldMessage = current[idx]
        val preprocessed = transformOutgoingContent(content)
        
        // Optimistic update
        current[idx] = oldMessage.copy(content = preprocessed, oldContent = oldMessage.content)
        messageCache[channelId] = current
        updateAllMessagesFlow()
        
        scope.launch {
            try {
                if (!messageApi.editMessage(channelId, id, preprocessed)) {
                    // Rollback
                    val rollback = messageCache[channelId]?.toMutableList() ?: return@launch
                    val rIdx = rollback.indexOfFirst { it.id == id }
                    if (rIdx != -1) {
                        rollback[rIdx] = oldMessage
                        messageCache[channelId] = rollback
                        updateAllMessagesFlow()
                    }
                    errorStore.pushError("Failed to edit message.")
                }
            } catch (e: Exception) {
                // Rollback
                val rollback = messageCache[channelId]?.toMutableList() ?: return@launch
                val rIdx = rollback.indexOfFirst { it.id == id }
                if (rIdx != -1) {
                    rollback[rIdx] = oldMessage
                    messageCache[channelId] = rollback
                    updateAllMessagesFlow()
                }
            }
        }
    }

    fun deleteMessage(message: Message) {
        val channelId = message.channel_id
        val id = message.id
        
        // Optimistic delete
        val cached = messageCache[channelId]
        handleMessageDelete(id)
        
        scope.launch {
            try {
                if (!messageApi.deleteMessage(channelId, id)) {
                    // Rollback
                    cached?.find { it.id == id }?.let { restored ->
                        val current = messageCache[channelId]?.toMutableList() ?: mutableListOf()
                        if (!current.any { it.id == id }) {
                            current.add(restored)
                            messageCache[channelId] = current.sortedByDescending { it.id }
                            updateAllMessagesFlow()
                        }
                    }
                    errorStore.pushError("Failed to delete message")
                }
            } catch (e: Exception) {
                // Rollback
                cached?.find { it.id == id }?.let { restored ->
                    val current = messageCache[channelId]?.toMutableList() ?: mutableListOf()
                    if (!current.any { it.id == id }) {
                        current.add(restored)
                        messageCache[channelId] = current.sortedByDescending { it.id }
                        updateAllMessagesFlow()
                    }
                }
            }
        }
    }

    fun pinMessage(message: Message) {
        val channelId = message.channel_id
        val id = message.id
        
        // Optimistic pin
        updateMessagePinState(channelId, id, true)
        
        scope.launch {
            try {
                if (!channelApi.pinMessage(channelId, id)) {
                    updateMessagePinState(channelId, id, false)
                    errorStore.pushError("Failed to pin message")
                }
            } catch (e: Exception) {
                updateMessagePinState(channelId, id, false)
            }
        }
    }

    fun unpinMessage(message: Message) {
        val channelId = message.channel_id
        val id = message.id
        
        // Optimistic unpin
        updateMessagePinState(channelId, id, false)
        
        scope.launch {
            try {
                if (!channelApi.unpinMessage(channelId, id)) {
                    updateMessagePinState(channelId, id, true)
                    errorStore.pushError("Failed to unpin message")
                }
            } catch (e: Exception) {
                updateMessagePinState(channelId, id, true)
            }
        }
    }

    private fun updateMessagePinState(channelId: String, messageId: String, pinned: Boolean) {
        val current = messageCache[channelId]?.toMutableList() ?: return
        val idx = current.indexOfFirst { it.id == messageId }
        if (idx != -1) {
            current[idx] = current[idx].copy(pinned = pinned)
            messageCache[channelId] = current
            updateAllMessagesFlow()
        }
    }

    fun toggleReaction(message: Message, emoji: me.lampu.lampcord.shared.model.Emoji) {
        val channelId = message.channel_id
        val emojiStr = if (emoji.id != null) "${emoji.name}:${emoji.id}" else emoji.name ?: return
        val currentUserId = userStore.currentUser.value?.id ?: return
        
        val reactions = message.reactions ?: emptyList()
        val existing = reactions.find { 
            (it.emoji.id != null && it.emoji.id == emoji.id) || 
            (it.emoji.id == null && it.emoji.name == emoji.name) 
        }
        
        val isAdding = existing == null || !existing.me

        // Optimistic update
        if (isAdding) {
            handleReactionAdd(MessageReactionAdd(
                user_id = currentUserId,
                channel_id = channelId,
                message_id = message.id,
                emoji = emoji
            ))
        } else {
            handleReactionRemove(MessageReactionRemove(
                user_id = currentUserId,
                channel_id = channelId,
                message_id = message.id,
                emoji = emoji
            ))
        }

        scope.launch {
            try {
                val success = if (isAdding) {
                    messageApi.addReaction(channelId, message.id, emojiStr)
                } else {
                    messageApi.removeReaction(channelId, message.id, emojiStr)
                }
                
                if (!success) {
                    // Rollback if failed
                    if (isAdding) {
                        handleReactionRemove(MessageReactionRemove(
                            user_id = currentUserId,
                            channel_id = channelId,
                            message_id = message.id,
                            emoji = emoji
                        ))
                    } else {
                        handleReactionAdd(MessageReactionAdd(
                            user_id = currentUserId,
                            channel_id = channelId,
                            message_id = message.id,
                            emoji = emoji
                        ))
                    }
                    errorStore.pushError("Failed to update reaction")
                }
            } catch (e: Exception) {
                // Rollback on exception
                if (isAdding) {
                    handleReactionRemove(MessageReactionRemove(
                        user_id = currentUserId,
                        channel_id = channelId,
                        message_id = message.id,
                        emoji = emoji
                    ))
                } else {
                    handleReactionAdd(MessageReactionAdd(
                        user_id = currentUserId,
                        channel_id = channelId,
                        message_id = message.id,
                        emoji = emoji
                    ))
                }
            }
        }
    }

    fun isMessageMentioningMe(message: Message, currentUser: User?, currentMember: Member?): Boolean {
        val myId = currentUser?.id ?: return false
        if (message.author?.id == myId) return false
        if (message.mentions.any { it.id == myId }) return true
        if (currentMember != null && message.mention_roles.any { it in currentMember.roles }) return true
        if (message.mention_everyone) return true
        return false
    }
}

private data class MessageTask(
    val nonce: String,
    val channelId: String,
    val content: String,
    val replyTo: String?,
    val files: List<PendingFile>,
    val forwardFrom: Message?,
    val stickerIds: List<String>? = null,
    val allowedMentions: AllowedMentions? = null,
    val poll: Poll? = null
)
