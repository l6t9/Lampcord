package me.lampu.lampcord

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import me.lampu.lampcord.shared.utils.Logging
import org.koin.core.context.GlobalContext

@Suppress("DEPRECATION", "OVERRIDE_DEPRECATION")
class LampcordFirebaseMessagingService : FirebaseMessagingService() {

    override fun onNewToken(token: String) {
        runCatching { GlobalContext.get().get<PushTokenRegistrar>().register(token) }
            .onFailure { Logging.e(TAG, "Unable to register a new push token", it) }
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val notifier = runCatching { GlobalContext.get().get<MessageNotifier>() }
            .onFailure { Logging.e(TAG, "Push received before the app finished starting", it) }
            .getOrNull() ?: return

        message.data.ackChannelIds().forEach { channelId ->
            runCatching { notifier.dismissChannelNotifications(channelId) }
                .onFailure { Logging.e(TAG, "Unable to dismiss acked channel $channelId", it) }
        }

        val data = message.data.toIncomingNotificationData() ?: return
        runCatching { notifier.showMessageNotification(data) }
            .onFailure { Logging.e(TAG, "Unable to show a notification for ${data.message.channel_id}", it) }
    }

    private companion object {
        const val TAG = "push"
    }
}
