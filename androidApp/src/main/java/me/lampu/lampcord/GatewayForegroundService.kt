package me.lampu.lampcord

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import me.lampu.lampcord.shared.settings.Settings

import android.os.UserManager
import me.lampu.lampcord.shared.state.SessionManager
import org.koin.core.context.GlobalContext

class GatewayForegroundService : Service() {

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> {
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
                return START_NOT_STICKY
            }
            else -> {
                startInForeground()
                connectGatewayIfNeeded()
            }
        }
        return START_STICKY
    }

    private fun connectGatewayIfNeeded() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            val userManager = getSystemService(Context.USER_SERVICE) as? UserManager
            if (userManager != null && !userManager.isUserUnlocked) return
        }

        try {
            val token = Settings.shared.discordToken
            if (token.isNotBlank()) {
                val koin = GlobalContext.getOrNull()
                if (koin != null) {
                    val sessionManager = koin.getOrNull<SessionManager>()
                    sessionManager?.connect(token)
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }


    private fun startInForeground() {
        ensureServiceChannel(this)
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(SERVICE_NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(SERVICE_NOTIFICATION_ID, notification)
        }
    }

    private fun buildNotification(): Notification {
        val contentIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val isSilent = Settings.shared.silentBackgroundService
        val builder = NotificationCompat.Builder(this, CHANNEL_ID_SERVICE)
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setContentTitle(getString(R.string.notification_service_title))
            .setContentText(getString(R.string.notification_service_text))
            .setOngoing(true)
            .setSilent(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setContentIntent(contentIntent)

        if (isSilent) {
            builder.setPriority(NotificationCompat.PRIORITY_MIN)
                .setVisibility(NotificationCompat.VISIBILITY_SECRET)
        } else {
            builder.setPriority(NotificationCompat.PRIORITY_LOW)
        }

        return builder.build()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        // Keep service running in background if notifications are enabled and user is logged in
        val token = Settings.shared.discordToken
        val notificationsEnabled = Settings.shared.notificationsEnabled
        if (token.isBlank() || !notificationsEnabled) {
            stopSelf()
        } else {
            // Re-trigger start to ensure service survives task dismissal
            start(this)
        }
    }

    companion object {
        const val CHANNEL_ID_SERVICE = "lampcord_service"
        const val SERVICE_NOTIFICATION_ID = 10001
        private const val ACTION_STOP = "me.lampu.lampcord.action.STOP_SERVICE"

        fun start(context: Context) {
            val intent = Intent(context, GatewayForegroundService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, GatewayForegroundService::class.java))
        }

        fun ensureServiceChannel(context: Context) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val manager = context.getSystemService(NotificationManager::class.java)
                val channel = NotificationChannel(
                    CHANNEL_ID_SERVICE,
                    context.getString(R.string.notification_channel_service),
                    NotificationManager.IMPORTANCE_LOW
                ).apply {
                    description = context.getString(R.string.notification_channel_service_description)
                    setShowBadge(false)
                    lockscreenVisibility = Notification.VISIBILITY_SECRET
                }
                manager.createNotificationChannel(channel)
            }
        }
    }
}

