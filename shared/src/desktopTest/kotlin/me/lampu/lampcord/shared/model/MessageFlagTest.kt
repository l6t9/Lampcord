package me.lampu.lampcord.shared.model

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class MessageFlagTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun flagsExposeEphemeralAndSuppressedNotifications() {
        val plain = Message(id = "1", channel_id = "2")
        assertFalse(plain.isEphemeral)
        assertFalse(plain.suppressNotifications)

        val ephemeral = Message(id = "1", channel_id = "2", flags = 1 shl 6)
        assertTrue(ephemeral.isEphemeral)
        assertFalse(ephemeral.suppressNotifications)

        val silent = Message(id = "1", channel_id = "2", flags = 1 shl 12)
        assertTrue(silent.suppressNotifications)
        assertFalse(silent.isEphemeral)

        val both = Message(id = "1", channel_id = "2", flags = (1 shl 6) or (1 shl 12))
        assertTrue(both.isEphemeral)
        assertTrue(both.suppressNotifications)
    }

    @Test
    fun gatewayPayloadsDecodeEphemeralityReasonAndReactionType() {
        val message = json.decodeFromString<Message>(
            """{"id":"1","channel_id":"2","flags":64,"interaction_metadata":{"id":"i1","type":2,"ephemerality_reason":14}}"""
        )
        assertTrue(message.isEphemeral)
        assertEquals(14, message.interaction_metadata?.ephemerality_reason)

        val add = json.decodeFromString<MessageReactionAdd>(
            """{"user_id":"u1","channel_id":"2","message_id":"1","emoji":{"name":"wave"},"type":1}"""
        )
        val remove = json.decodeFromString<MessageReactionRemove>(
            """{"user_id":"u1","channel_id":"2","message_id":"1","emoji":{"name":"wave"},"type":1}"""
        )
        assertEquals(1, add.type)
        assertEquals(1, remove.type)
    }

    @Test
    fun mergeCopiesPresentMutableFieldsAndKeepsClientState() {
        val base = Message(id = "1", channel_id = "2", content = "old", isDeleted = true)
        val update = json.parseToJsonElement(
            """
            {
              "content": "new",
              "tts": true,
              "mention_everyone": true,
              "mentions": [{"id":"u1","username":"alice"}],
              "mention_roles": ["r1"],
              "reactions": [{"emoji":{"id":"e1","name":"wave"},"count":1,"count_details":{"burst":0,"normal":1},"me":false,"me_burst":false}],
              "interaction_metadata": {"id":"i1","type":2,"ephemerality_reason":14},
              "thread": {"id":"t1","type":11},
              "sticker_items": [],
              "flags": 64
            }
            """.trimIndent()
        ).jsonObject

        val merged = base.merge(update, json)

        assertEquals("new", merged.content)
        assertTrue(merged.tts)
        assertTrue(merged.mention_everyone)
        assertEquals("u1", merged.mentions.single().id)
        assertEquals(listOf("r1"), merged.mention_roles)
        assertEquals(1, merged.reactions?.size)
        assertNotNull(merged.interaction_metadata)
        assertEquals(14, merged.interaction_metadata.ephemerality_reason)
        assertEquals("t1", merged.thread?.id)
        assertNotNull(merged.sticker_items)
        assertTrue(merged.isEphemeral)
        assertTrue(merged.isDeleted)
    }

    @Test
    fun mergeKeepsFieldsOmittedFromPartialUpdate() {
        val reaction = MessageReaction(
            emoji = Emoji(id = "e1", name = "wave"),
            count = 1,
            count_details = ReactionCountDetails(burst = 0, normal = 1),
            me = false,
            me_burst = false
        )
        val base = Message(id = "1", channel_id = "2", content = "old", reactions = listOf(reaction))
        val update = json.parseToJsonElement("""{"content":"new"}""").jsonObject

        val merged = base.merge(update, json)

        assertEquals("new", merged.content)
        assertEquals(base.reactions, merged.reactions)
    }
}
