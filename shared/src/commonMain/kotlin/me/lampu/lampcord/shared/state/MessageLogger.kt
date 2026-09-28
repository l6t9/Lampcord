package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.database.MessageDao
import me.lampu.lampcord.shared.database.MessageEntity
import me.lampu.lampcord.shared.model.Message

class MessageLogger(
    private val json: Json,
    private val messageDao: MessageDao,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    fun logMessage(message: Message) {
        if (!me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled) return
        val settings = me.lampu.lampcord.shared.settings.Settings.shared
        if (settings.messageLoggerIgnoreBots && message.author?.bot == true) return
        if (settings.messageLoggerIgnoreSelf && message.author?.id == userStore.currentUser.value?.id) return
        
        scope.launch {
            try {
                val entity = MessageEntity(
                    id = message.id,
                    channelId = message.channel_id,
                    authorId = message.author?.id,
                    authorName = message.author?.username,
                    content = message.content,
                    timestamp = message.timestamp,
                    isDeleted = message.isDeleted,
                    oldContent = message.oldContent,
                    jsonPayload = json.encodeToString(message)
                )
                messageDao.insert(entity)
            } catch (e: Exception) {
                println("Error logging message: ${e.message}")
            }
        }
    }

    fun logUpdate(message: Message) {
        if (!me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled) return
        val settings = me.lampu.lampcord.shared.settings.Settings.shared
        if (settings.messageLoggerIgnoreBots && message.author?.bot == true) return
        if (settings.messageLoggerIgnoreSelf && message.author?.id == userStore.currentUser.value?.id) return

        scope.launch {
            try {
                messageDao.applyEdit(message.id, message.content, json.encodeToString(message))
            } catch (e: Exception) {
                println("Error logging update: ${e.message}")
            }
        }
    }

    fun logDelete(channelId: String, messageId: String) {
        if (!me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled) return
        scope.launch {
            try {
                messageDao.markDeleted(messageId)
            } catch (e: Exception) {
                println("Error logging delete: ${e.message}")
            }
        }
    }

    fun clearLoggedMessages(onComplete: () -> Unit = {}) {
        scope.launch {
            try {
                messageDao.clearAll()
                onComplete()
            } catch (e: Exception) {
                println("Error clearing logged messages: ${e.message}")
            }
        }
    }

    suspend fun getLoggedMessages(channelId: String): List<Message> {
        return try {
            val entities = messageDao.getMessagesForChannel(channelId)
            entities.map { entity ->
                val base = try {
                    json.decodeFromString<Message>(entity.jsonPayload)
                } catch (e: Exception) {
                    // Fallback reconstruction if JSON is corrupted
                    Message(
                        id = entity.id,
                        channel_id = entity.channelId,
                        content = entity.content,
                        timestamp = entity.timestamp
                    )
                }
                base.copy(
                    isDeleted = entity.isDeleted,
                    oldContent = entity.oldContent
                )
            }
        } catch (e: Exception) {
            emptyList()
        }
    }
}
