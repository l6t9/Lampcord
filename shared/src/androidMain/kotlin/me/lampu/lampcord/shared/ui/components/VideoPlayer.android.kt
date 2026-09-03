package me.lampu.lampcord.shared.ui.components

import androidx.annotation.OptIn
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(UnstableApi::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun VideoPlayer(
    url: String,
    modifier: Modifier,
    loop: Boolean,
    showControls: Boolean,
    title: String?,
    subtitle: String?,
    compact: Boolean,
    autoPlay: Boolean,
    showSeekBar: Boolean,
    onFullscreenClick: (() -> Unit)?,
) {
    val context = LocalContext.current
    val exoPlayer = remember(url, loop, autoPlay) {
        ExoPlayer.Builder(context).build().apply {
            setMediaItem(MediaItem.fromUri(url))
            repeatMode = if (loop) ExoPlayer.REPEAT_MODE_ALL else ExoPlayer.REPEAT_MODE_OFF
            playWhenReady = autoPlay
            prepare()
        }
    }
    var isPlaying by remember(url) { mutableStateOf(autoPlay) }
    var isBuffering by remember(url) { mutableStateOf(true) }
    var playerError by remember(url) { mutableStateOf(false) }
    var positionMs by remember(url) { mutableLongStateOf(0L) }
    var durationMs by remember(url) { mutableLongStateOf(0L) }
    var bufferedPositionMs by remember(url) { mutableLongStateOf(0L) }

    var areControlsVisible by remember { mutableStateOf(showControls) }
    var isDraggingSlider by remember { mutableStateOf(false) }

    fun retry() {
        playerError = false
        isBuffering = true
        exoPlayer.seekTo(0L)
        exoPlayer.prepare()
        exoPlayer.playWhenReady = autoPlay
    }

    DisposableEffect(exoPlayer) {
        val listener = object : Player.Listener {
            override fun onIsPlayingChanged(playing: Boolean) {
                isPlaying = playing
            }

            override fun onPlaybackStateChanged(playbackState: Int) {
                isBuffering = playbackState == Player.STATE_BUFFERING || playbackState == Player.STATE_IDLE
                if (playbackState == Player.STATE_READY) {
                    durationMs = exoPlayer.duration.coerceAtLeast(0L)
                }
                if (playbackState == Player.STATE_ENDED && !loop) {
                    exoPlayer.seekTo(0L)
                    exoPlayer.pause()
                    isPlaying = false
                }
            }

            override fun onPlayerError(error: PlaybackException) {
                isBuffering = false
                isPlaying = false
                playerError = true
            }
        }
        exoPlayer.addListener(listener)
        onDispose {
            exoPlayer.removeListener(listener)
            exoPlayer.release()
        }
    }

    LaunchedEffect(exoPlayer, isDraggingSlider) {
        while (true) {
            withFrameMillis {
                if (!isDraggingSlider) {
                    positionMs = exoPlayer.currentPosition.coerceAtLeast(0L)
                }
                bufferedPositionMs = exoPlayer.bufferedPosition.coerceAtLeast(0L)
                durationMs = exoPlayer.duration.takeIf { it >= 0L } ?: durationMs
                isPlaying = exoPlayer.isPlaying
                isBuffering = (exoPlayer.playbackState == Player.STATE_BUFFERING ||
                    exoPlayer.playbackState == Player.STATE_IDLE)
            }
        }
    }

    LaunchedEffect(isPlaying, areControlsVisible, isDraggingSlider) {
        if (isPlaying && areControlsVisible && !isDraggingSlider) {
            delay(3000)
            areControlsVisible = false
        }
    }

    Box(
        modifier = modifier
            .background(Color.Black)
            .pointerInput(Unit) {
                detectTapGestures {
                    areControlsVisible = !areControlsVisible
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = {
                PlayerView(context).apply {
                    player = exoPlayer
                    useController = false
                    resizeMode = AspectRatioFrameLayout.RESIZE_MODE_FIT
                    setShutterBackgroundColor(android.graphics.Color.TRANSPARENT)
                    setBackgroundColor(android.graphics.Color.TRANSPARENT)
                    setKeepContentOnPlayerReset(true)
                    layoutParams = android.widget.FrameLayout.LayoutParams(
                        android.view.ViewGroup.LayoutParams.MATCH_PARENT,
                        android.view.ViewGroup.LayoutParams.MATCH_PARENT,
                    )
                }
            },
            update = { view ->
                view.player = exoPlayer
                view.useController = false
            }
        )

        if (isBuffering) {
            ContainedLoadingIndicator(
                indicatorColor = Color.White,
                modifier = Modifier.size(if (compact) 32.dp else 48.dp),
            )
        } else if (playerError) {
            Surface(
                onClick = ::retry,
                shape = RoundedCornerShape(12.dp),
                color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
                contentColor = MaterialTheme.colorScheme.onSurface,
            ) {
                Text(
                    text = "Unable to play video\nTap to retry",
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.padding(horizontal = 20.dp, vertical = 14.dp),
                )
            }
        }

        if (showControls && areControlsVisible && !isBuffering && !playerError && !isPlaying) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.24f)),
                contentAlignment = Alignment.Center,
            ) {
                Surface(
                    onClick = { exoPlayer.play() },
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.primaryContainer,
                    contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.size(if (compact) 56.dp else 72.dp),
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = "Play video",
                            modifier = Modifier.size(if (compact) 28.dp else 40.dp),
                        )
                    }
                }
            }
        }

        if (showControls && areControlsVisible && !isBuffering && !playerError) {
            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .background(
                        Brush.verticalGradient(
                            listOf(Color.Transparent, Color.Black.copy(alpha = 0.88f)),
                        ),
                    )
                    .padding(horizontal = if (compact) 10.dp else 18.dp, vertical = 10.dp),
            ) {
                if (!compact && (title != null || subtitle != null)) {
                    Text(
                        text = title ?: subtitle.orEmpty(),
                        style = MaterialTheme.typography.labelMedium,
                        color = Color.White,
                        maxLines = 1,
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Surface(
                        onClick = {
                            if (exoPlayer.isPlaying) exoPlayer.pause() else exoPlayer.play()
                        },
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.primaryContainer,
                        contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(if (compact) 32.dp else 40.dp),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Icon(
                                imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                                contentDescription = if (isPlaying) "Pause video" else "Play video",
                                modifier = Modifier.size(if (compact) 18.dp else 22.dp),
                            )
                        }
                    }

                    Text(
                        text = "${formatVideoTime(positionMs)} / ${formatVideoTime(durationMs)}",
                        style = MaterialTheme.typography.labelSmall,
                        fontSize = 10.sp,
                        color = Color.White.copy(alpha = 0.92f),
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                    )
                    if (showSeekBar) {
                        val progress = remember(positionMs, durationMs) {
                            if (durationMs > 0L) (positionMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f) else 0f
                        }
                        val bufferedProgress = remember(bufferedPositionMs, durationMs) {
                            if (durationMs > 0L) (bufferedPositionMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f) else 0f
                        }

                        VideoSlider(
                            value = progress,
                            onValueChange = { 
                                isDraggingSlider = true
                                positionMs = (it * durationMs).toLong() 
                            },
                            onValueChangeFinished = {
                                isDraggingSlider = false
                                exoPlayer.seekTo(positionMs)
                            },
                            bufferedFraction = bufferedProgress,
                            modifier = Modifier.weight(1f),
                        )
                    }

                    if (onFullscreenClick != null) {
                        Surface(
                            onClick = onFullscreenClick,
                            shape = RoundedCornerShape(8.dp),
                            color = Color.Black.copy(alpha = 0.62f),
                            contentColor = Color.White,
                            modifier = Modifier.size(if (compact) 48.dp else 52.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    imageVector = Icons.Filled.Fullscreen,
                                    contentDescription = "Fullscreen",
                                    modifier = Modifier.size(if (compact) 24.dp else 28.dp),
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

private fun formatVideoTime(milliseconds: Long): String {
    val totalSeconds = (milliseconds / 1_000L).coerceAtLeast(0L)
    return "%d:%02d".format(totalSeconds / 60L, totalSeconds % 60L)
}
