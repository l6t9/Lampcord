package me.lampu.lampcord

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.PorterDuff
import android.graphics.PorterDuffXfermode
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.app.RemoteInput
import androidx.core.content.pm.ShortcutInfoCompat
import androidx.core.content.pm.ShortcutManagerCompat
import androidx.core.graphics.drawable.IconCompat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.notifications.NotificationIds
import me.lampu.lampcord.shared.notifications.PendingIntentRequest
import me.lampu.lampcord.shared.utils.Logging
import java.net.URL

object NotificationHelper {
    const val CHANNEL_ID_MESSAGES = "lampcord_messages"
    const val CHANNEL_ID_DIRECT_MESSAGES = "lampcord_direct_messages"
    const val GROUP_KEY_MESSAGES = "me.lampu.lampcord.MESSAGES"
    const val EXTRA_CHANNEL_ID = "me.lampu.lampcord.extra.CHANNEL_ID"
    const val EXTRA_GUILD_ID = "me.lampu.lampcord.extra.GUILD_ID"
    const val EXTRA_IS_BUBBLE = "me.lampu.lampcord.extra.IS_BUBBLE"
    const val EXTRA_MESSAGE_ID = "me.lampu.lampcord.extra.MESSAGE_ID"
    const val KEY_TEXT_REPLY = "me.lampu.lampcord.key.TEXT_REPLY"

    private const val TAG = "notifications"
    private const val MAX_AVATAR_SIZE = 192

    private fun messageChannelId(isDirectMessage: Boolean): String =
        if (isDirectMessage) CHANNEL_ID_DIRECT_MESSAGES else CHANNEL_ID_MESSAGES

    fun ensureMessageChannels(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(NotificationManager::class.java)
        val soundUri = Uri.parse("android.resource://" + context.packageName + "/" + R.raw.notification)
        val audioAttributes = android.media.AudioAttributes.Builder()
            .setContentType(android.media.AudioAttributes.CONTENT_TYPE_SONIFICATION)
            .setUsage(android.media.AudioAttributes.USAGE_NOTIFICATION)
            .build()

        val channels = listOf(
            NotificationChannel(
                CHANNEL_ID_MESSAGES,
                context.getString(R.string.notification_channel_messages),
                NotificationManager.IMPORTANCE_HIGH
            ),
            NotificationChannel(
                CHANNEL_ID_DIRECT_MESSAGES,
                context.getString(R.string.notification_channel_direct_messages),
                NotificationManager.IMPORTANCE_HIGH
            )
        )
        channels.forEach { channel ->
            if (manager.getNotificationChannel(channel.id) != null) return@forEach
            channel.description =
                context.getString(R.string.notification_channel_messages_description)
            channel.enableLights(true)
            channel.setSound(soundUri, audioAttributes)
            manager.createNotificationChannel(channel)
        }
    }

