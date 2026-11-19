package me.lampu.lampcord

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import me.lampu.lampcord.shared.notifications.NotificationPushType

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
        assertTrue(notification?.isMention == true)
    }

    @Test
    fun dropsPayloadsWithoutAChannel() {
        assertNull(mapOf("type" to "CALL_RING", "message_id" to "1").toIncomingNotificationData())
    }

    @Test
    fun dropsUnknownPushTypes() {
        assertNull(mapOf("type" to "ACTIVITY_START", "channel_id" to "1", "message_id" to "1")
            .toIncomingNotificationData())
        assertNull(mapOf("type" to "CALL_CREATE", "channel_id" to "1", "message_id" to "1")
            .toIncomingNotificationData())
        assertNull(mapOf("type" to "RELATIONSHIP_ADD", "channel_id" to "1", "message_id" to "1")
            .toIncomingNotificationData())
    }

    @Test
    fun recognisesCallRingsWithoutAMessageId() {
        val notification = mapOf(
            "type" to "CALL_RING",
            "channel_id" to "456",
            "channel_type" to "2",
            "guild_id" to "789",
            "user_id" to "10",
            "user_username" to "Nyx"
        ).toIncomingNotificationData()

        assertEquals(NotificationPushType.CALL_RING, notification?.pushType)
        assertEquals("call:456", notification?.message?.id)
        assertEquals("456", notification?.message?.channel_id)
        assertEquals("Nyx", notification?.authorDisplayName)
    }

    @Test
    fun treatsMinusOneGuildIdAsADirectMessage() {
        val notification = mapOf(
            "type" to "MESSAGE_CREATE",
            "message_id" to "1",
            "channel_id" to "2",
            "channel_type" to "1",
            "guild_id" to "-1",
            "user_id" to "3",
            "user_username" to "Nyx"
        ).toIncomingNotificationData()

        assertTrue(notification?.isDm == true)
        assertNull(notification?.message?.guild_id)
    }

    @Test
    fun readsTheAckedChannelListFromJson() {
        val acks = mapOf("channel_ids" to "[\"100\",\"200\",\"-1\",\"300\"]").ackChannelIds()
        assertEquals(listOf("100", "200", "300"), acks)
        assertTrue(emptyMap<String, String>().ackChannelIds().isEmpty())
    }

    @Test
    fun ignoresAckedChannelListsThatAreNotJsonArrays() {
        assertTrue(mapOf("channel_ids" to "100,200,-1,300").ackChannelIds().isEmpty())
        assertTrue(mapOf("channel_ids" to "not-json").ackChannelIds().isEmpty())
    }

    @Test
    fun detectsCallRingEnd() {
        assertTrue(mapOf("type" to "CALL_RING_END", "channel_ids" to "[\"1\"]").isCallRingEnd())
        assertFalse(mapOf("type" to "MESSAGE_CREATE").isCallRingEnd())
    }

    @Test
    fun prefersGlobalNameOverUsername() {
        val notification = mapOf(
            "type" to "MESSAGE_CREATE",
            "message_id" to "1",
            "channel_id" to "2",
            "user_id" to "3",
            "user_username" to "nyx",
            "user_global_name" to "Nyx"
        ).toIncomingNotificationData()

        assertEquals("Nyx", notification?.authorDisplayName)
    }
}
