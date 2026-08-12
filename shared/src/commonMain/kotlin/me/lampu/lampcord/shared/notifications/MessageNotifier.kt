package me.lampu.lampcord.shared.notifications

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message

data class IncomingNotificationData(
    val message: Message,
    val channel: Channel?,
    val guild: Guild?,
    val authorDisplayName: String,
    val authorAvatarUrl: String?,
    val channelLabel: String?,
    val isDm: Boolean,
    val isMention: Boolean
)

interface MessageNotifier {
    val isInForeground: Boolean
    fun showMessageNotification(data: IncomingNotificationData)
    fun dismissChannelNotifications(channelId: String)
}
