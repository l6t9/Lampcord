package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.awt.SwingPanel
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.coroutines.delay
import uk.co.caprica.vlcj.factory.MediaPlayerFactory
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery
import uk.co.caprica.vlcj.player.base.MediaPlayer
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter
import uk.co.caprica.vlcj.player.component.EmbeddedMediaPlayerComponent
import java.awt.Component
import java.awt.Container
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent

// Shared factory to avoid "Failed to get a new native library instance" errors
// which often happen when creating/releasing factories too rapidly.
private object VlcPlayerManager {
    private var _factory: MediaPlayerFactory? = null
    
    val factory: MediaPlayerFactory?
        get() {
            if (_factory == null) {
                try {
                    NativeDiscovery().discover()
                    _factory = MediaPlayerFactory(
                        "--no-video-title-show",
                        "--avcodec-hw=any",
                        "--video-filter=sharpen",
                        "--network-caching=1000"
                    )
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }
            return _factory
        }
}

@Composable
actual fun VideoPlayer(
    url: String,
    modifier: Modifier
) {
    val factory = VlcPlayerManager.factory
    if (factory == null) {
        Box(modifier.background(Color.Black), contentAlignment = Alignment.Center) {
            Text("VLC native library not found", color = Color.White)
        }
        return
    }

    val mediaPlayerComponent = remember { EmbeddedMediaPlayerComponent(factory, null, null, null, null) }
    
    var isPlaying by remember { mutableStateOf(false) }
    var isHovered by remember { mutableStateOf(false) }
    var currentTime by remember { mutableStateOf(0L) }
    var duration by remember { mutableStateOf(0L) }
    var volume by remember { mutableStateOf(100f) }

    val mediaPlayer = remember { mediaPlayerComponent.mediaPlayer() }

    LaunchedEffect(Unit) {
        mediaPlayer.events().addMediaPlayerEventListener(object : MediaPlayerEventAdapter() {
            override fun playing(mediaPlayer: MediaPlayer?) { isPlaying = true }
            override fun paused(mediaPlayer: MediaPlayer?) { isPlaying = false }
            override fun stopped(mediaPlayer: MediaPlayer?) { isPlaying = false }
            override fun timeChanged(mediaPlayer: MediaPlayer?, newTime: Long) { currentTime = newTime }
            override fun lengthChanged(mediaPlayer: MediaPlayer?, newLength: Long) { duration = newLength }
        })

        val container = mediaPlayerComponent as Container
        val mouseListener = object : MouseAdapter() {
            override fun mouseEntered(e: MouseEvent?) { isHovered = true }
            override fun mouseExited(e: MouseEvent?) { isHovered = false }
        }
        if (container.componentCount > 0) {
            container.getComponent(0).addMouseListener(mouseListener)
        }
        
        mediaPlayer.media().play(url)
    }

    DisposableEffect(Unit) {
        onDispose {
            mediaPlayer.controls().stop()
            mediaPlayerComponent.release()
        }
    }

    Box(
        modifier = modifier.fillMaxSize()
    ) {
        SwingPanel(
            factory = { mediaPlayerComponent },
            modifier = Modifier.fillMaxSize()
        )

        // Overlay using Popups (these work across heavyweight boundaries)
        if (isHovered || !isPlaying) {
            Popup(
                alignment = Alignment.BottomCenter,
                offset = IntOffset(0, -32),
                properties = PopupProperties(focusable = false)
            ) {
                Surface(
                    color = Color.Black.copy(alpha = 0.8f),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier
                        .padding(horizontal = 32.dp)
                        .fillMaxWidth(0.9f)
                        .height(56.dp)
                ) {
                    Row(
                        modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        IconButton(onClick = { 
                            if (isPlaying) mediaPlayer.controls().pause() 
                            else mediaPlayer.controls().play() 
                        }) {
                            Icon(
                                imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                                contentDescription = "Play/Pause",
                                tint = Color.White,
                                modifier = Modifier.size(28.dp)
                            )
                        }
                        
                        Text(
                            text = "${formatDuration(currentTime)} / ${formatDuration(duration)}",
                            style = MaterialTheme.typography.labelSmall,
                            color = Color.White
                        )
                        
                        Slider(
                            value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                            onValueChange = { mediaPlayer.controls().setTime((it * duration).toLong()) },
                            modifier = Modifier.weight(1f).padding(horizontal = 16.dp),
                            colors = SliderDefaults.colors(
                                thumbColor = Color.White,
                                activeTrackColor = MaterialTheme.colorScheme.primary,
                                inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                            )
                        )
                        
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.VolumeUp,
                            contentDescription = "Volume",
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                        
                        Slider(
                            value = (volume / 200f).coerceIn(0f, 1f),
                            onValueChange = { 
                                volume = it * 200f
                                mediaPlayer.audio().setVolume(volume.toInt())
                            },
                            modifier = Modifier.width(80.dp).padding(horizontal = 8.dp),
                            colors = SliderDefaults.colors(
                                thumbColor = Color.White,
                                activeTrackColor = Color.White,
                                inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                            )
                        )
                        
                        IconButton(onClick = { mediaPlayer.fullScreen()?.toggle() }) {
                            Icon(
                                imageVector = Icons.Filled.Fullscreen,
                                contentDescription = "Fullscreen",
                                tint = Color.White,
                                modifier = Modifier.size(28.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}

private fun formatDuration(ms: Long): String {
    if (ms <= 0) return "0:00"
    val totalSeconds = ms / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "${minutes}:${seconds.toString().padStart(2, '0')}"
}
