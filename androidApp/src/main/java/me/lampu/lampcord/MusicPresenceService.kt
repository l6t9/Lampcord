package me.lampu.lampcord

import android.content.ComponentName
import android.media.MediaMetadata
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
import android.service.notification.NotificationListenerService
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.ActivityAssets
import me.lampu.lampcord.shared.model.ActivityTimestamps
import me.lampu.lampcord.shared.settings.Settings
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

class MusicPresenceService : NotificationListenerService(), KoinComponent {

    private val gatewayManager: GatewayManager by inject()
    private lateinit var mediaSessionManager: MediaSessionManager
    
    private val controllerCallbacks = mutableMapOf<MediaController, MediaController.Callback>()

    override fun onListenerConnected() {
        super.onListenerConnected()
        mediaSessionManager = getSystemService(MediaSessionManager::class.java)
        mediaSessionManager.addOnActiveSessionsChangedListener(
            { controllers -> updateControllers(controllers) },
            ComponentName(this, MusicPresenceService::class.java)
        )
        updateControllers(mediaSessionManager.getActiveSessions(ComponentName(this, MusicPresenceService::class.java)))
    }

    override fun onDestroy() {
        super.onDestroy()
        controllerCallbacks.forEach { (controller, callback) ->
            controller.unregisterCallback(callback)
        }
        controllerCallbacks.clear()
    }

    private fun updateControllers(controllers: List<MediaController>?) {
        controllerCallbacks.forEach { (controller, callback) ->
            controller.unregisterCallback(callback)
        }
        controllerCallbacks.clear()

        controllers?.forEach { controller ->
            val callback = object : MediaController.Callback() {
                override fun onMetadataChanged(metadata: MediaMetadata?) {
                    syncPresence()
                }

                override fun onPlaybackStateChanged(state: PlaybackState?) {
                    syncPresence()
                }
            }
            controller.registerCallback(callback)
            controllerCallbacks[controller] = callback
        }
        syncPresence()
    }

    private fun syncPresence() {
        if (Settings.shared.discordToken.isBlank() || !Settings.shared.musicPresenceEnabled) return

        val activeController = controllerCallbacks.keys.find { 
            it.playbackState?.state == PlaybackState.STATE_PLAYING 
        }

        if (activeController == null) {
            return
        }

        val metadata = activeController.metadata
        val title = metadata?.getString(MediaMetadata.METADATA_KEY_TITLE) ?: return
        val artist = metadata.getString(MediaMetadata.METADATA_KEY_ARTIST) ?: "Unknown Artist"
        val album = metadata.getString(MediaMetadata.METADATA_KEY_ALBUM)
        val duration = metadata.getLong(MediaMetadata.METADATA_KEY_DURATION)
        val position = activeController.playbackState?.position ?: 0L

        val appName = when (activeController.packageName) {
            "com.spotify.music" -> "Spotify"
            "com.google.android.apps.youtube.music" -> "YouTube Music"
            "com.apple.android.music" -> "Apple Music"
            else -> activeController.packageName ?: "Music"
        }

        val activity = Activity(
            name = appName,
            type = 2, // Listening
            details = title,
            state = artist,
            assets = ActivityAssets(
                large_text = album,
                large_image = if (appName == "Spotify") "spotify:${activeController.packageName}" else null // Dummy for now
            ),
            timestamps = if (duration > 0) {
                val now = System.currentTimeMillis()
                ActivityTimestamps(
                    start = now - position,
                    end = now - position + duration
                )
            } else null
        )

        gatewayManager.updatePresence(null, listOf(activity))
    }
}
