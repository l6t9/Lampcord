package me.lampu.lampcord

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DiscordPushPayloadTest {
    @Test
    fun parsesMessageCreatePayload() {
        val notification = mapOf(
            "type" to "MESSAGE_CREATE",
            "message_id" to "123",
            "channel_id" to "456",
            "channel_type" to "0",
            "channel_name" to "general",
            "guild_id" to "789",
            "guild_name" to "Lampcord",
            "user_id" to "10",
            "user_username" to "Nyx",
            "message_content" to "hello"
        ).toIncomingNotificationData()

        assertEquals("123", notification?.message?.id)
        assertEquals("hello", notification?.message?.content)
        assertEquals("#general", notification?.channelLabel)
        assertNull(mapOf("type" to "CALL_RING").toIncomingNotificationData())
    }
}
