package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.database.MessageDao
import me.lampu.lampcord.shared.database.MessageEntity
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlin.time.Instant

private const val LOG_RETENTION_MILLIS = 90L * 24 * 60 * 60 * 1000
private const val PRUNE_INTERVAL_MILLIS = 6L * 60 * 60 * 1000

private sealed interface LogWrite {
    data class Insert(val entity: MessageEntity) : LogWrite
    data class Edit(val id: String, val content: String, val jsonPayload: String) : LogWrite
    data class Delete(val messageId: String) : LogWrite
    data class Clear(val onComplete: () -> Unit) : LogWrite
    data class Prune(val at: Long) : LogWrite
}

class MessageLogger(
    private val json: Json,
    private val messageDao: MessageDao,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    // One writer keeps inserts, edits and deletes in gateway order.
    private val writes = Channel<LogWrite>(Channel.UNLIMITED)

    init {
        writes.trySend(LogWrite.Prune(getCurrentTimeMillis()))
        scope.launch {
            var lastPruneAt = 0L
            for (write in writes) {
                when (write) {
                    is LogWrite.Insert -> runCatching { messageDao.upsertLoggedMessage(write.entity) }
                        .onFailure { Logging.e("MessageLogger", "Error logging message", it) }

                    is LogWrite.Edit -> runCatching {
                        messageDao.applyEdit(write.id, write.content, write.jsonPayload)
                    }.onFailure { Logging.e("MessageLogger", "Error logging update", it) }

                    is LogWrite.Delete -> runCatching { messageDao.markDeleted(write.messageId) }
                        .onFailure { Logging.e("MessageLogger", "Error logging delete", it) }

                    is LogWrite.Clear -> runCatching {
                        messageDao.clearAll()
                        write.onComplete()
                    }.onFailure { Logging.e("MessageLogger", "Error clearing logged messages", it) }

                    is LogWrite.Prune -> runCatching {
                        messageDao.deleteOlderThan(retentionCutoff(write.at))
                    }.onFailure { Logging.e("MessageLogger", "Error pruning logged messages", it) }
                        .also { lastPruneAt = write.at }
                }

                val now = getCurrentTimeMillis()
                if (write !is LogWrite.Prune && now - lastPruneAt >= PRUNE_INTERVAL_MILLIS) {
                    runCatching { messageDao.deleteOlderThan(retentionCutoff(now)) }
                        .onFailure { Logging.e("MessageLogger", "Error pruning logged messages", it) }
                    lastPruneAt = now
                }
            }
        }
    }

    fun logMessage(message: Message) {
        val settings = Settings.shared
        if (!settings.messageLoggerEnabled) return
        if (message.isEphemeral) return
        if (settings.messageLoggerIgnoreBots && message.author?.bot == true) return
        if (settings.messageLoggerIgnoreSelf && message.author?.id == userStore.currentUser.value?.id) return

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
        submit(LogWrite.Insert(entity))
    }

    fun logUpdate(message: Message) {
        val settings = Settings.shared
        if (!settings.messageLoggerEnabled) return
        if (message.isEphemeral) return
        if (settings.messageLoggerIgnoreBots && message.author?.bot == true) return
        if (settings.messageLoggerIgnoreSelf && message.author?.id == userStore.currentUser.value?.id) return

        submit(LogWrite.Edit(message.id, message.content, json.encodeToString(message)))
    }

    fun logDelete(messageId: String) {
        if (!Settings.shared.messageLoggerEnabled) return
        submit(LogWrite.Delete(messageId))
    }

    fun clearLoggedMessages(onComplete: () -> Unit = {}) {
        submit(LogWrite.Clear(onComplete))
    }

    suspend fun getLoggedMessages(channelId: String): List<Message> {
        return withContext(Dispatchers.Default) {
            try {
                val entities = messageDao.getMessagesForChannel(channelId)
                entities.map { entity ->
                    val base = try {
                        json.decodeFromString<Message>(entity.jsonPayload)
                    } catch (e: Exception) {
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
                Logging.e("MessageLogger", "Error reading logged messages", e)
                emptyList()
            }
        }
    }

    private fun submit(write: LogWrite) {
        if (!writes.trySend(write).isSuccess) {
            Logging.e("MessageLogger", "Message logger is closed; write was dropped")
        }
    }

    private fun retentionCutoff(now: Long): String =
        Instant.fromEpochMilliseconds(now - LOG_RETENTION_MILLIS).toString()
}
