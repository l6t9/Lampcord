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
    private val scope: CoroutineScope
) {
    fun logMessage(message: Message) {
        if (!me.lampu.lampcord.shared.settings.Settings.shared.messageLoggerEnabled) return
        val settings = me.lampu.lampcord.shared.settings.Settings.shared
        if (settings.messageLoggerIgnoreBots && message.author?.bot == true) return
        
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
        scope.launch {
            try {
                val existing = messageDao.getMessageById(message.id)
                if (existing != null && existing.content != message.content) {
                    val entity = existing.copy(
                        content = message.content,
                        oldContent = existing.content,
                        jsonPayload = json.encodeToString(message)
                    )
                    messageDao.update(entity)
                }
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
