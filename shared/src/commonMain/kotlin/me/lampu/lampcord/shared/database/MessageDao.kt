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

    @Query("UPDATE logged_messages SET isDeleted = 1 WHERE id = :id")
    suspend fun markDeleted(id: String)
}
