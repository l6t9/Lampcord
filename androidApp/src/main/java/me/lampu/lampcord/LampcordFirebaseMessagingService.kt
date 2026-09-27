package me.lampu.lampcord

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import org.koin.core.context.GlobalContext

@Suppress("DEPRECATION", "OVERRIDE_DEPRECATION")
class LampcordFirebaseMessagingService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        GlobalContext.get().get<PushTokenRegistrar>().register(token)
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val type = message.data["type"]
        if (type == "MESSAGE_ACK" || type == "READ_STATE_UPDATE") {
            val channelId = message.data["channel_id"]
            if (channelId != null) {
                GlobalContext.get().get<MessageNotifier>().dismissChannelNotifications(channelId)
            }
            return
        }

        val data = message.data.toIncomingNotificationData() ?: return
        GlobalContext.get().get<MessageNotifier>().showMessageNotification(data)
    }
}
