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
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonObject
import me.lampu.lampcord.shared.model.AllowedMentions
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.utils.Backoff
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlin.time.Clock
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeoutOrNull

class MessageStore(
    private val discordClient: DiscordClient,
    private val userStore: UserStore,
    private val errorStore: AppErrorStore,
    private val selectionStore: SelectionStore,
    private val scope: CoroutineScope
) {
    private val _messages = MutableStateFlow<List<Message>>(emptyList())
    val messages: StateFlow<List<Message>> = _messages.asStateFlow()

    private val _isLoadingHistory = MutableStateFlow(false)
    val isLoadingHistory: StateFlow<Boolean> = _isLoadingHistory.asStateFlow()

    private val _hasMoreHistory = MutableStateFlow(true)
    val hasMoreHistory: StateFlow<Boolean> = _hasMoreHistory.asStateFlow()

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

    private val emojiMarkdownRegexCompound = Regex(
        """(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?"""
    )
    private val emojiMarkdownRegexSingle = Regex(
        """^(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?$"""
    )
    private val fEmojiRegex = Regex("""<(a?):F_([a-zA-Z0-9_]+):(\d+)>""")

    private fun preprocess(message: Message): Message {
        val settings = me.lampu.lampcord.shared.settings.Settings.shared
        if (!settings.freeNitroEmojis || !settings.realmojis) return message

        val content = message.content
        val embeds = message.embeds.toMutableList()
        var newContent = content

        val regex = if (settings.compoundRealmojis) emojiMarkdownRegexCompound else emojiMarkdownRegexSingle
        val matches = regex.findAll(content)
        var changed = false
        matches.forEach { match ->
            val fullMatch = match.value
            val url = match.groupValues[1]
            val id = match.groupValues[2]
            val ext = match.groupValues[3]
            
            val embedIndex = embeds.indexOfFirst { 
                it.url == url || it.thumbnail?.url == url || it.image?.url == url || 
                it.thumbnail?.proxy_url == url || it.image?.proxy_url == url
            }
            if (embedIndex != -1) {
                embeds.removeAt(embedIndex)
                changed = true
            }
            
            var name = "emoji"
            val queryParams = url.substringAfter('?', "").split('&')
            queryParams.forEach { param ->
                val parts = param.split('=')
                if (parts[0] == "name" && parts.size > 1) {
                    name = parts[1]
                }
            }
            
            val animated = ext == "gif" || url.contains("animated=true")
            val emojiTag = "<${if (animated) "a" else ""}:F_$name:$id>"
            if (newContent.contains(fullMatch)) {
                newContent = newContent.replace(fullMatch, emojiTag)
                changed = true
            }
        }

        return if (changed) message.copy(content = newContent, embeds = embeds) else message
    }

    private fun transformOutgoingContent(content: String): String {
        return fEmojiRegex.replace(content) { match ->
            val animated = match.groupValues[1] == "a"
            val name = match.groupValues[2]
            val id = match.groupValues[3]
            val ext = if (animated) "gif" else "png"
            val url = "https://cdn.discordapp.com/emojis/$id.$ext?size=48&name=$name"
            "[$name]($url)"
        }
    }

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
                val msgs = discordClient.getPinnedMessages(channel.id)
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
        _messages.value = emptyList()
        _hasMoreHistory.value = true
        _isLoadingHistory.value = false
        historyLoadingJob?.cancel()
    }

    fun addMessages(newMessages: List<Message>) {
        _messages.update { current ->
            val existingIds = current.map { it.id }.toSet()
            current + newMessages.filter { it.id !in existingIds }.map { preprocess(it) }
        }
    }

    fun handleMessageCreate(message: Message) {
        val preprocessed = preprocess(message)
        _messages.update { current ->
            val index = current.indexOfFirst { it.id == preprocessed.id || (preprocessed.nonce != null && it.nonce == preprocessed.nonce) }
            if (index != -1) {
                current.toMutableList().apply { set(index, preprocessed) }
            } else {
                listOf(preprocessed) + current
            }
        }
    }

    fun handleMessageUpdate(message: Message, dataObj: JsonObject) {
        _messages.update { current ->
            val index = current.indexOfFirst { it.id == message.id }
            if (index != -1) {
                val existing = current[index]
                val merged = existing.merge(dataObj)
                current.toMutableList().apply { set(index, preprocess(merged)) }
            } else current
        }
    }

    fun handleMessageDelete(id: String) {
        _messages.update { it.filter { msg -> msg.id != id } }
    }

    fun loadMoreMessages(channelId: String, guildId: String?, threadId: String?) {
        if (_isLoadingHistory.value || !_hasMoreHistory.value) return
        val before = _messages.value.lastOrNull()?.id ?: return
        _isLoadingHistory.value = true
        historyLoadingJob = scope.launch {
            try {
                val more = discordClient.getChannelMessages(threadId ?: channelId, before = before)
                if (more.isEmpty()) {
                    _hasMoreHistory.value = false
                } else {
                    more.forEach { msg -> 
                        msg.author?.let { author ->
                            msg.member?.let { m -> userStore.cacheMember(guildId ?: "", author.id, m) }
                        }
                    }
                    val preprocessed = more.map { preprocess(it) }
                    _messages.update { it + preprocessed }
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
        val nonce = getCurrentTimeMillis().toString()
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
        
        val preprocessed = preprocess(tempMessage)
        
        // Add to UI if it's the current channel
        if (selectionStore.selectedChannel?.id == channelId || selectionStore.selectedThread?.id == channelId) {
            _messages.update { listOf(preprocessed) + it }
        }
        
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
                            val settings = me.lampu.lampcord.shared.settings.Settings.shared
                            val contentToSend = if (settings.freeNitroEmojis && settings.realmojis) {
                                transformOutgoingContent(task.content)
                            } else task.content

                            val message = withTimeoutOrNull(60000.milliseconds) {
                                discordClient.sendMessage(
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
            _messages.update { current ->
                val index = current.indexOfFirst { it.nonce == task.nonce }
                if (index != -1) {
                    current.toMutableList().apply { 
                        set(index, get(index).copy(sendError = error, isPending = true))
                    }
                } else current
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
        
        // Find if there's already a task or create a new one
        scope.launch {
            queuesMutex.withLock {
                val queue = queues[channelId] ?: return@launch
                // If it was already failed, it's not in the queue. 
                // We should probably store the task info even when it fails to retry it easily.
                // For now, we'll recreate a simple task.
                val task = MessageTask(nonce, channelId, message.content, null, emptyList(), null)
                
                _messages.update { current ->
                    val index = current.indexOfFirst { it.nonce == nonce }
                    if (index != -1) {
                        current.toMutableList().apply {
                            set(index, get(index).copy(sendError = null, isPending = true))
                        }
                    } else current
                }
                
                queue.retry(task)
            }
        }
    }

    fun deletePendingMessage(message: Message) {
        val channelId = message.channel_id
        val nonce = message.nonce ?: return
        _messages.update { it.filter { msg -> msg.nonce != nonce } }
        scope.launch {
            queuesMutex.withLock {
                queues[channelId]?.cancel(nonce)
            }
        }
    }

    fun sendTyping(channelId: String) {
        scope.launch {
            try {
                discordClient.triggerTyping(channelId)
            } catch (e: Exception) { }
        }
    }

    fun editMessage(message: Message, content: String) {
        scope.launch {
            try {
                val settings = me.lampu.lampcord.shared.settings.Settings.shared
                val contentToSend = if (settings.freeNitroEmojis && settings.realmojis) {
                    transformOutgoingContent(content)
                } else content
                
                if (!discordClient.editMessage(message.channel_id, message.id, contentToSend)) {
                    errorStore.pushError("Failed to edit message.")
                }
            } catch (e: Exception) {
                errorStore.pushError("Error editing message: ${e.message}")
            }
        }
    }

    fun deleteMessage(message: Message) {
        scope.launch {
            try {
                discordClient.deleteMessage(message.channel_id, message.id)
            } catch (e: Exception) { }
        }
    }

    fun pinMessage(message: Message) {
        scope.launch {
            discordClient.pinMessage(message.channel_id, message.id)
        }
    }

    fun unpinMessage(message: Message) {
        scope.launch {
            discordClient.unpinMessage(message.channel_id, message.id)
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