    fun conversationChannelId(
        context: Context,
        channelId: String,
        conversationName: String,
        isDirectMessage: Boolean
    ): String {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return messageChannelId(isDirectMessage)
        ensureMessageChannels(context)
        val conversationChannelId = shortcutIdFor(channelId)
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager.getNotificationChannel(conversationChannelId) == null) {
            manager.createNotificationChannel(
                NotificationChannel(
                    conversationChannelId,
                    conversationName,
                    NotificationManager.IMPORTANCE_HIGH
                ).apply {
                    setSound(null, null)
                    setConversationId(messageChannelId(isDirectMessage), channelId)
                }
            )
        }
        return conversationChannelId
    }

    fun updateGroupSummary(context: Context) {
        val manager = NotificationManagerCompat.from(context)
        val active = NotificationMessageCache.activeChannelIds()
            .mapNotNull { id -> NotificationMessageCache.getMeta(id)?.let { id to it } }

        if (active.isEmpty()) {
            manager.cancel(NotificationIds.GROUP_SUMMARY)
            return
        }

        val inboxStyle = NotificationCompat.InboxStyle()
            .setBigContentTitle(context.getString(R.string.notification_group_messages))
        active.take(6).forEach { (channelId, meta) ->
            val last = NotificationMessageCache.getMessages(channelId).lastOrNull()
            inboxStyle.addLine(if (last != null) "${meta.conversationName}: ${last.text}" else meta.conversationName)
        }

        val summaryIntent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
        }
        val contentIntent = PendingIntent.getActivity(
            context,
            NotificationIds.GROUP_SUMMARY,
            summaryIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val summary = NotificationCompat.Builder(context, CHANNEL_ID_MESSAGES)
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setStyle(inboxStyle)
            .setGroup(GROUP_KEY_MESSAGES)
            .setGroupSummary(true)
            .setAutoCancel(true)
            .setSilent(true)
            .setContentIntent(contentIntent)
            .build()

        try {
            manager.notify(NotificationIds.GROUP_SUMMARY, summary)
        } catch (e: SecurityException) {
            Logging.w(TAG, "Not allowed to post the notification group summary", e)
        }
    }

    fun notificationIdFor(channelId: String): Int = NotificationIds.forMessage(channelId)

    suspend fun loadAvatar(url: String?): Bitmap? {
        if (url == null) return null
        NotificationMessageCache.getCachedAvatar(url)?.let { return it }
        return try {
            withContext(Dispatchers.IO) {
                val stream = URL(url).openStream()
                val bitmap = stream.use { BitmapFactory.decodeStream(it) }
                if (bitmap == null) {
                    null
                } else {
                    val circular = toCircular(scaleToMax(bitmap, MAX_AVATAR_SIZE))
                    NotificationMessageCache.cacheAvatar(url, circular)
                    circular
                }
            }
        } catch (e: Exception) {
            // A failed avatar must not cost the user the notification itself.
            Logging.w(TAG, "Unable to load the notification avatar", e)
            null
        }
    }

    private fun scaleToMax(bitmap: Bitmap, maxSize: Int): Bitmap {
        val width = bitmap.width
        val height = bitmap.height
        if (width <= maxSize && height <= maxSize) return bitmap
        val scale = maxSize.toFloat() / maxOf(width, height)
        return Bitmap.createScaledBitmap(
            bitmap,
            (width * scale).toInt().coerceAtLeast(1),
            (height * scale).toInt().coerceAtLeast(1),
            true
        )
    }

    private fun toCircular(bitmap: Bitmap): Bitmap {
        val size = minOf(bitmap.width, bitmap.height)
        val output = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(output)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)
        canvas.drawCircle(size / 2f, size / 2f, size / 2f, paint)
        paint.xfermode = PorterDuffXfermode(PorterDuff.Mode.SRC_IN)
        canvas.drawBitmap(
            bitmap,
            android.graphics.Rect(0, 0, bitmap.width, bitmap.height),
            android.graphics.Rect(0, 0, size, size),
            paint
        )
        return output
    }

    fun buildContentIntent(context: Context, channelId: String, guildId: String?): PendingIntent {
        val intent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra(EXTRA_CHANNEL_ID, channelId)
            putExtra(EXTRA_GUILD_ID, guildId)
        }
        return PendingIntent.getActivity(
            context,
            PendingIntentRequest.content(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
    }

    fun buildBubbleIntent(context: Context, channelId: String, guildId: String?): PendingIntent {
        val intent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra(EXTRA_CHANNEL_ID, channelId)
            putExtra(EXTRA_GUILD_ID, guildId)
            putExtra(EXTRA_IS_BUBBLE, true)
        }
        return PendingIntent.getActivity(
            context,
            PendingIntentRequest.bubble(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )
    }

    fun buildDeleteIntent(context: Context, channelId: String): PendingIntent {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_DISMISSED
            putExtra(EXTRA_CHANNEL_ID, channelId)
        }
        return PendingIntent.getBroadcast(
            context,
            PendingIntentRequest.delete(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
    }

    fun buildReplyAction(context: Context, channelId: String, guildId: String?): NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_REPLY
            putExtra(EXTRA_CHANNEL_ID, channelId)
            putExtra(EXTRA_GUILD_ID, guildId)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            PendingIntentRequest.reply(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )
        val remoteInput = RemoteInput.Builder(KEY_TEXT_REPLY)
            .setLabel(context.getString(R.string.notification_action_reply))
            .build()
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_reply,
            context.getString(R.string.notification_action_reply),
            pendingIntent
        ).addRemoteInput(remoteInput).setAllowGeneratedReplies(true).build()
    }

    fun buildMarkReadAction(context: Context, channelId: String, guildId: String?, messageId: String): NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_MARK_READ
            putExtra(EXTRA_CHANNEL_ID, channelId)
            putExtra(EXTRA_GUILD_ID, guildId)
            putExtra(EXTRA_MESSAGE_ID, messageId)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            PendingIntentRequest.markRead(channelId),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_check,
            context.getString(R.string.notification_action_mark_read),
            pendingIntent
        ).build()
    }

    fun buildPerson(name: String, avatar: Bitmap?, key: String? = null): Person {
        val builder = Person.Builder().setName(name)
        key?.let { builder.setKey(it) }
        if (avatar != null) builder.setIcon(IconCompat.createWithBitmap(avatar))
        return builder.build()
    }

    fun publishConversationShortcut(context: Context, channelId: String, guildId: String?, conversationName: String, avatar: Bitmap?) {
        val intent = Intent(context, MainActivity::class.java).apply {
            action = Intent.ACTION_VIEW
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
            putExtra(EXTRA_CHANNEL_ID, channelId)
            putExtra(EXTRA_GUILD_ID, guildId)
            putExtra(EXTRA_IS_BUBBLE, true)
        }
        val shortcut = ShortcutInfoCompat.Builder(context, shortcutIdFor(channelId))
            .setShortLabel(conversationName)
            .setPerson(buildPerson(conversationName, avatar))
            .setIntent(intent)
            .setLongLived(true)
            .setCategories(setOf("android.shortcut.conversation"))
            .apply { if (avatar != null) setIcon(IconCompat.createWithBitmap(avatar)) }
            .build()
        ShortcutManagerCompat.pushDynamicShortcut(context, shortcut)
    }

    fun shortcutIdFor(channelId: String): String = "lampcord_conv_$channelId"

    fun dismissChannel(context: Context, channelId: String) {
        NotificationManagerCompat.from(context).cancel(notificationIdFor(channelId))
        NotificationMessageCache.clearChannel(channelId)
        updateGroupSummary(context)
    }

    fun dismissAll(context: Context) {
        NotificationManagerCompat.from(context).cancelAll()
        NotificationMessageCache.activeChannelIds().forEach { NotificationMessageCache.clearChannel(it) }
    }
}
