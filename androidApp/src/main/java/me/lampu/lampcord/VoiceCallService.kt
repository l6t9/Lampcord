package me.lampu.lampcord

import android.app.*
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.compose.runtime.snapshotFlow
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import me.lampu.lampcord.shared.gateway.VoicePhase
import me.lampu.lampcord.shared.notifications.NotificationIds
import me.lampu.lampcord.shared.state.VoiceStore
import org.koin.android.ext.android.getKoin

class VoiceCallService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val store get() = getKoin().get<VoiceStore>()
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onCreate() {
        super.onCreate()
        val manager = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= 26) manager.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, getString(R.string.notification_channel_voice_calls), NotificationManager.IMPORTANCE_LOW)
        )
        startForeground(NotificationIds.VOICE_SESSION, notification())
        wakeLock = getSystemService(PowerManager::class.java).newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "Lampcord:VoiceCall").also { it.acquire() }
        scope.launch {
            snapshotFlow { Triple(store.activeChannel?.id, store.connection.phase, store.selfMuted) }.collect {
                manager.notify(NotificationIds.VOICE_SESSION, notification())
            }
        }
    }

    private fun notification(): Notification {
        val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        fun action(name: String) = PendingIntent.getService(this, name.hashCode(), Intent(this, VoiceCallService::class.java).setAction(name), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val description = when (store.connection.phase) {
            VoicePhase.SECURE -> getString(R.string.notification_voice_encrypted)
            VoicePhase.CONNECTED -> getString(R.string.notification_voice_waiting)
            else -> getString(R.string.notification_voice_connecting)
        }
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setColor(getColor(R.color.notification_accent))
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setContentTitle(store.activeChannel?.name ?: getString(R.string.notification_voice_title))
            .setContentText(description)
            .setCategory(NotificationCompat.CATEGORY_CALL)
            .setContentIntent(open).setOngoing(true).setOnlyAlertOnce(true)
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            .addAction(0, getString(if (store.selfMuted) R.string.notification_voice_unmute else R.string.notification_voice_mute), action(ACTION_MUTE))
            .addAction(0, getString(R.string.notification_voice_end), action(ACTION_END))
            .build()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_MUTE -> store.toggleVoiceMute()
            ACTION_END -> { store.disconnectFromVoice(); stopSelf() }
        }
        if (store.activeChannel == null) stopSelf()
        // Never resurrect a microphone session after process death without a fresh user action.
        return START_NOT_STICKY
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        store.disconnectFromVoice()
        stopSelf()
    }

    override fun onDestroy() {
        scope.cancel()
        wakeLock?.let { if (it.isHeld) it.release() }
        wakeLock = null
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private companion object {
        const val CHANNEL_ID = "lampcord_voice_calls"
        const val ACTION_MUTE = "me.lampu.lampcord.voice.MUTE"
        const val ACTION_END = "me.lampu.lampcord.voice.END"
    }
}
