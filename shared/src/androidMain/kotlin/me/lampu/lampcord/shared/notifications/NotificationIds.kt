package me.lampu.lampcord.shared.notifications

import android.content.Context

object NotificationIds {
    private const val MESSAGE_BASE = 0x1000_0000
    private const val MESSAGE_MASK = 0x0FFF_FFFF

    const val VOICE_SESSION = 4201

    const val INCOMING_CALL = 4202

    const val GROUP_SUMMARY = -1

    fun forMessage(channelId: String): Int = MESSAGE_BASE or (channelId.hashCode() and MESSAGE_MASK)
}

object PendingIntentRequest {
    private const val CHANNEL_MASK = 0x0FFF_FFF0

    private const val CONTENT = 0x1
    private const val BUBBLE = 0x2
    private const val REPLY = 0x3
    private const val MARK_READ = 0x4
    private const val DELETE = 0x5
    private const val INCOMING_CALL = 0x6

    fun content(channelId: String): Int = of(channelId, CONTENT)
    fun bubble(channelId: String): Int = of(channelId, BUBBLE)
    fun reply(channelId: String): Int = of(channelId, REPLY)
    fun markRead(channelId: String): Int = of(channelId, MARK_READ)
    fun delete(channelId: String): Int = of(channelId, DELETE)

    val incomingCall: Int = INCOMING_CALL

    private fun of(channelId: String, purpose: Int): Int =
        (channelId.hashCode() and CHANNEL_MASK) or purpose
}

fun isDirectMessageChannel(channelType: Int?, guildId: String?): Boolean =
    guildId == null || channelType == 1 || channelType == 3

internal fun Context.notificationManager(): android.app.NotificationManager =
    getSystemService(android.app.NotificationManager::class.java)