package me.lampu.lampcord.shared.notifications

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message

enum class NotificationPushType {
    MESSAGE_CREATE,
    CALL_RING,
    RELATIONSHIP_ADD
}

data class IncomingNotificationData(
    val message: Message,
    val channel: Channel?,
    val guild: Guild?,
    val authorDisplayName: String,
    val authorAvatarUrl: String?,
    val channelLabel: String?,
    val isDm: Boolean,
    val isMention: Boolean,
    val pushType: NotificationPushType = NotificationPushType.MESSAGE_CREATE
)

interface MessageNotifier {
    val isInForeground: Boolean
    fun showMessageNotification(data: IncomingNotificationData)
    fun dismissChannelNotifications(channelId: String)

    fun dismissAllNotifications() {}
}
