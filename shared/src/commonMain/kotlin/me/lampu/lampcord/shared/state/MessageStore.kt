package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlin.random.Random

class MessageStore(
    private val discordClient: DiscordClient,
    private val scope: CoroutineScope
) {
    val messages = mutableStateListOf<Message>()
    private val messageTasks = mutableStateListOf<MessageTask>()
    private var isProcessingQueue = false
    
    var isLoadingHistory by mutableStateOf(false)
    var hasMoreHistory by mutableStateOf(true)

    fun clear() {
        messages.clear()
        hasMoreHistory = true
    }

    fun addMessages(newMessages: List<Message>) {
        messages.addAll(newMessages)
    }

    fun addMessageAtTop(message: Message) {
        if (messages.none { it.id == message.id }) {
            messages.add(0, message)
        }
    }

    fun handleMessageCreate(message: Message) {
        // Remove matching pending message
        message.nonce?.let { nonce ->
            messages.removeAll { it.nonce == nonce && it.isPending }
        }
        addMessageAtTop(message)
    }

    fun handleMessageUpdate(partialMessage: Message, dataObj: kotlinx.serialization.json.JsonObject) {
        val index = messages.indexOfFirst { it.id == partialMessage.id }
        if (index != -1) {
            val existing = messages[index]
            messages[index] = existing.copy(
                content = if ("content" in dataObj) partialMessage.content else existing.content,
                embeds = if ("embeds" in dataObj) partialMessage.embeds else existing.embeds,
                attachments = if ("attachments" in dataObj) partialMessage.attachments else existing.attachments,
                edited_timestamp = if ("edited_timestamp" in dataObj) partialMessage.edited_timestamp else existing.edited_timestamp
            )
        }
    }

    fun handleMessageDelete(id: String) {
        messages.removeAll { it.id == id }
    }

    fun sendMessage(
        channelId: String,
        content: String,
        currentUser: User,
        replyTo: String?,
        files: List<Pair<String, ByteArray>>,
        guildId: String?
    ) {
        val nowMillis = me.lampu.lampcord.shared.utils.getCurrentTimeMillis()
        val nonce = "${nowMillis}${Random.nextInt(1000, 9999)}"
        
        // Optimistic UI
        val pendingMessage = Message(
            id = nonce,
            channel_id = channelId,
            author = currentUser,
            content = content,
            timestamp = nowMillis.toString(),
            nonce = nonce,
            isPending = true,
            guild_id = guildId
        )
        
        messages.add(0, pendingMessage)
        messageTasks.add(MessageTask(nonce, channelId, content, replyTo, files))
        
        startQueueProcessing()
    }

    private fun startQueueProcessing() {
        if (isProcessingQueue || messageTasks.isEmpty()) return
        isProcessingQueue = true
        
        scope.launch {
            while (messageTasks.isNotEmpty()) {
                val task = messageTasks.first()
                val success = discordClient.sendMessage(
                    task.channelId,
                    task.content,
                    task.replyTo,
                    task.files,
                    task.nonce
                )
                
                if (success) {
                    messageTasks.removeAt(0)
                } else {
                    // Halt queue and mark error on the message
                    val index = messages.indexOfFirst { it.nonce == task.nonce && it.isPending }
                    if (index != -1) {
                        messages[index] = messages[index].copy(sendError = "Failed to send. Tap to retry.")
                    }
                    isProcessingQueue = false
                    return@launch
                }
            }
            isProcessingQueue = false
        }
    }

    fun retryMessage(message: Message) {
        val task = messageTasks.find { it.nonce == message.nonce } ?: return
        val index = messages.indexOfFirst { it.nonce == message.nonce && it.isPending }
        if (index != -1) {
            messages[index] = messages[index].copy(sendError = null)
        }
        startQueueProcessing()
    }

    fun deletePendingMessage(message: Message) {
        messages.removeAll { it.nonce == message.nonce && it.isPending }
        messageTasks.removeAll { it.nonce == message.nonce }
        startQueueProcessing()
    }
}

private data class MessageTask(
    val nonce: String,
    val channelId: String,
    val content: String,
    val replyTo: String?,
    val files: List<Pair<String, ByteArray>>
)
