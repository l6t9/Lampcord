package me.lampu.lampcord.shared.database

import androidx.room.*

@Dao
interface MessageDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(message: MessageEntity)

    @Update
    suspend fun update(message: MessageEntity)

    @Transaction
    suspend fun upsertLoggedMessage(message: MessageEntity) {
        val existing = getMessageById(message.id) ?: run {
            insert(message)
            return
        }
        if (existing.content == message.content &&
            existing.jsonPayload == message.jsonPayload &&
            existing.isDeleted == message.isDeleted
        ) return
        update(
            existing.copy(
                channelId = message.channelId,
                authorId = message.authorId,
                authorName = message.authorName,
                content = message.content,
                oldContent = if (message.content != existing.content) existing.content else existing.oldContent,
                timestamp = message.timestamp,
                isDeleted = message.isDeleted,
                jsonPayload = message.jsonPayload
            )
        )
    }

    @Query("SELECT * FROM logged_messages WHERE channelId = :channelId ORDER BY timestamp DESC LIMIT 500")
    suspend fun getMessagesForChannel(channelId: String): List<MessageEntity>

    @Query("SELECT * FROM logged_messages WHERE id = :id")
    suspend fun getMessageById(id: String): MessageEntity?

    @Transaction
    suspend fun applyEdit(
        id: String,
        content: String,
        jsonPayload: String
    ) {
        val existing = getMessageById(id) ?: return
        if (existing.content == content) return
        update(
            existing.copy(
                content = content,
                oldContent = existing.content,
                jsonPayload = jsonPayload
            )
        )
    }

    @Query("UPDATE logged_messages SET isDeleted = 1 WHERE id = :id")
    suspend fun markDeleted(id: String)

    @Query("DELETE FROM logged_messages WHERE timestamp < :cutoff")
    suspend fun deleteOlderThan(cutoff: String)

    @Query("DELETE FROM logged_messages")
    suspend fun clearAll()
}
