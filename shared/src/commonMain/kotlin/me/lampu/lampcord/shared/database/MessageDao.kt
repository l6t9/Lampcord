package me.lampu.lampcord.shared.database

import androidx.room.*
import kotlinx.coroutines.flow.Flow

@Dao
interface MessageDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(message: MessageEntity)

    @Update
    suspend fun update(message: MessageEntity)

    @Query("SELECT * FROM logged_messages WHERE channelId = :channelId ORDER BY timestamp DESC LIMIT 500")
    suspend fun getMessagesForChannel(channelId: String): List<MessageEntity>

    @Query("SELECT * FROM logged_messages WHERE id = :id")
    suspend fun getMessageById(id: String): MessageEntity?

    /** Read-modify-write for an edit; the lookup and update must share a transaction. */
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

    @Query("DELETE FROM logged_messages")
    suspend fun clearAll()
}
