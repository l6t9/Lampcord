package me.lampu.lampcord.shared.database

import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "logged_messages",
    indices = [Index("channelId")]
)
data class MessageEntity(
    @PrimaryKey val id: String,
    val channelId: String,
    val authorId: String?,
    val authorName: String?,
    val content: String,
    val timestamp: String,
    val isDeleted: Boolean = false,
    val oldContent: String? = null,
    val jsonPayload: String // Full original JSON for reconstruction
)
