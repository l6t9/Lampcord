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
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.Person
import androidx.core.content.pm.ShortcutInfoCompat
import androidx.core.content.pm.ShortcutManagerCompat
import androidx.core.graphics.drawable.IconCompat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.notifications.IncomingNotificationData
import java.net.URL

object NotificationHelper {
    const val CHANNEL_ID_MESSAGES = "lampcord_messages"
    const val GROUP_KEY_MESSAGES = "me.lampu.lampcord.MESSAGES"
    const val GROUP_SUMMARY_ID = -1
    const val EXTRA_CHANNEL_ID = "me.lampu.lampcord.extra.CHANNEL_ID"
    const val EXTRA_GUILD_ID = "me.lampu.lampcord.extra.GUILD_ID"
    const val EXTRA_IS_BUBBLE = "me.lampu.lampcord.extra.IS_BUBBLE"
    const val EXTRA_MESSAGE_ID = "me.lampu.lampcord.extra.MESSAGE_ID"
    const val KEY_TEXT_REPLY = "me.lampu.lampcord.key.TEXT_REPLY"

    private const val MAX_AVATAR_SIZE = 192

    fun ensureMessageChannel(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = context.getSystemService(NotificationManager::class.java)
            if (manager.getNotificationChannel(CHANNEL_ID_MESSAGES) == null) {
                val channel = NotificationChannel(
                    CHANNEL_ID_MESSAGES,
                    context.getString(R.string.notification_channel_messages),
                    NotificationManager.IMPORTANCE_HIGH
                ).apply {
                    description = context.getString(R.string.notification_channel_messages_description)
                    enableLights(true)
                }
                manager.createNotificationChannel(channel)
            }
        }
    }

    fun conversationChannelId(context: Context, channelId: String, conversationName: String): String {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return CHANNEL_ID_MESSAGES
        ensureMessageChannel(context)
        val conversationChannelId = "lampcord_conv_$channelId"
        val manager = context.getSystemService(NotificationManager::class.java)
        if (manager.getNotificationChannel(conversationChannelId) == null) {
            val channel = NotificationChannel(
                conversationChannelId,
                conversationName,
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                setConversationId(CHANNEL_ID_MESSAGES, channelId)
            }
            manager.createNotificationChannel(channel)
        }
        return conversationChannelId
    }

    fun updateGroupSummary(context: Context) {
        val manager = NotificationManagerCompat.from(context)
        val active = NotificationMessageCache.activeChannelIds()
            .mapNotNull { id -> NotificationMessageCache.getMeta(id)?.let { id to it } }

        if (active.isEmpty()) {
            manager.cancel(GROUP_SUMMARY_ID)
            return
        }

        val inboxStyle = androidx.core.app.NotificationCompat.InboxStyle()
            .setBigContentTitle(context.getString(R.string.notification_group_messages))
        active.take(6).forEach { (channelId, meta) ->
            val last = NotificationMessageCache.getMessages(channelId).lastOrNull()
            val line = if (last != null) "${meta.conversationName}: ${last.text}" else meta.conversationName
            inboxStyle.addLine(line)
        }

        val summaryIntent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
        }
        val contentIntent = PendingIntent.getActivity(
            context,
            GROUP_SUMMARY_ID,
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
            manager.notify(GROUP_SUMMARY_ID, summary)
        } catch (e: SecurityException) {
        }
    }

    fun notificationIdFor(channelId: String): Int = channelId.hashCode()

    suspend fun loadAvatar(url: String?): Bitmap? {
        if (url == null) return null
        NotificationMessageCache.getCachedAvatar(url)?.let { return it }
        return withContext(Dispatchers.IO) {
            try {
                val stream = URL(url).openStream()
                val bitmap = stream.use { BitmapFactory.decodeStream(it) }
                if (bitmap != null) {
                    val scaled = scaleToMax(bitmap, MAX_AVATAR_SIZE)
                    val circular = toCircular(scaled)
                    NotificationMessageCache.cacheAvatar(url, circular)
                    circular
                } else null
            } catch (e: Exception) {
                null
            }
        }
    }

    private fun scaleToMax(bitmap: Bitmap, maxSize: Int): Bitmap {
        val width = bitmap.width
        val height = bitmap.height
        if (width <= maxSize && height <= maxSize) return bitmap
        val scale = maxSize.toFloat() / maxOf(width, height)
        val newWidth = (width * scale).toInt().coerceAtLeast(1)
        val newHeight = (height * scale).toInt().coerceAtLeast(1)
        return Bitmap.createScaledBitmap(bitmap, newWidth, newHeight, true)
    }

    private fun toCircular(bitmap: Bitmap): Bitmap {
        val size = minOf(bitmap.width, bitmap.height)
        val output = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(output)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)
        canvas.drawCircle(size / 2f, size / 2f, size / 2f, paint)
        paint.xfermode = PorterDuffXfermode(PorterDuff.Mode.SRC_IN)
        val src = android.graphics.Rect(0, 0, bitmap.width, bitmap.height)
        val dst = android.graphics.Rect(0, 0, size, size)
        canvas.drawBitmap(bitmap, src, dst, paint)
        return output
    }

    fun buildContentIntent(context: Context, data: IncomingNotificationData): PendingIntent {
        val intent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra(EXTRA_CHANNEL_ID, data.message.channel_id)
            putExtra(EXTRA_GUILD_ID, data.message.guild_id)
        }
        return PendingIntent.getActivity(
            context,
            notificationIdFor(data.message.channel_id),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
    }

    fun buildReplyAction(context: Context, data: IncomingNotificationData): androidx.core.app.NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_REPLY
            putExtra(EXTRA_CHANNEL_ID, data.message.channel_id)
            putExtra(EXTRA_GUILD_ID, data.message.guild_id)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            notificationIdFor(data.message.channel_id),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        )
        val remoteInput = androidx.core.app.RemoteInput.Builder(KEY_TEXT_REPLY)
            .setLabel(context.getString(R.string.notification_action_reply))
            .build()
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_reply,
            context.getString(R.string.notification_action_reply),
            pendingIntent
        ).addRemoteInput(remoteInput).build()
    }

    fun buildMarkReadAction(context: Context, data: IncomingNotificationData): NotificationCompat.Action {
        val intent = Intent(context, NotificationActionReceiver::class.java).apply {
            action = NotificationActionReceiver.ACTION_MARK_READ
            putExtra(EXTRA_CHANNEL_ID, data.message.channel_id)
            putExtra(EXTRA_GUILD_ID, data.message.guild_id)
            putExtra(EXTRA_MESSAGE_ID, data.message.id)
        }
        val pendingIntent = PendingIntent.getBroadcast(
            context,
            notificationIdFor(data.message.channel_id) + 1,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Action.Builder(
            R.drawable.ic_action_check,
            context.getString(R.string.notification_action_mark_read),
            pendingIntent
        ).build()
    }

    fun buildPerson(name: String, avatar: Bitmap?): Person {
        val builder = Person.Builder().setName(name)
        if (avatar != null) builder.setIcon(IconCompat.createWithBitmap(avatar))
        return builder.build()
    }

    fun publishConversationShortcut(context: Context, data: IncomingNotificationData, avatar: Bitmap?) {
        val shortcutId = shortcutIdFor(data.message.channel_id)
        val bubbleIntent = Intent(context, MainActivity::class.java).apply {
            action = Intent.ACTION_VIEW
            putExtra(EXTRA_CHANNEL_ID, data.message.channel_id)
            putExtra(EXTRA_GUILD_ID, data.message.guild_id)
            putExtra(EXTRA_IS_BUBBLE, true)
        }
        val person = buildPerson(conversationName(data), avatar)
        val shortcut = ShortcutInfoCompat.Builder(context, shortcutId)
            .setShortLabel(conversationName(data))
            .setPerson(person)
            .setIntent(bubbleIntent)
            .setLongLived(true)
            .apply { if (avatar != null) setIcon(IconCompat.createWithBitmap(avatar)) }
            .build()
        ShortcutManagerCompat.pushDynamicShortcut(context, shortcut)
    }

    fun shortcutIdFor(channelId: String): String = "lampcord_conv_$channelId"

    private fun conversationName(data: IncomingNotificationData): String {
        return if (data.isDm) data.authorDisplayName else (data.channelLabel ?: data.authorDisplayName)
    }

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
