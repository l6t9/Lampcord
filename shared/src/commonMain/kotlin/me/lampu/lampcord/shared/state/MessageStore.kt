package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.api.AllowedMentions
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.settings.Settings
import kotlin.time.Clock

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

    private val messageTasks = mutableListOf<MessageTask>()
    private var isProcessingQueue = false
    private var historyLoadingJob: Job? = null

    init {
        scope.launch {
            try {
                val text = ResourceLoader.readText("files/loading_messages.txt")
                val lines = text?.split("\n")
                    ?.map { it.trim() }
                    ?.filter { it.isNotEmpty() && !it.startsWith("#") }
                if (lines != null && lines.isNotEmpty()) {
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

    fun clear() {
        _messages.value = emptyList()
        messageTasks.clear()
        _hasMoreHistory.value = true
        _isLoadingHistory.value = false
        historyLoadingJob?.cancel()
    }

    fun addMessages(newMessages: List<Message>) {
        _messages.update { current ->
            val existingIds = current.map { it.id }.toSet()
            current + newMessages.filter { it.id !in existingIds }
        }
    }

    fun handleMessageCreate(message: Message) {
        _messages.update { current ->
            val index = current.indexOfFirst { it.id == message.id || (message.nonce != null && it.nonce == message.nonce) }
            if (index != -1) {
                current.toMutableList().apply { set(index, message) }
            } else {
                listOf(message) + current
            }
        }
    }

    fun handleMessageUpdate(message: Message, dataObj: JsonObject) {
        _messages.update { current ->
            val index = current.indexOfFirst { it.id == message.id }
            if (index != -1) {
                val existing = current[index]
                current.toMutableList().apply { set(index, existing.merge(dataObj)) }
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
                    _messages.update { it + more }
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
        _messages.update { listOf(tempMessage) + it }
        
        messageTasks.add(MessageTask(nonce, channelId, content, replyTo, files, forwardFrom, stickerIds, allowedMentions, poll))
        if (!isProcessingQueue) {
            startQueueProcessing()
        }
    }

    private fun startQueueProcessing() {
        isProcessingQueue = true
        scope.launch {
            while (messageTasks.isNotEmpty()) {
                val task = messageTasks[0]
                try {
                    val success = discordClient.sendMessage(
                        channelId = task.channelId,
                        content = task.content,
                        nonce = task.nonce,
                        replyTo = task.replyTo,
                        forwardFrom = task.forwardFrom,
                        files = task.files.map { it.name to it.data },
                        stickerIds = task.stickerIds,
                        allowedMentions = task.allowedMentions,
                        poll = task.poll
                    )
                    
                    if (success) {
                        messageTasks.removeAt(0)
                    } else {
                        _messages.update { current ->
                            val index = current.indexOfFirst { it.nonce == task.nonce }
                            if (index != -1) {
                                current.toMutableList().apply { 
                                    set(index, get(index).copy(sendError = "Failed to send", isPending = false))
                                }
                            } else current
                        }
                        errorStore.pushError("Failed to send message.")
                        messageTasks.removeAt(0)
                    }
                } catch (e: Exception) {
                    _messages.update { current ->
                        val index = current.indexOfFirst { it.nonce == task.nonce }
                        if (index != -1) {
                            current.toMutableList().apply { 
                                set(index, get(index).copy(sendError = e.message ?: "Error", isPending = false))
                            }
                        } else current
                    }
                    errorStore.pushError("Error sending message: ${e.message}")
                    messageTasks.removeAt(0)
                }
                delay(500)
            }
            isProcessingQueue = false
        }
    }

    fun retryMessage(message: Message) {
        val task = messageTasks.find { it.nonce == message.nonce }
        if (task == null) {
            val index = _messages.value.indexOf(message)
            if (index != -1) {
                _messages.update { current ->
                    current.toMutableList().apply {
                        set(index, message.copy(sendError = null, isPending = true))
                    }
                }
                messageTasks.add(MessageTask(message.nonce!!, message.channel_id, message.content, null, emptyList(), null))
                if (!isProcessingQueue) startQueueProcessing()
            }
        }
    }

    fun deletePendingMessage(message: Message) {
        _messages.update { it - message }
        messageTasks.removeAll { it.nonce == message.nonce }
    }

    fun sendTyping(channelId: String) {
        scope.launch {
            try {
                discordClient.sendTyping(channelId)
            } catch (e: Exception) { }
        }
    }

    fun editMessage(message: Message, content: String) {
        scope.launch {
            try {
                if (!discordClient.editMessage(message.channel_id, message.id, content)) {
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
