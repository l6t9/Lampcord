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
        val data = message.data.toIncomingNotificationData() ?: return
        GlobalContext.get().get<MessageNotifier>().showMessageNotification(data)
    }
}
