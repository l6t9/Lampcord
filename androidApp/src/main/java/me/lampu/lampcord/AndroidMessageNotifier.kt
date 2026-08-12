package me.lampu.lampcord

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.graphics.drawable.IconCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.notifications.IncomingNotificationData
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import me.lampu.lampcord.shared.state.UserStore

class AndroidMessageNotifier(
    private val context: Context,
    private val notificationStore: NotificationStore,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) : MessageNotifier {

    override val isInForeground: Boolean
        get() = AppLifecycleTracker.isInForeground

    override fun showMessageNotification(data: IncomingNotificationData) {
        if (AppLifecycleTracker.isInForeground) {
            if (Settings.shared.showInAppNotifications) {
                notificationStore.show(data)
            }
            return
        }
        if (!Settings.shared.notificationsEnabled) return

        scope.launch { postSystemNotification(data) }
    }

    override fun dismissChannelNotifications(channelId: String) {
        NotificationHelper.dismissChannel(context, channelId)
        notificationStore.dismissChannel(channelId)
    }

    fun refreshChannelNotification(channelId: String) {
        scope.launch { refreshFromCache(channelId) }
    }

    private suspend fun postSystemNotification(data: IncomingNotificationData) {
        try {
            postSystemNotificationInternal(data)
        } catch (e: Exception) {
        }
    }

    private suspend fun postSystemNotificationInternal(data: IncomingNotificationData) {
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return

        NotificationHelper.ensureMessageChannel(context)

        val avatar = NotificationHelper.loadAvatar(data.authorAvatarUrl)
        val channelId = data.message.channel_id
        val timestamp = parseTimestamp(data.message.timestamp)

        NotificationMessageCache.addMessage(
            channelId,
            NotificationMessage(
                authorName = data.authorDisplayName,
                authorAvatarUrl = data.authorAvatarUrl,
                text = previewText(data),
                timestamp = timestamp,
                isFromSelf = false
            )
        )
        NotificationMessageCache.updateLastMessageId(channelId, data.message.id)

        val conversationName = if (data.isDm) data.authorDisplayName else (data.channelLabel ?: data.authorDisplayName)
        NotificationMessageCache.setMeta(
            channelId,
            ChannelNotificationMeta(
                guildId = data.message.guild_id,
                conversationTitle = data.channelLabel,
                conversationName = conversationName,
                lastMessageId = data.message.id
            )
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            NotificationHelper.publishConversationShortcut(context, data, avatar)
        }

        val notification = buildMessagingNotification(data, avatar, timestamp, channelId)
        try {
            manager.notify(NotificationHelper.notificationIdFor(channelId), notification)
        } catch (e: SecurityException) {
            return
        }

        NotificationHelper.updateGroupSummary(context)
    }

    private suspend fun refreshFromCache(channelId: String) {
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return
        val meta = NotificationMessageCache.getMeta(channelId) ?: return
        val messages = NotificationMessageCache.getMessages(channelId)
        if (messages.isEmpty()) return

        val last = messages.last()
        val avatar = NotificationHelper.loadAvatar(last.authorAvatarUrl)
        val selfUser = userStore.currentUser.value
        val selfPerson = Person.Builder()
            .setName(selfUser?.global_name ?: selfUser?.username ?: "You")
            .build()

        val conversationTitle = meta.conversationTitle
        val messagingStyle = NotificationCompat.MessagingStyle(selfPerson)
            .setGroupConversation(conversationTitle != null)
        conversationTitle?.let { messagingStyle.conversationTitle = it }

        messages.forEach { msg ->
            val person = if (msg.isFromSelf) selfPerson
            else NotificationHelper.buildPerson(msg.authorName, if (msg.authorAvatarUrl == last.authorAvatarUrl) avatar else null)
            messagingStyle.addMessage(msg.text, msg.timestamp, person)
        }

        val builder = NotificationCompat.Builder(context, conversationChannelId(context, channelId, meta))
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setStyle(messagingStyle)
            .setGroup(NotificationHelper.GROUP_KEY_MESSAGES)
            .setCategory(NotificationCompat.CATEGORY_MESSAGE)
            .setAutoCancel(true)
            .setShowWhen(true)
            .setWhen(messages.last().timestamp)
            .setNumber(messages.size)
            .setSilent(true)

        if (avatar != null) builder.setLargeIcon(avatar)

        val contentIntent = PendingIntent.getActivity(
            context,
            NotificationHelper.notificationIdFor(channelId),
            Intent(context, MainActivity::class.java).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
                putExtra(NotificationHelper.EXTRA_CHANNEL_ID, channelId)
                putExtra(NotificationHelper.EXTRA_GUILD_ID, meta.guildId)
            },
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        builder.setContentIntent(contentIntent)

        builder.addAction(buildReplyAction(context, channelId, meta.guildId))
        builder.addAction(buildMarkReadAction(context, channelId, meta.guildId, meta.lastMessageId))

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            val pendingBubble = PendingIntent.getActivity(
                context,
                NotificationHelper.notificationIdFor(channelId),
                Intent(context, BubbleActivity::class.java).apply {
                    putExtra(NotificationHelper.EXTRA_CHANNEL_ID, channelId)
                    putExtra(NotificationHelper.EXTRA_GUILD_ID, meta.guildId)
                },
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
            )
            val bubbleIcon = avatar?.let(IconCompat::createWithBitmap)
                ?: IconCompat.createWithResource(context, R.drawable.ic_stat_notify)
            val metadata = NotificationCompat.BubbleMetadata.Builder(pendingBubble, bubbleIcon)
                .setDesiredHeight(600)
                .setAutoExpandBubble(false)
                .setSuppressNotification(false)
                .build()
            builder.setBubbleMetadata(metadata)
            builder.setShortcutId(NotificationHelper.shortcutIdFor(channelId))
            val person = NotificationHelper.buildPerson(last.authorName, avatar)
            builder.addPerson(person)
            if (meta.conversationTitle == null) {
                builder.setLocusId(androidx.core.content.LocusIdCompat(channelId))
            }
        }

        try {
            manager.notify(NotificationHelper.notificationIdFor(channelId), builder.build())
        } catch (e: SecurityException) {
        }

        NotificationHelper.updateGroupSummary(context)
    }

    private fun previewText(data: IncomingNotificationData): String {
        if (!Settings.shared.showMessagePreview) return "New message"
        val message = data.message
        return when {
            message.content.isNotBlank() -> message.content
            message.attachments.isNotEmpty() -> "Sent an attachment"
            message.sticker_items?.isNotEmpty() == true -> "Sent a sticker"
            message.embeds.isNotEmpty() -> message.embeds.first().title ?: "Sent an embed"
            else -> "New message"
        }
    }

    private fun conversationChannelId(context: Context, channelId: String, meta: ChannelNotificationMeta): String {
        return NotificationHelper.conversationChannelId(context, channelId, meta.conversationName)
    }

    private fun buildMessagingNotification(
        data: IncomingNotificationData,
        avatar: android.graphics.Bitmap?,
        timestamp: Long,
        channelId: String
    ): Notification {
        val meta = NotificationMessageCache.getMeta(channelId)
        val conversationName = meta?.conversationName ?: if (data.isDm) data.authorDisplayName else (data.channelLabel ?: data.authorDisplayName)
        val channelForNotice = NotificationHelper.conversationChannelId(context, channelId, conversationName)

        val selfUser = userStore.currentUser.value
        val selfPerson = Person.Builder()
            .setName(selfUser?.global_name ?: selfUser?.username ?: "You")
            .build()

        val conversationTitle = data.channelLabel
        val messagingStyle = NotificationCompat.MessagingStyle(selfPerson)
            .setGroupConversation(conversationTitle != null)
        conversationTitle?.let { messagingStyle.conversationTitle = it }

        NotificationMessageCache.getMessages(channelId).forEach { msg ->
            val person = if (msg.isFromSelf) selfPerson
            else NotificationHelper.buildPerson(msg.authorName, if (msg.authorAvatarUrl == data.authorAvatarUrl) avatar else null)
            messagingStyle.addMessage(msg.text, msg.timestamp, person)
        }

        val contentIntent = NotificationHelper.buildContentIntent(context, data)

        val builder = NotificationCompat.Builder(context, channelForNotice)
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setStyle(messagingStyle)
            .setContentIntent(contentIntent)
            .setGroup(NotificationHelper.GROUP_KEY_MESSAGES)
            .setCategory(NotificationCompat.CATEGORY_MESSAGE)
            .setAutoCancel(true)
            .setShowWhen(true)
            .setWhen(timestamp)
            .setNumber(NotificationMessageCache.getMessages(channelId).size)
            .addAction(NotificationHelper.buildReplyAction(context, data))
            .addAction(NotificationHelper.buildMarkReadAction(context, data))

        if (avatar != null) {
            builder.setLargeIcon(avatar)
        }

        builder.setDefaults(Notification.DEFAULT_VIBRATE)
        if (Settings.shared.notificationSound) {
            builder.setDefaults(Notification.DEFAULT_SOUND or Notification.DEFAULT_VIBRATE)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            applyBubble(builder, data, avatar, channelId)
        }

        return builder.build()
    }

    private fun applyBubble(
        builder: NotificationCompat.Builder,
        data: IncomingNotificationData,
        avatar: android.graphics.Bitmap?,
        channelId: String
    ) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return

        val bubbleIntent = Intent(context, BubbleActivity::class.java).apply {
            putExtra(NotificationHelper.EXTRA_CHANNEL_ID, channelId)
            putExtra(NotificationHelper.EXTRA_GUILD_ID, data.message.guild_id)
        }
        val pendingIntent = PendingIntent.getActivity(
            context,
            NotificationHelper.notificationIdFor(channelId),
            bubbleIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )

        val icon = if (avatar != null) IconCompat.createWithBitmap(avatar) else IconCompat.createWithResource(context, R.drawable.ic_stat_notify)
        val metadata = NotificationCompat.BubbleMetadata.Builder(pendingIntent, icon)
            .setDesiredHeight(600)
            .setAutoExpandBubble(false)
            .setSuppressNotification(false)
            .build()

        val person = NotificationHelper.buildPerson(data.authorDisplayName, avatar)
        builder.setBubbleMetadata(metadata)
        builder.setShortcutId(NotificationHelper.shortcutIdFor(channelId))
        builder.addPerson(person)
        if (data.isDm || data.isMention) {
            builder.setLocusId(androidx.core.content.LocusIdCompat(channelId))
        }
    }

    private fun buildReplyAction(context: Context, channelId: String, guildId: String?): NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_REPLY
            putExtra(NotificationHelper.EXTRA_CHANNEL_ID, channelId)
            putExtra(NotificationHelper.EXTRA_GUILD_ID, guildId)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            NotificationHelper.notificationIdFor(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )
        val remoteInput = androidx.core.app.RemoteInput.Builder(NotificationHelper.KEY_TEXT_REPLY)
            .setLabel(context.getString(R.string.notification_action_reply))
            .build()
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_reply,
            context.getString(R.string.notification_action_reply),
            pendingIntent
        ).addRemoteInput(remoteInput).build()
    }

    private fun buildMarkReadAction(context: Context, channelId: String, guildId: String?, messageId: String): NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_MARK_READ
            putExtra(NotificationHelper.EXTRA_CHANNEL_ID, channelId)
            putExtra(NotificationHelper.EXTRA_GUILD_ID, guildId)
            putExtra(NotificationHelper.EXTRA_MESSAGE_ID, messageId)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            NotificationHelper.notificationIdFor(channelId) + 1,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_check,
            context.getString(R.string.notification_action_mark_read),
            pendingIntent
        ).build()
    }

    private fun parseTimestamp(iso: String): Long {
        return try {
            kotlin.time.Instant.parse(iso).toEpochMilliseconds()
        } catch (e: Exception) {
            System.currentTimeMillis()
        }
    }
}