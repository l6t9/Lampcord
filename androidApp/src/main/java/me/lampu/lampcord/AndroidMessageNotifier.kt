package me.lampu.lampcord

import android.app.Notification
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.content.LocusIdCompat
import androidx.core.graphics.drawable.IconCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.notifications.IncomingCallNotifier
import me.lampu.lampcord.shared.notifications.IncomingNotificationData
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.notifications.NotificationPushType
import me.lampu.lampcord.shared.notifications.isDirectMessageChannel
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.utils.Logging

class AndroidMessageNotifier(
    private val context: Context,
    private val notificationStore: NotificationStore,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) : MessageNotifier {

    override val isInForeground: Boolean
        get() = AppLifecycleTracker.isInForeground

    override fun showMessageNotification(data: IncomingNotificationData) {
        // A ringing call is a call notification whether it arrived over the gateway or as a
        // CALL_RING push. Routing it through the message path showed a text bubble for a call.
        if (data.pushType == NotificationPushType.CALL_RING) {
            IncomingCallNotifier.notifyIncomingCall(context, data.message.channel_id)
            if (AppLifecycleTracker.isInForeground) return
        }

        if (AppLifecycleTracker.isInForeground) {
            if (Settings.shared.showInAppNotifications) notificationStore.show(data)
            return
        }
        if (!Settings.shared.notificationsEnabled) return

        scope.launch { postSystemNotification(data) }
    }

    override fun dismissChannelNotifications(channelId: String) {
        NotificationHelper.dismissChannel(context, channelId)
        notificationStore.dismissChannel(channelId)
    }

    override fun dismissAllNotifications() {
        NotificationHelper.dismissAll(context)
    }

    fun refreshChannelNotification(channelId: String) {
        scope.launch { refreshFromCache(channelId) }
    }

    private suspend fun postSystemNotification(data: IncomingNotificationData) {
        try {
            postSystemNotificationInternal(data)
        } catch (e: Exception) {
            Logging.e(TAG, "Unable to post a notification for ${data.message.channel_id}", e)
        }
    }

    private suspend fun postSystemNotificationInternal(data: IncomingNotificationData) {
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return

        NotificationHelper.ensureMessageChannels(context)

        val channelId = data.message.channel_id
        if (!NotificationMessageCache.markMessageSeen(data.message.id)) return
        val avatar = NotificationHelper.loadAvatar(data.authorAvatarUrl)
        val timestamp = parseTimestamp(data.message.timestamp)

        val conversationName = conversationName(data)
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
        NotificationMessageCache.setMeta(
            channelId,
            ChannelNotificationMeta(
                guildId = data.message.guild_id,
                conversationTitle = data.channelLabel,
                conversationName = conversationName,
                isDirectMessage = isDirectMessageChannel(data.channel?.type, data.message.guild_id),
                lastMessageId = data.message.id
            )
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            NotificationHelper.publishConversationShortcut(
                context, channelId, data.message.guild_id, conversationName, avatar
            )
        }

        val notification = buildMessagingNotification(data, avatar, timestamp)
        try {
            manager.notify(NotificationHelper.notificationIdFor(channelId), notification)
        } catch (e: SecurityException) {
            Logging.w(TAG, "Not allowed to post a notification for $channelId", e)
            return
        }

        NotificationHelper.updateGroupSummary(context)
    }

    private suspend fun refreshFromCache(channelId: String) {
        try {
            refreshFromCacheInternal(channelId)
        } catch (e: Exception) {
            Logging.e(TAG, "Unable to refresh the notification for $channelId", e)
        }
    }

    private suspend fun refreshFromCacheInternal(channelId: String) {
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
            .setKey("me")
            .build()

        val channelForNotice = NotificationHelper.conversationChannelId(
            context, channelId, meta.conversationName, meta.isDirectMessage
        )

        val builder = baseBuilder(context, channelForNotice, meta, messages, avatar)
            // Re-rendering after a reply must not replay the alert.
            .setOnlyAlertOnce(true)
            .setSilent(true)
            .setContentIntent(NotificationHelper.buildContentIntent(context, channelId, meta.guildId))
            .setDeleteIntent(NotificationHelper.buildDeleteIntent(context, channelId))
            .addAction(NotificationHelper.buildReplyAction(context, channelId, meta.guildId))
            .addAction(NotificationHelper.buildMarkReadAction(context, channelId, meta.guildId, meta.lastMessageId))

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            val bubbleIcon = avatar?.let(IconCompat::createWithBitmap)
                ?: IconCompat.createWithResource(context, R.drawable.ic_stat_notify)
            builder.setBubbleMetadata(
                NotificationCompat.BubbleMetadata.Builder(
                    NotificationHelper.buildBubbleIntent(context, channelId, meta.guildId),
                    bubbleIcon
                )
                    .setDesiredHeight(600)
                    .setAutoExpandBubble(false)
                    .setSuppressNotification(false)
                    .build()
            )
            builder.setShortcutId(NotificationHelper.shortcutIdFor(channelId))
            builder.addPerson(NotificationHelper.buildPerson(last.authorName, avatar, key = last.authorAvatarUrl))
            if (meta.conversationTitle == null) {
                builder.setLocusId(LocusIdCompat(channelId))
            }
        }

        try {
            manager.notify(NotificationHelper.notificationIdFor(channelId), builder.build())
        } catch (e: SecurityException) {
            Logging.w(TAG, "Not allowed to refresh the notification for $channelId", e)
        }

        NotificationHelper.updateGroupSummary(context)
    }

    private fun previewText(data: IncomingNotificationData): String {
        if (!Settings.shared.showMessagePreview) return context.getString(R.string.notification_preview_hidden)
        val message = data.message
        return when {
            message.content.isNotBlank() -> message.content
            message.attachments.isNotEmpty() -> context.getString(R.string.notification_preview_attachment)
            message.sticker_items?.isNotEmpty() == true -> context.getString(R.string.notification_preview_sticker)
            message.embeds.isNotEmpty() -> context.getString(R.string.notification_preview_embed)
            else -> context.getString(R.string.notification_preview_message)
        }
    }

    private fun conversationName(data: IncomingNotificationData): String =
        if (data.isDm) data.authorDisplayName else (data.channelLabel ?: data.authorDisplayName)

    private fun buildMessagingNotification(
        data: IncomingNotificationData,
        avatar: android.graphics.Bitmap?,
        timestamp: Long
    ): Notification {
        val channelId = data.message.channel_id
        val meta = NotificationMessageCache.getMeta(channelId)
        val conversationName = meta?.conversationName ?: conversationName(data)
        val isDirectMessage = meta?.isDirectMessage
            ?: isDirectMessageChannel(data.channel?.type, data.message.guild_id)
        val channelForNotice = NotificationHelper.conversationChannelId(
            context, channelId, conversationName, isDirectMessage
        )

        val messages = NotificationMessageCache.getMessages(channelId)
        val builder = baseBuilder(
            context,
            channelForNotice,
            meta,
            messages,
            avatar,
            sortKey = timestamp
        )
            // A conversation that already has messages must not alert again for each one.
            .setOnlyAlertOnce(messages.size > 1)
            .setContentIntent(NotificationHelper.buildContentIntent(context, channelId, data.message.guild_id))
            .setDeleteIntent(NotificationHelper.buildDeleteIntent(context, channelId))
            .addAction(NotificationHelper.buildReplyAction(context, channelId, data.message.guild_id))
            .addAction(NotificationHelper.buildMarkReadAction(context, channelId, data.message.guild_id, data.message.id))

        // The channel owns sound and vibration from API 26 onwards, so setDefaults is ignored
        // there; the user-facing toggle is the channel, not the notification.
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O && Settings.shared.notificationSound) {
            builder.setDefaults(Notification.DEFAULT_SOUND)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            applyBubble(builder, data, avatar, channelId)
        }

        return builder.build()
    }

    private fun baseBuilder(
        context: Context,
        channelId: String,
        meta: ChannelNotificationMeta?,
        messages: List<NotificationMessage>,
        avatar: android.graphics.Bitmap?,
        sortKey: Long = messages.lastOrNull()?.timestamp ?: System.currentTimeMillis()
    ): NotificationCompat.Builder {
        val selfUser = userStore.currentUser.value
        val selfPerson = Person.Builder()
            .setName(selfUser?.global_name ?: selfUser?.username ?: "You")
            .setKey("me")
            .build()

        val conversationTitle = meta?.conversationTitle
        val messagingStyle = NotificationCompat.MessagingStyle(selfPerson)
            .setGroupConversation(conversationTitle != null)
        conversationTitle?.let { messagingStyle.conversationTitle = it }

        // MessagingStyle silently drops messages that share a timestamp with the one before
        // them, so the thread is walked forward strictly in time.
        var previous = 0L
        messages.forEach { msg ->
            val at = if (msg.timestamp <= previous) previous + 1 else msg.timestamp
            previous = at
            val person = if (msg.isFromSelf) selfPerson
            else NotificationHelper.buildPerson(
                msg.authorName,
                if (msg.authorAvatarUrl == messages.last().authorAvatarUrl) avatar else null,
                key = msg.authorAvatarUrl
            )
            messagingStyle.addMessage(msg.text, at, person)
        }

        return NotificationCompat.Builder(context, channelId)
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setColor(context.getColor(R.color.notification_accent))
            .setStyle(messagingStyle)
            .setGroup(NotificationHelper.GROUP_KEY_MESSAGES)
            .setCategory(NotificationCompat.CATEGORY_MESSAGE)
            .setAutoCancel(true)
            .setShowWhen(true)
            .setWhen(sortKey)
            .setNumber(messages.size)
            .apply { if (avatar != null) setLargeIcon(avatar) }
    }

    private fun applyBubble(
        builder: NotificationCompat.Builder,
        data: IncomingNotificationData,
        avatar: android.graphics.Bitmap?,
        channelId: String
    ) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return

        val icon = avatar?.let(IconCompat::createWithBitmap)
            ?: IconCompat.createWithResource(context, R.drawable.ic_stat_notify)
        builder.setBubbleMetadata(
            NotificationCompat.BubbleMetadata.Builder(
                NotificationHelper.buildBubbleIntent(context, channelId, data.message.guild_id),
                icon
            )
                .setDesiredHeight(600)
                .setAutoExpandBubble(false)
                .setSuppressNotification(false)
                .build()
        )
        builder.setShortcutId(NotificationHelper.shortcutIdFor(channelId))
        builder.addPerson(
            NotificationHelper.buildPerson(data.authorDisplayName, avatar, key = data.message.author?.id)
        )
        if (data.isDm || data.isMention) {
            builder.setLocusId(LocusIdCompat(channelId))
        }
    }

    private fun parseTimestamp(iso: String): Long {
        return try {
            kotlin.time.Instant.parse(iso).toEpochMilliseconds()
        } catch (e: Exception) {
            System.currentTimeMillis()
        }
    }

    private companion object {
        const val TAG = "notifications"
    }
}
