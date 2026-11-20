package me.lampu.lampcord

import android.graphics.Bitmap
import android.util.LruCache

data class NotificationMessage(
    val authorName: String,
    val authorAvatarUrl: String?,
    val text: String,
    val timestamp: Long,
    val isFromSelf: Boolean
)

data class ChannelNotificationMeta(
    val guildId: String?,
    val conversationTitle: String?,
    val conversationName: String,
    val isDirectMessage: Boolean,
    val lastMessageId: String
)

object NotificationMessageCache {
    private const val MAX_MESSAGES_PER_CHANNEL = 12
    private const val MAX_SEEN_MESSAGES = 100
    private const val MAX_CHANNELS = 100

    // Access order so the least recently used channel is evicted first.
    private val messagesByChannel =
        object : LinkedHashMap<String, MutableList<NotificationMessage>>(16, 0.75f, true) {}
    private val seenMessageIds = LinkedHashSet<String>()
    private val metaByChannel = mutableMapOf<String, ChannelNotificationMeta>()
    private val avatarCache = LruCache<String, Bitmap>(24)

    @Synchronized
    fun markMessageSeen(messageId: String): Boolean {
        if (!seenMessageIds.add(messageId)) return false
        if (seenMessageIds.size > MAX_SEEN_MESSAGES) seenMessageIds.remove(seenMessageIds.first())
        return true
    }

    @Synchronized
    fun addMessage(channelId: String, message: NotificationMessage) {
        val isNewChannel = !messagesByChannel.containsKey(channelId)
        val list = messagesByChannel.getOrPut(channelId) { mutableListOf() }
        list.add(message)
        while (list.size > MAX_MESSAGES_PER_CHANNEL) list.removeAt(0)
        if (isNewChannel && messagesByChannel.size > MAX_CHANNELS) {
            val oldest = messagesByChannel.keys.first()
            messagesByChannel.remove(oldest)
            metaByChannel.remove(oldest)
        }
    }

    @Synchronized
    fun setMeta(channelId: String, meta: ChannelNotificationMeta) {
        metaByChannel[channelId] = meta
    }

    @Synchronized
    fun updateLastMessageId(channelId: String, messageId: String) {
        metaByChannel[channelId]?.let { metaByChannel[channelId] = it.copy(lastMessageId = messageId) }
    }

    @Synchronized
    fun getMeta(channelId: String): ChannelNotificationMeta? = metaByChannel[channelId]

    @Synchronized
    fun getMessages(channelId: String): List<NotificationMessage> =
        messagesByChannel[channelId]?.toList() ?: emptyList()

    @Synchronized
    fun clearChannel(channelId: String) {
        messagesByChannel.remove(channelId)
        metaByChannel.remove(channelId)
    }

    @Synchronized
    fun activeChannelIds(): Set<String> = messagesByChannel.keys.toSet()

    fun cacheAvatar(url: String, bitmap: Bitmap) {
        avatarCache.put(url, bitmap)
    }

    fun getCachedAvatar(url: String?): Bitmap? {
        if (url == null) return null
        return avatarCache.get(url)
    }
}
