package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.api.AllowedMentions
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

class MessageStore(
    private val discordClient: DiscordClient,
    private val userStore: UserStore,
    private val errorStore: AppErrorStore,
    private val scope: CoroutineScope
) {
    val messages = mutableStateListOf<Message>()
    private val messageTasks = mutableStateListOf<MessageTask>()
    private var isProcessingQueue = false

    var isLoadingHistory by mutableStateOf(false)
    var hasMoreHistory by mutableStateOf(true)
    
    private var historyLoadingJob: Job? = null

    fun clear() {
        messages.clear()
        messageTasks.clear()
        hasMoreHistory = true
        isLoadingHistory = false
        historyLoadingJob?.cancel()
    }

    fun addMessages(newMessages: List<Message>) {
        messages.addAll(newMessages)
    }

    fun handleMessageCreate(message: Message) {
        val index = messages.indexOfFirst { it.id == message.id || (message.nonce != null && it.nonce == message.nonce) }
        if (index != -1) {
            messages[index] = message
        } else {
            messages.add(0, message)
        }
    }

    fun handleMessageUpdate(message: Message, dataObj: JsonObject) {
        val index = messages.indexOfFirst { it.id == message.id }
        if (index != -1) {
            val existing = messages[index]
            messages[index] = existing.merge(dataObj)
        }
    }

    fun handleMessageDelete(id: String) {
        messages.removeAll { it.id == id }
    }

    fun loadMoreMessages(channelId: String, guildId: String?, threadId: String?) {
        if (isLoadingHistory || !hasMoreHistory) return
        val before = messages.lastOrNull()?.id ?: return
        isLoadingHistory = true
        historyLoadingJob = scope.launch {
            try {
                val more = discordClient.getChannelMessages(threadId ?: channelId, before = before)
                if (more.isEmpty()) {
                    hasMoreHistory = false
                } else {
                    more.forEach { msg -> 
                        msg.author?.let { author ->
                            msg.member?.let { m -> userStore.cacheMember(guildId ?: "", author.id, m) }
                        }
                    }
                    messages.addAll(more)
                }
            } catch (e: Exception) {
                println("Error loading history: ${e.message}")
            } finally {
                isLoadingHistory = false
            }
        }
    }

    fun sendMessage(
        channelId: String,
        content: String,
        currentUser: User,
        replyTo: String? = null,
        files: List<Pair<String, ByteArray>> = emptyList(),
        guildId: String? = null,
        forwardFrom: Message? = null,
        stickerIds: List<String>? = null,
        allowedMentions: AllowedMentions? = null
    ) {
        val nonce = getCurrentTimeMillis().toString()
        val tempMessage = Message(
            id = nonce,
            channel_id = channelId,
            content = content,
            author = currentUser,
            timestamp = "",
            nonce = nonce,
            isPending = true,
            guild_id = guildId
        )
        messages.add(0, tempMessage)
        
        messageTasks.add(MessageTask(nonce, channelId, content, replyTo, files, forwardFrom, stickerIds, allowedMentions))
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
                        files = task.files,
                        stickerIds = task.stickerIds,
                        allowedMentions = task.allowedMentions
                    )
                    
                    if (success) {
                        messageTasks.removeAt(0)
                    } else {
                        val index = messages.indexOfFirst { it.nonce == task.nonce }
                        if (index != -1) {
                            messages[index] = messages[index].copy(sendError = "Failed to send", isPending = false)
                        }
                        errorStore.pushError("Failed to send message.")
                        messageTasks.removeAt(0)
                    }
                } catch (e: Exception) {
                    val index = messages.indexOfFirst { it.nonce == task.nonce }
                    if (index != -1) {
                        messages[index] = messages[index].copy(sendError = e.message ?: "Error", isPending = false)
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
            val index = messages.indexOf(message)
            if (index != -1) {
                messages[index] = message.copy(sendError = null, isPending = true)
                messageTasks.add(MessageTask(message.nonce!!, message.channel_id, message.content, null, emptyList(), null))
                if (!isProcessingQueue) startQueueProcessing()
            }
        }
    }

    fun deletePendingMessage(message: Message) {
        messages.remove(message)
        messageTasks.removeAll { it.nonce == message.nonce }
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
    val files: List<Pair<String, ByteArray>>,
    val forwardFrom: Message?,
    val stickerIds: List<String>? = null,
    val allowedMentions: AllowedMentions? = null
)
