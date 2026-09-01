package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.playback.ffmpeg.AudioPlayer as FfmpegAudioPlayer
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
actual fun AudioPlayer(
    url: String,
    modifier: Modifier,
    autoPlay: Boolean,
) {
    var isLoading by remember(url) { mutableStateOf(true) }
    var isPlaying by remember(url) { mutableStateOf(false) }
    var currentTime by remember(url) { mutableLongStateOf(0L) }
    var duration by remember(url) { mutableLongStateOf(0L) }
    var loadError by remember(url) { mutableStateOf<String?>(null) }

    val player = remember(url) {
        FfmpegAudioPlayer {
            isPlaying = false
        }
    }

    DisposableEffect(player) {
        onDispose { player.close() }
    }

    LaunchedEffect(url, player) {
        isLoading = true
        loadError = null
        currentTime = 0L
        duration = 0L

        val loaded = withContext(Dispatchers.IO) {
            player.prepareFile(
                path = url,
                headers = mapOf("User-Agent" to "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
                startPaused = true,
            )
        }

        if (loaded) {
            duration = player.getDuration()
            isLoading = false
            if (autoPlay) {
                player.play()
                isPlaying = true
            }
        } else {
            isLoading = false
            loadError = "Unable to load this audio"
        }
    }

    LaunchedEffect(player, isLoading, loadError) {
        while (!isLoading && loadError == null) {
            currentTime = player.getCurrentPosition().coerceIn(0L, duration)
            if (!player.isPlaying() && duration > 0L && currentTime >= duration) {
                isPlaying = false
            }
            delay(100L)
        }
    }

    Column(modifier = modifier.padding(horizontal = 8.dp, vertical = 4.dp)) {
        when {
            isLoading -> {
                Box(
                    modifier = Modifier.fillMaxWidth().height(48.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator(modifier = Modifier.height(24.dp))
                }
            }
            loadError != null -> {
                Text(
                    text = loadError!!,
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.padding(vertical = 8.dp),
                )
            }
            else -> {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    IconButton(
                        onClick = {
                            if (isPlaying) {
                                player.pause()
                                isPlaying = false
                            } else {
                                if (duration > 0L && currentTime >= duration) {
                                    player.seekTo(0L)
                                    currentTime = 0L
                                }
                                player.resume()
                                isPlaying = true
                            }
                        },
                    ) {
                        Icon(
                            imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                            contentDescription = if (isPlaying) "Pause audio" else "Play audio",
                        )
                    }

                    Slider(
                        value = if (duration > 0L) currentTime.toFloat() / duration else 0f,
                        onValueChange = { fraction ->
                            currentTime = (fraction * duration).toLong()
                        },
                        onValueChangeFinished = {
                            player.seekTo(currentTime)
                        },
                        modifier = Modifier.weight(1f),
                    )

                    Text(
                        text = "${formatDuration(currentTime)} / ${formatDuration(duration)}",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

private fun formatDuration(durationMs: Long): String {
    val totalSeconds = (durationMs / 1000L).coerceAtLeast(0L)
    val minutes = totalSeconds / 60L
    val seconds = totalSeconds % 60L
    return "%d:%02d".format(minutes, seconds)
}
