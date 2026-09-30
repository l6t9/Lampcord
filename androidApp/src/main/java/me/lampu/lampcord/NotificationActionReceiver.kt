package me.lampu.lampcord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.app.RemoteInput
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.utils.Logging
import org.koin.core.context.GlobalContext

class NotificationActionReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        val channelId = intent.getStringExtra(NotificationHelper.EXTRA_CHANNEL_ID) ?: return
        val guildId = intent.getStringExtra(NotificationHelper.EXTRA_GUILD_ID)

        when (intent.action) {
            ACTION_DISMISSED -> NotificationHelper.dismissChannel(context, channelId)

            ACTION_REPLY -> {
                val results = RemoteInput.getResultsFromIntent(intent)
                val text = results?.getCharSequence(NotificationHelper.KEY_TEXT_REPLY)?.toString()?.trim()
                if (text.isNullOrBlank()) return
                val koin = GlobalContext.get()
                val messageApi = koin.get<MessageApi>()
                val pendingResult = goAsync()
                CoroutineScope(Dispatchers.IO).launch {
                    try {
                        val sent = messageApi.sendMessage(channelId, text)
                        if (sent == null) return@launch
                        val self = koin.get<UserStore>().currentUser.value
                        NotificationMessageCache.addMessage(
                            channelId,
                            NotificationMessage(
                                authorName = self?.global_name ?: self?.username ?: "You",
                                authorAvatarUrl = self?.id,
                                text = text,
                                timestamp = System.currentTimeMillis(),
                                isFromSelf = true
                            )
                        )
                        (koin.get<MessageNotifier>() as? AndroidMessageNotifier)
                            ?.refreshChannelNotification(channelId)
                    } catch (e: Exception) {
                        Logging.e(TAG, "Reply from the notification failed", e)
                    } finally {
                        pendingResult.finish()
                    }
                }
            }

            ACTION_MARK_READ -> {
                val messageId = intent.getStringExtra(NotificationHelper.EXTRA_MESSAGE_ID) ?: return
                val koin = GlobalContext.get()
                val channelApi = koin.get<ChannelApi>()
                val pendingResult = goAsync()
                CoroutineScope(Dispatchers.IO).launch {
                    try {
                        channelApi.ackMessage(channelId, messageId)
                        (koin.get<MessageNotifier>() as? AndroidMessageNotifier)
                            ?.dismissChannelNotifications(channelId)
                            ?: NotificationHelper.dismissChannel(context, channelId)
                    } catch (e: Exception) {
                        Logging.e(TAG, "Mark as read from the notification failed", e)
                    } finally {
                        pendingResult.finish()
                    }
                }
            }
        }
    }

    companion object {
        private const val TAG = "notifications"
        const val ACTION_REPLY = "me.lampu.lampcord.action.REPLY"
        const val ACTION_MARK_READ = "me.lampu.lampcord.action.MARK_READ"
        const val ACTION_DISMISSED = "me.lampu.lampcord.action.DISMISSED"
    }
}
