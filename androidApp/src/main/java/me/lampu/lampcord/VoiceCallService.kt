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
import me.lampu.lampcord.shared.state.VoiceStore
import org.koin.android.ext.android.getKoin

/** Keeps an explicitly started microphone call alive when the app is backgrounded. */
class VoiceCallService : Service() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val store get() = getKoin().get<VoiceStore>()
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onCreate() {
        super.onCreate()
        val manager = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= 26) manager.createNotificationChannel(
            NotificationChannel("voice_calls", "Ongoing voice calls", NotificationManager.IMPORTANCE_LOW)
        )
        startForeground(4201, notification())
        wakeLock = getSystemService(PowerManager::class.java).newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "Lampcord:VoiceCall").also { it.acquire() }
        scope.launch {
            snapshotFlow { Triple(store.activeChannel?.id, store.connection.phase, store.selfMuted) }.collect {
                manager.notify(4201, notification())
            }
        }
    }

    private fun notification(): Notification {
        val open = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        fun action(name: String) = PendingIntent.getService(this, name.hashCode(), Intent(this, VoiceCallService::class.java).setAction(name), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val description = when (store.connection.phase) {
            VoicePhase.SECURE -> "DAVE encrypted audio"
            VoicePhase.CONNECTED -> "Waiting for encrypted audio"
            else -> "Connecting to voice"
        }
        return NotificationCompat.Builder(this, "voice_calls")
            .setSmallIcon(R.drawable.ic_stat_notify)
            .setContentTitle(store.activeChannel?.name ?: "Lampcord voice call")
            .setContentText(description)
            .setCategory(NotificationCompat.CATEGORY_CALL)
            .setContentIntent(open).setOngoing(true).setOnlyAlertOnce(true)
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            .addAction(0, if (store.selfMuted) "Unmute" else "Mute", action("MUTE"))
            .addAction(0, "End call", action("END"))
            .build()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "MUTE" -> store.toggleVoiceMute()
            "END" -> { store.disconnectFromVoice(); stopSelf() }
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
}
