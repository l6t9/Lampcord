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
import kotlinx.coroutines.flow.collect
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
import me.lampu.lampcord.shared.model.TextReplaceRule
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Backoff
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.Snowflake
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
    private val settingsStore: SettingsStore,
    private val scope: CoroutineScope
) {
    private companion object {
        const val CACHE_MAX_CHANNELS = 50
        const val CACHE_MAX_MESSAGES_PER_CHANNEL = 300
        const val CACHE_MAX_MESSAGES_ACTIVE_CHANNEL = 800
        const val TYPING_EMISSION_INTERVAL_MS = 10_000L
    }

    private val channelAccessOrder = mutableListOf<String>()
    private val lastTypingEmissionMillis = mutableMapOf<String, Long>()
    private val messageCache = mutableMapOf<String, List<Message>>()

    private val _activeMessages = MutableStateFlow<List<Message>>(emptyList())

    val messages: StateFlow<List<Message>> = _activeMessages.asStateFlow()

    private val _loadingHistoryChannels = MutableStateFlow<Set<String>>(emptySet())
    val isLoadingHistory: StateFlow<Boolean> = combine(_loadingHistoryChannels, selectionStore.activeChannelIdFlow) { loading, activeId ->
        activeId != null && activeId in loading
    }.stateIn(scope, SharingStarted.Eagerly, false)

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

    private val queues = mutableMapOf<String, PendingSendQueue>()
    private val queuesMutex = Mutex()
    private var historyLoadingJobs = mutableMapOf<String, Job>()

    private val messageNonceIds = mutableMapOf<String, String>()
    private val messageTasks = mutableMapOf<String, MessageTask>()

    private class CompiledReplaceRules(
        val source: List<TextReplaceRule>,
        val regexes: List<Regex?>
    )

    private var compiledRuleCache: CompiledReplaceRules? = null

    private fun compiledRuleRegexes(rules: List<TextReplaceRule>): List<Regex?> {
        compiledRuleCache?.let { if (it.source === rules) return it.regexes }
        val compiled = CompiledReplaceRules(
            rules,
            rules.map { rule ->
                if (!rule.isRegex) {
                    null
                } else {
                    runCatching {
                        Regex(
                            rule.pattern,
                            if (rule.ignoreCase) setOf(RegexOption.IGNORE_CASE) else emptySet()
                        )
                    }.getOrNull()
                }
            }
        )
        compiledRuleCache = compiled
        return compiled.regexes
    }

    private fun transformOutgoingContent(content: String): String {
        var processed = content
        val rules = settingsStore.textReplaceRules
        val regexes = compiledRuleRegexes(rules)
        rules.forEachIndexed { index, rule ->
            if (!rule.enabled || !rule.matchUnsent) return@forEachIndexed
            processed = try {
                if (rule.isRegex) {
                    val regex = regexes[index] ?: return@forEachIndexed
                    processed.replace(regex, rule.replacement)
                } else {
                    processed.replace(rule.pattern, rule.replacement, ignoreCase = rule.ignoreCase)
                }
            } catch (e: Exception) { processed }
        }
        return me.lampu.lampcord.shared.utils.FreeNitroEmojis.transformOutgoing(processed)
    }

    private fun preprocess(message: Message): Message {
        var preprocessed = me.lampu.lampcord.shared.utils.FreeNitroEmojis.preprocessIncoming(message)

        var content = preprocessed.content
        var changed = false
        val rules = settingsStore.textReplaceRules
        val regexes = compiledRuleRegexes(rules)
        rules.forEachIndexed { index, rule ->
            if (!rule.enabled || !rule.matchSent) return@forEachIndexed
            try {
                val newContent = if (rule.isRegex) {
                    val regex = regexes[index] ?: return@forEachIndexed
                    content.replace(regex, rule.replacement)
                } else {
                    content.replace(rule.pattern, rule.replacement, ignoreCase = rule.ignoreCase)
                }
                if (newContent != content) {
                    content = newContent
                    changed = true
                }
            } catch (e: Exception) { }
        }

        if (changed) {
            preprocessed = preprocessed.copy(content = content)
        }

        return preprocessed
    }

    init {
        scope.launch {
            var previousChannelId: String? = null
            selectionStore.activeChannelIdFlow.collect { channelId ->
                previousChannelId?.let { trimChannelMessages(it) }
                previousChannelId = channelId
                _activeMessages.value =
                    if (channelId == null) emptyList() else messageCache[channelId] ?: emptyList()
            }
        }
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
        _activeMessages.value = emptyList()
        _hasMoreHistory.value = emptyMap()
        _loadingHistoryChannels.value = emptySet()
        historyLoadingJobs.values.forEach { it.cancel() }
        historyLoadingJobs.clear()
        // Pending sends belong to the account being logged out; cancelling and
        // removing them here prevents them from being drained under the next account.
        // Clear the nonce/task maps in the same place and order as the pending
        // queues so a logout cannot leave stale send state behind.
        scope.launch {
            clearPendingQueues(queues, queuesMutex)
            messageNonceIds.clear()
            messageTasks.clear()
        }
    }

    fun loadLoggedMessages(channelId: String) {
        scope.launch {
            val logged = messageLogger.getLoggedMessages(channelId)
            if (logged.isNotEmpty()) {
                addMessages(channelId, logged)
            }
        }
    }

    private fun publishMessages(channelId: String) {
        if (selectionStore.activeChannelIdFlow.value == channelId) {
            _activeMessages.value = messageCache[channelId] ?: emptyList()
        }
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
            publishMessages(oldest)
            _hasMoreHistory.update { it + (oldest to true) }
        }
        trimChannelMessages(channelId)
    }

    private fun trimChannelMessages(channelId: String) {
        val active = selectionStore.activeChannelIdFlow.value == channelId
        val limit = if (active) CACHE_MAX_MESSAGES_ACTIVE_CHANNEL else CACHE_MAX_MESSAGES_PER_CHANNEL

        val current = messageCache[channelId] ?: return
        if (current.size <= limit) return
        val pending = current.filter { it.isPending }
        val keep = (limit - pending.size).coerceAtLeast(1)
        val nonPending = current.filterNot { it.isPending }
        if (nonPending.size <= keep) return
        val kept = nonPending.take(keep) + pending
        messageCache[channelId] = kept.sortedByDescending { it.id.toLongOrNull() ?: 0L }
        if (nonPending.size > keep) {
            _hasMoreHistory.update { it + (channelId to true) }
        }
    }

    fun addMessages(channelId: String, newMessages: List<Message>) {
        val channelMessages = messageCache[channelId] ?: emptyList()
        val existingById = HashMap<String, Message>().apply { channelMessages.forEach { put(it.id, it) } }

        val incoming = ArrayList<Message>(newMessages.size)
        val replacedIds = HashSet<String>()
        newMessages.forEach { msg ->
            val preprocessed = preprocess(msg)
            val existing = existingById[msg.id]
            if (existing == null || hasNewerData(existing, preprocessed)) {
                incoming.add(preprocessed)
                existingById[msg.id] = preprocessed
                if (existing != null) replacedIds.add(msg.id)
                messageLogger.logMessage(preprocessed)
            }
        }

        if (incoming.isNotEmpty()) {
            incoming.sortByDescending { it.id.toLongOrNull() ?: 0L }
            val base = if (replacedIds.isEmpty()) channelMessages else channelMessages.filterNot { it.id in replacedIds }
            messageCache[channelId] = mergeDescending(base, incoming)
            recordAccess(channelId)
            publishMessages(channelId)
        }
    }

    private fun hasNewerData(existing: Message, incoming: Message): Boolean {
        if (incoming.edited_timestamp != null && incoming.edited_timestamp != existing.edited_timestamp) return true
        if (existing.edited_timestamp != null && incoming.edited_timestamp == null) return false
        if (incoming.content != existing.content) return true
        if (incoming.embeds != existing.embeds) return true
        if (incoming.attachments != existing.attachments) return true
        if (incoming.reactions != existing.reactions) return true
        if (incoming.pinned != existing.pinned) return true
        if (incoming.flags != existing.flags) return true
        return false
    }

    private fun mergeDescending(existing: List<Message>, incoming: List<Message>): List<Message> {
        if (existing.isEmpty()) return incoming
        if (incoming.isEmpty()) return existing
        val merged = ArrayList<Message>(existing.size + incoming.size)
        var left = 0
        var right = 0
        while (left < existing.size && right < incoming.size) {
            val leftId = existing[left].id.toLongOrNull() ?: 0L
            val rightId = incoming[right].id.toLongOrNull() ?: 0L
            if (leftId >= rightId) {
                merged.add(existing[left++])
            } else {
                merged.add(incoming[right++])
            }
        }
        while (left < existing.size) merged.add(existing[left++])
        while (right < incoming.size) merged.add(incoming[right++])
        return merged
    }

    fun mergeMessages(channelId: String, newMessages: List<Message>) {
        val channelMessages = messageCache[channelId]?.toMutableList() ?: mutableListOf()
        var changed = false
        newMessages.forEach { msg ->
            val preprocessed = preprocess(msg)
            val idx = channelMessages.indexOfFirst { it.id == preprocessed.id }
            if (idx != -1) {
                if (channelMessages[idx] != preprocessed) {
                    channelMessages[idx] = preprocessed
                    messageLogger.logMessage(preprocessed)
                    changed = true
                }
            } else {
                channelMessages.add(preprocessed)
                messageLogger.logMessage(preprocessed)
                changed = true
            }
        }
        if (changed) {
            val sorted = channelMessages.sortedByDescending { it.id.toLongOrNull() ?: 0L }
            messageCache[channelId] = sorted
            recordAccess(channelId)
            publishMessages(channelId)
        }
    }

    fun latestCachedMessageId(channelId: String): String? =
        messageCache[channelId]?.firstOrNull()?.id

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
            } else if (preprocessed.isPending) {
                // Optimistic messages have no snowflake id yet; keep them above
                // the loaded history until the nonce is replaced by the real one.
                channelMessages.add(0, preprocessed)
            } else {
                // Out-of-order MESSAGE_CREATE events must not reverse the list.
                val newId = preprocessed.id.toLongOrNull() ?: 0L
                val insertAt = channelMessages.indexOfFirst { existing ->
                    !existing.isPending && (existing.id.toLongOrNull() ?: 0L) < newId
                }
                if (insertAt == -1) channelMessages.add(preprocessed) else channelMessages.add(insertAt, preprocessed)
            }
        }

        messageCache[channelId] = channelMessages
        recordAccess(channelId)
        publishMessages(channelId)
    }

    fun handleMessageUpdate(message: Message, dataObj: JsonObject) {
        Logging.d("MessageStore", "Handling MESSAGE_UPDATE: ${message.id}")
        val channelId = message.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: mutableListOf()
        val index = channelMessages.indexOfFirst { it.id == message.id }
        val base = if (index != -1) channelMessages[index] else message
        val merged = base.merge(dataObj)
        val preprocessed = preprocess(merged)
        // Log edits even for channels that are not in the in-memory cache.
        messageLogger.logUpdate(preprocessed)
        if (index != -1) {
            channelMessages[index] = preprocessed
            messageCache[channelId] = channelMessages
            publishMessages(channelId)
        }
    }

    fun handleMessageDelete(id: String, channelId: String? = null) {
        Logging.d("MessageStore", "Handling MESSAGE_DELETE: $id")
        val keepInLog = me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled
        val entries = if (channelId != null) {
            messageCache[channelId]?.let { listOf(channelId to it) }.orEmpty()
        } else {
            messageCache.toList()
        }
        entries.forEach { (chanId, channelMessages) ->
            var channelChanged = false
            val newList = if (keepInLog) {
                channelMessages.map { msg ->
                    if (msg.id == id) {
                        channelChanged = true
                        msg.copy(isDeleted = true)
                    } else msg
                }
            } else {
                channelMessages.filter { msg ->
                    if (msg.id == id) {
                        channelChanged = true
                        false
                    } else true
                }
            }
            if (channelChanged) {
                messageCache[chanId] = newList
                publishMessages(chanId)
            }
        }
    }

    /** Removes an ephemeral interaction response locally; there is no server message to delete. */
    suspend fun dismissEphemeral(message: Message) {
        val channelId = message.channel_id
        val channelMessages = messageCache[channelId] ?: return
        messageCache[channelId] = channelMessages.filterNot { it.id == message.id }
        publishMessages(channelId)
        messageLogger.logDelete(message.id)
    }

    fun handleReactionAdd(update: MessageReactionAdd) {
        val channelId = update.channel_id
        val isBurst = update.burst || update.type == 1
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        var msgChanged = false
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) {
                val reactions = msg.reactions?.toMutableList() ?: mutableListOf()
                val index = reactions.indexOfFirst {
                    (it.emoji.id != null && it.emoji.id == update.emoji.id) ||
                    (it.emoji.id == null && it.emoji.name == update.emoji.name)
                }
                val currentUserId = userStore.currentUser.value?.id
                val isMe = update.user_id == currentUserId
                val existing = if (index != -1) reactions[index] else null

                if (existing != null && !isBurst && isMe && existing.me) {
                    return@map msg
                }

                msgChanged = true
                if (index != -1) {
                    reactions[index] = ReactionCountMath.afterAdd(reactions[index], isBurst, isMe)
                } else {
                    reactions.add(ReactionCountMath.initial(update.emoji, isBurst, isMe))
                }
                msg.copy(reactions = reactions)
            } else msg
        }
        if (msgChanged) {
            messageCache[channelId] = newList
            publishMessages(channelId)
        }
    }

    fun handleReactionRemove(update: MessageReactionRemove) {
        val channelId = update.channel_id
        val isBurst = update.burst || update.type == 1
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        var msgChanged = false
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) {
                val reactions = msg.reactions?.toMutableList() ?: return@map msg
                val index = reactions.indexOfFirst {
                    (it.emoji.id != null && it.emoji.id == update.emoji.id) ||
                    (it.emoji.id == null && it.emoji.name == update.emoji.name)
                }
                if (index != -1) {
                    val reaction = reactions[index]
                    val currentUserId = userStore.currentUser.value?.id
                    val isMe = update.user_id == currentUserId

                    if (!isBurst && isMe && !reaction.me) {
                        return@map msg
                    }

                    msgChanged = true

                    val removed = ReactionCountMath.afterRemove(reaction, isBurst, isMe)
                    if (removed.count_details.normal == 0 && removed.count_details.burst == 0) {
                        reactions.removeAt(index)
                    } else {
                        reactions[index] = removed
                    }
                }
                msg.copy(reactions = reactions)
            } else msg
        }
        if (msgChanged) {
            messageCache[channelId] = newList
            publishMessages(channelId)
        }
    }

    fun handleReactionRemoveAll(update: MessageReactionRemoveAll) {
        val channelId = update.channel_id
        val channelMessages = messageCache[channelId]?.toMutableList() ?: return
        val newList = channelMessages.map { msg ->
            if (msg.id == update.message_id) msg.copy(reactions = emptyList()) else msg
        }
        messageCache[channelId] = newList
        publishMessages(channelId)
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
        publishMessages(channelId)
    }

    fun loadMoreMessages(channelId: String, guildId: String?, threadId: String?) {
        if (channelId in _loadingHistoryChannels.value) return
        Logging.i("MessageStore", "Loading more messages for $channelId")
        val currentChannelMessages = messageCache[channelId] ?: emptyList()
        val hasMore = _hasMoreHistory.value[channelId] ?: true
        if (!hasMore) return

        val before = currentChannelMessages.lastOrNull()?.id
        _loadingHistoryChannels.update { it + channelId }
        historyLoadingJobs[channelId] = scope.launch {
            try {
                val more = messageApi.getChannelMessagesPage(threadId ?: channelId, before = before)
                if (more == null) {
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
                _loadingHistoryChannels.update { it - channelId }
                historyLoadingJobs.remove(channelId)
            }
        }
    }

    fun jumpToMessage(channelId: String, messageId: String) {
        if (messageCache[channelId]?.any { it.id == messageId } == true) {
            scrollToMessageId = messageId
            return
        }

        scope.launch {
            try {
                val around = messageApi.getMessagesAround(channelId, messageId)
                if (around.isNotEmpty()) {
                    around.forEach { msg ->
                        msg.author?.let { author ->
                            msg.member?.let { member ->
                                userStore.cacheMember(msg.guild_id ?: "", author.id, member)
                            }
                        }
                    }
                    addMessages(channelId, around)
                }
            } catch (e: Exception) {
                Logging.e("MessageStore", "Failed to jump to message $messageId: ${e.message}")
            } finally {
                scrollToMessageId = messageId
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
        val nonce = Snowflake.nextId()
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
        messageTasks[nonce] = task
        if (messageTasks.size > CACHE_MAX_MESSAGES_ACTIVE_CHANNEL) {
            messageTasks.keys.firstOrNull()?.let { messageTasks.remove(it) }
        }

        scope.launch {
            queuesMutex.withLock {
                val queue = queues.getOrPut(channelId) { MessageQueue(channelId) }
                queue.enqueue(task)
            }
        }
    }

    private inner class MessageQueue(val channelId: String) : PendingSendQueue {
        private val queue = mutableListOf<MessageTask>()
        private var isProcessing = false
        private var processingJob: Job? = null
        private val backoff = Backoff()

        override fun enqueue(task: MessageTask) {
            queue.add(task)
            if (!isProcessing) {
                process()
            }
        }

        override fun process() {
            if (isProcessing || queue.isEmpty()) return
            isProcessing = true

            processingJob = scope.launch {
                try {
                    while (true) {
                        val task = queue.firstOrNull() ?: break
                        var succeeded = false
                        var failureDelayMs = 0L
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
                                messageTasks.remove(task.nonce)
                                succeeded = true
                                backoff.succeed()
                            } else {
                                handleFailure(task, "Failed to send")
                                failureDelayMs = backoff.fail()
                            }
                        } catch (e: Exception) {
                            handleFailure(task, e.message ?: "Error")
                            failureDelayMs = backoff.fail()
                        }

                        if (queue.isNotEmpty() && queue[0].nonce == task.nonce) queue.removeAt(0)
                        // Backoff is for failures only; successful sends must not
                        // be throttled by a random delay before the next message.
                        if (!succeeded) delay(failureDelayMs)
                    }
                } finally {
                    isProcessing = false
                    processingJob = null
                }
            }
        }

        private fun handleFailure(task: MessageTask, error: String) {
            val channelMessages = messageCache[task.channelId]?.toMutableList() ?: return
            val index = channelMessages.indexOfFirst { it.nonce == task.nonce }
            if (index != -1) {
                channelMessages[index] = channelMessages[index].copy(sendError = error, isPending = true)
                messageCache[task.channelId] = channelMessages
                publishMessages(task.channelId)
            }
            errorStore.pushError("Message failed: $error")
        }

        override fun cancel(nonce: String) {
            queue.removeAll { it.nonce == nonce }
        }

        override fun retry(task: MessageTask) {
            if (!queue.any { it.nonce == task.nonce }) {
                enqueue(task)
            }
        }

        override fun cancelAll() {
            processingJob?.cancel()
            processingJob = null
            isProcessing = false
            queue.clear()
        }
    }

    fun retryMessage(message: Message) {
        val channelId = message.channel_id
        val nonce = message.nonce ?: return

        scope.launch {
            queuesMutex.withLock {
                val queue = queues[channelId] ?: return@launch
                val task = messageTasks[nonce] ?: MessageTask(nonce, channelId, message.content, null, emptyList(), null)

                val channelMessages = messageCache[channelId]?.toMutableList() ?: return@launch
                val index = channelMessages.indexOfFirst { it.nonce == nonce }
                if (index != -1) {
                    channelMessages[index] = channelMessages[index].copy(sendError = null, isPending = true)
                    messageCache[channelId] = channelMessages
                    publishMessages(channelId)
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
        messageNonceIds.remove(nonce)
        messageTasks.remove(nonce)
        publishMessages(channelId)
        scope.launch {
            queuesMutex.withLock {
                queues[channelId]?.cancel(nonce)
            }
        }
    }

    fun sendTyping(channelId: String) {
        if (Settings.shared.silentTyping) return
        val now = getCurrentTimeMillis()
        val last = lastTypingEmissionMillis[channelId] ?: 0L
        if (now - last < TYPING_EMISSION_INTERVAL_MS) return
        lastTypingEmissionMillis[channelId] = now
        scope.launch {
            try {
                channelApi.triggerTyping(channelId)
            } catch (e: Exception) { }
        }
    }

    fun resetTypingEmission(channelId: String) {
        lastTypingEmissionMillis.remove(channelId)
    }

    fun editMessage(message: Message, content: String) {
        val channelId = message.channel_id
        val id = message.id

        val current = messageCache[channelId]?.toMutableList() ?: return
        val idx = current.indexOfFirst { it.id == id }
        if (idx == -1) return

        val oldMessage = current[idx]
        val preprocessed = transformOutgoingContent(content)

        current[idx] = oldMessage.copy(content = preprocessed, oldContent = oldMessage.content)
        messageCache[channelId] = current
        publishMessages(channelId)

        scope.launch {
            try {
                if (!messageApi.editMessage(channelId, id, preprocessed)) {
                    val rollback = messageCache[channelId]?.toMutableList() ?: return@launch
                    val rIdx = rollback.indexOfFirst { it.id == id }
                    if (rIdx != -1) {
                        rollback[rIdx] = oldMessage
                        messageCache[channelId] = rollback
                        publishMessages(channelId)
                    }
                    errorStore.pushError("Failed to edit message.")
                }
            } catch (e: Exception) {
                val rollback = messageCache[channelId]?.toMutableList() ?: return@launch
                val rIdx = rollback.indexOfFirst { it.id == id }
                if (rIdx != -1) {
                    rollback[rIdx] = oldMessage
                    messageCache[channelId] = rollback
                    publishMessages(channelId)
                }
            }
        }
    }

    fun deleteMessage(message: Message) {
        val channelId = message.channel_id
        val id = message.id

        val cached = messageCache[channelId]
        handleMessageDelete(id, channelId)

        scope.launch {
            try {
                if (!messageApi.deleteMessage(channelId, id)) {
                    cached?.find { it.id == id }?.let { restoreAfterFailedDelete(channelId, it) }
                    errorStore.pushError("Failed to delete message")
                }
            } catch (e: Exception) {
                cached?.find { it.id == id }?.let { restoreAfterFailedDelete(channelId, it) }
            }
        }
    }

    private fun restoreAfterFailedDelete(channelId: String, restored: Message) {
        val current = messageCache[channelId]?.toMutableList() ?: mutableListOf()
        val index = current.indexOfFirst { it.id == restored.id }
        // With logging enabled the delete only marks the cached message as
        // deleted, so replace it unconditionally and clear that flag.
        val original = restored.copy(isDeleted = false)
        if (index != -1) current[index] = original else current.add(original)
        messageCache[channelId] = current.sortedByDescending { it.id.toLongOrNull() ?: 0L }
        publishMessages(channelId)
    }

    fun pinMessage(message: Message) {
        val channelId = message.channel_id
        val id = message.id

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
            publishMessages(channelId)
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
        if (message.mentions.any { it.id == myId }) return true
        if (currentMember != null && message.mention_roles.any { it in currentMember.roles }) return true
        if (message.mention_everyone) return true
        return false
    }
}

internal interface PendingSendQueue {
    fun enqueue(task: MessageTask)
    fun process()
    fun cancel(nonce: String)
    fun retry(task: MessageTask)
    fun cancelAll()
}

/** Cancels and removes every pending outgoing queue, used on logout/account switch. */
internal suspend fun clearPendingQueues(
    queues: MutableMap<String, PendingSendQueue>,
    mutex: Mutex
) {
    mutex.withLock {
        queues.values.forEach { it.cancelAll() }
        queues.clear()
    }
}

internal data class MessageTask(
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
