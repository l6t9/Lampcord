package me.lampu.lampcord.shared.database

import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertEquals

private class FakeMessageDao : MessageDao {
    var stored: MessageEntity? = null

    override suspend fun insert(message: MessageEntity) {
        stored = message
    }

    override suspend fun update(message: MessageEntity) {
        stored = message
    }

    override suspend fun getMessagesForChannel(channelId: String): List<MessageEntity> =
        listOfNotNull(stored?.takeIf { it.channelId == channelId })

    override suspend fun getMessageById(id: String): MessageEntity? =
        stored?.takeIf { it.id == id }

    override suspend fun markDeleted(id: String) {
        stored = stored?.takeIf { it.id == id }?.copy(isDeleted = true)
    }

    override suspend fun deleteOlderThan(cutoff: String) {
        stored = stored?.takeIf { it.timestamp >= cutoff }
    }

    override suspend fun clearAll() {
        stored = null
    }
}

class MessageDaoUpsertTest {
    private val original = MessageEntity(
        id = "1",
        channelId = "channel",
        authorId = "author",
        authorName = "author",
        content = "hello",
        timestamp = "2026-01-01T00:00:00Z",
        jsonPayload = "original"
    )

    @Test
    fun editKeepsThePreviousContentAsOldContent() = runBlocking {
        val dao = FakeMessageDao()
        dao.upsertLoggedMessage(original)
        dao.upsertLoggedMessage(original.copy(content = "edited", jsonPayload = "edited"))

        assertEquals("edited", dao.stored?.content)
        assertEquals("hello", dao.stored?.oldContent)
    }

    @Test
    fun repeatingTheSameMessageDoesNotEraseOldContent() = runBlocking {
        val dao = FakeMessageDao()
        val edited = original.copy(content = "edited", jsonPayload = "edited")
        dao.upsertLoggedMessage(original)
        dao.upsertLoggedMessage(edited)
        dao.upsertLoggedMessage(edited)

        assertEquals("edited", dao.stored?.content)
        assertEquals("hello", dao.stored?.oldContent)
    }
}
