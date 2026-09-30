package me.lampu.lampcord.shared.notifications

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.AndroidContextProvider
import me.lampu.lampcord.shared.utils.Logging

object IncomingCallNotifier {

    const val CHANNEL_ID = "lampcord_incoming_calls"

    private const val TAG = "notifications"
    private val callRingVibration = longArrayOf(100, 200, 300, 400, 500, 400, 300, 200, 400)

    private fun Context.postNotificationsAllowed(): Boolean {
        if (!NotificationManagerCompat.from(this).areNotificationsEnabled()) return false
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return true
        return checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED
    }

    private fun Context.ensureChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = notificationManager()
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Incoming voice calls",
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            enableVibration(true)
            vibrationPattern = callRingVibration
            enableLights(true)
            setShowBadge(false)
        }
        manager.createNotificationChannel(channel)
    }

    fun notifyIncomingCall(context: Context, channelId: String?) {
        if (channelId == null) {
            NotificationManagerCompat.from(context).cancel(NotificationIds.INCOMING_CALL)
            return
        }
        if (!context.postNotificationsAllowed()) return
        context.ensureChannel()

        val launch = context.packageManager.getLaunchIntentForPackage(context.packageName)
        if (launch == null) {
            Logging.w(TAG, "No launch intent for ${context.packageName}; not ringing")
            return
        }

        val pending = PendingIntent.getActivity(
            context,
            PendingIntentRequest.incomingCall,
            launch,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(notificationIcon(context))
            .setContentTitle("Incoming voice call")
            .setContentText("Open Lampcord to answer")
            .setCategory(NotificationCompat.CATEGORY_CALL)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setContentIntent(pending)
            .setAutoCancel(true)
            .setOngoing(true)
            .setSilent(!Settings.shared.incomingCallSound)
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            .build()

        try {
            NotificationManagerCompat.from(context).notify(NotificationIds.INCOMING_CALL, notification)
        } catch (e: SecurityException) {
            Logging.w(TAG, "Not allowed to post the incoming call notification", e)
        }
    }

    fun notifyIncomingCall(channelId: String?) =
        notifyIncomingCall(AndroidContextProvider.applicationContext, channelId)

    private fun notificationIcon(context: Context): Int {
        val id = context.resources.getIdentifier("ic_stat_notify", "drawable", context.packageName)
        if (id == 0) {
            Logging.w(TAG, "ic_stat_notify is missing; the call notification will not render")
        }
        return id
    }
}
