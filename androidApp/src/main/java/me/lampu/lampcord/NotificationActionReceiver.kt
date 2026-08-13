package me.lampu.lampcord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.app.RemoteInput
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.state.UserStore
import org.koin.core.context.GlobalContext

class NotificationActionReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        val koin = GlobalContext.get()
        val discordClient = koin.get<DiscordClient>()
        val channelId = intent.getStringExtra(NotificationHelper.EXTRA_CHANNEL_ID) ?: return
        val guildId = intent.getStringExtra(NotificationHelper.EXTRA_GUILD_ID)

        when (intent.action) {
            ACTION_REPLY -> {
                val results = RemoteInput.getResultsFromIntent(intent)
                val text = results?.getCharSequence(NotificationHelper.KEY_TEXT_REPLY)?.toString()?.trim()
                if (text.isNullOrBlank()) return
                val pendingResult = goAsync()
                CoroutineScope(Dispatchers.IO).launch {
                    try {
                        val sent = discordClient.sendMessage(channelId, text)
                        if (sent != null) {
                            val userStore = koin.get<UserStore>()
                            val self = userStore.currentUser.value
                            NotificationMessageCache.addMessage(
                                channelId,
                                NotificationMessage(
                                    authorName = self?.global_name ?: self?.username ?: "You",
                                    authorAvatarUrl = null,
                                    text = text,
                                    timestamp = System.currentTimeMillis(),
                                    isFromSelf = true
                                )
                            )
                            val notifier = try { koin.get<MessageNotifier>() as AndroidMessageNotifier } catch (e: Exception) { null }
                            notifier?.refreshChannelNotification(channelId)
                        }
                    } finally {
                        pendingResult.finish()
                    }
                }
            }

            ACTION_MARK_READ -> {
                val messageId = intent.getStringExtra(NotificationHelper.EXTRA_MESSAGE_ID) ?: return
                val pendingResult = goAsync()
                CoroutineScope(Dispatchers.IO).launch {
                    try {
                        discordClient.ackMessage(channelId, messageId)
                        try {
                            koin.get<MessageNotifier>().dismissChannelNotifications(channelId)
                        } catch (e: Exception) {
                            NotificationHelper.dismissChannel(context, channelId)
                        }
                    } finally {
                        pendingResult.finish()
                    }
                }
            }
        }
    }

    companion object {
        const val ACTION_REPLY = "me.lampu.lampcord.action.REPLY"
        const val ACTION_MARK_READ = "me.lampu.lampcord.action.MARK_READ"
    }
}