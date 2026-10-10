@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameMillis
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asComposeImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.contentDescription
import me.lampu.lampcord.shared.playback.ffmpeg.AudioRenderer
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegFrameGrabber
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegLogCallback
import me.lampu.lampcord.shared.playback.ffmpeg.Frame
import me.lampu.lampcord.shared.playback.ffmpeg.FrameGrabber
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.settings.Settings
import org.bytedeco.ffmpeg.global.avutil.AV_SAMPLE_FMT_S16
import org.jetbrains.skia.*
import org.jetbrains.skia.Image as SkiaImage
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.ShortBuffer
import javax.sound.sampled.AudioFormat
import javax.swing.SwingUtilities
import kotlin.math.roundToInt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.ui.kit.clickableCursor

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
    onFullscreenClick: (() -> Unit)?
) {
    var videoFrame by remember { mutableStateOf<ImageBitmap?>(null) }
    var isPlaying by remember { mutableStateOf(false) }
    var currentTime by remember { mutableLongStateOf(0L) }
    var duration by remember { mutableLongStateOf(0L) }
    var hasEnded by remember { mutableStateOf(false) }
    var volume by remember { mutableFloatStateOf(Settings.shared.videoVolume.coerceIn(0f, 1f)) }
    var isResolving by remember { mutableStateOf(true) }
    var loadError by remember { mutableStateOf<String?>(null) }
    var loadAttempt by remember { mutableIntStateOf(0) }
    var isVolumeMenuOpen by remember { mutableStateOf(false) }

    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()

    val player = remember(url, loop, autoPlay) {
        DesktopVideoPlayer(
            repeat = loop,
            onFrame = { videoFrame = it },
            onEnded = {
                if (!loop) {
                    isPlaying = false
                    hasEnded = true
                }
            },
            onError = { message ->
                if (loadError == null) {
                    loadError = message
                    isPlaying = false
                }
            }
        )
    }

    LaunchedEffect(url, loadAttempt) {
        isResolving = true
        loadError = null
        videoFrame = null
        currentTime = 0L
        duration = 0L
        isPlaying = false
        hasEnded = false
        val success = withContext(Dispatchers.IO) {
            player.load(url, headers = mapOf("User-Agent" to "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")) {
                currentTime
            }
        }
        isResolving = false
        if (success) {
            duration = player.getDuration()
            player.setVolume(volume)
            player.play(startPaused = !autoPlay)
            isPlaying = autoPlay
        } else {
            loadError = "Unable to load this video"
        }
    }

    LaunchedEffect(isPlaying) {
        if (isPlaying) player.resume() else player.pause()
    }

    LaunchedEffect(volume) {
        player.setVolume(volume)
    }

    val togglePlayback = {
        if (!isPlaying && hasEnded) {
            player.seekTo(0L)
            currentTime = 0L
            hasEnded = false
        }
        isPlaying = !isPlaying
    }

    DisposableEffect(player) {
        onDispose {
            player.close()
        }
    }

    var isDraggingSlider by remember { mutableStateOf(false) }

    LaunchedEffect(isPlaying, isDraggingSlider) {
        if (!isPlaying || isDraggingSlider) return@LaunchedEffect
        while (true) {
            currentTime = player.getCurrentPosition()
            delay(32)
        }
    }

    Box(
        modifier = modifier
            .hoverable(interactionSource),
        contentAlignment = Alignment.Center
    ) {
        DesktopVideoSurface(videoFrame, togglePlayback)

        if (isResolving) {
            ContainedLoadingIndicator(indicatorColor = Color.White)
        } else if (loadError != null) {
            Surface(
                onClick = { loadAttempt++ },
                shape = RoundedCornerShape(8.dp),
                color = Color.Black.copy(alpha = 0.8f),
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "${loadError}\nClick to retry",
                    color = Color.White,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp)
                )
            }
        }

        if (compact && showControls) {
            val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
            Row(
                modifier = Modifier.align(Alignment.TopEnd).padding(8.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                AnimatedVisibility(
                    visible = isHovered,
                    enter = if (Settings.shared.reduceMotion) EnterTransition.None else fadeIn(),
                    exit = if (Settings.shared.reduceMotion) ExitTransition.None else fadeOut()
                ) {
                    VideoActionButton(
                        onClick = { uriHandler.openUri(url) },
                        modifier = Modifier.size(32.dp),
                        contentDescription = "Download"
                    ) {
                        Icon(Icons.Filled.Download, null, modifier = Modifier.size(20.dp))
                    }
                }
                if (onFullscreenClick != null) {
                    VideoActionButton(
                        onClick = onFullscreenClick,
                        modifier = Modifier.size(32.dp),
                        contentDescription = "Fullscreen"
                    ) {
                        Icon(Icons.Filled.Fullscreen, null, modifier = Modifier.size(20.dp))
                    }
                }
            }
        }

        if (showControls) {
            HoverablePlayControls(
                isPlaying = isPlaying,
                compact = compact,
                onTogglePlay = togglePlayback
            )

            AnimatedVisibility(
                visible = isHovered || !isPlaying || isVolumeMenuOpen,
                enter = if (Settings.shared.reduceMotion) EnterTransition.None else fadeIn(),
                exit = if (Settings.shared.reduceMotion) ExitTransition.None else fadeOut(),
                modifier = Modifier.align(Alignment.BottomCenter)
            ) {
                BottomVideoControls(
                    url = url,
                    currentTime = currentTime,
                    duration = duration,
                    isPlaying = isPlaying,
                    volume = volume,
                    title = title,
                    subtitle = subtitle,
                    compact = compact,
                    showSeekBar = !compact || videoFrame?.let { it.height > it.width } != true,
                    onTogglePlay = togglePlayback,
                    onSeek = { 
                        isDraggingSlider = true
                        currentTime = (it * duration).toLong()
                    },
                    onSeekFinished = {
                        isDraggingSlider = false
                        player.seekTo(currentTime)
                    },
                    onVolumeChange = {
                        volume = it
                        Settings.shared.videoVolume = it
                    },
                    onVolumeMenuOpenChanged = { isVolumeMenuOpen = it },
                    onFullscreenClick = onFullscreenClick
                )
            }
        }
    }
}

@Composable
private fun DesktopVideoSurface(
    frame: ImageBitmap?,
    onClick: () -> Unit
) {
    if (frame != null) {
        Image(
            bitmap = frame,
            contentDescription = null,
            contentScale = ContentScale.Fit,
            modifier = Modifier
                .fillMaxSize()
                .clickableCursor(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                    onClick()
                }
        )
    }
}

@Composable
private fun HoverablePlayControls(
    isPlaying: Boolean,
    compact: Boolean,
    onTogglePlay: () -> Unit
) {
    AnimatedVisibility(
        visible = !isPlaying,
        enter = if (Settings.shared.reduceMotion) EnterTransition.None else fadeIn(),
        exit = if (Settings.shared.reduceMotion) ExitTransition.None else fadeOut(),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.4f)),
            contentAlignment = Alignment.Center
        ) {
            Surface(
                onClick = onTogglePlay,
                shape = CircleShape,
                color = MaterialTheme.colorScheme.primaryContainer,
                modifier = Modifier.size(if (compact) 48.dp else 72.dp)
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Filled.PlayArrow,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(if (compact) 24.dp else 40.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun BottomVideoControls(
    url: String,
    currentTime: Long,
    duration: Long,
    isPlaying: Boolean,
    volume: Float,
    title: String?,
    subtitle: String?,
    compact: Boolean,
    showSeekBar: Boolean,
    onTogglePlay: () -> Unit,
    onSeek: (Float) -> Unit,
    onSeekFinished: () -> Unit,
    onVolumeChange: (Float) -> Unit,
    onVolumeMenuOpenChanged: (Boolean) -> Unit,
    onFullscreenClick: (() -> Unit)?
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .background(
                Brush.verticalGradient(
                    listOf(Color.Transparent, Color.Black.copy(alpha = 0.7f), Color.Black.copy(alpha = 0.9f))
                )
            )
            .padding(top = 48.dp) // Gradient starting area
    ) {
        Column(
            modifier = Modifier
                .padding(horizontal = if (compact) 12.dp else 24.dp, vertical = if (compact) 8.dp else 20.dp)
        ) {
            if (compact) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    val playPauseIcon = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow
                    Surface(
                        onClick = onTogglePlay,
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.primaryContainer,
                        modifier = Modifier.size(32.dp)
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Icon(
                                imageVector = playPauseIcon,
                                contentDescription = "Play/Pause",
                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                    }

                    Text(
                        text = "${formatDuration(currentTime)} / ${formatDuration(duration, unknownWhenZero = true)}",
                        style = MaterialTheme.typography.labelSmall,
                        fontSize = if (compact) 10.sp else 11.sp,
                        color = Color.White.copy(alpha = 0.9f),
                        maxLines = 1,
                        softWrap = false
                    )

                    if (showSeekBar) {
                        VideoSlider(
                            value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                            onValueChange = onSeek,
                            onValueChangeFinished = onSeekFinished,
                            bufferedFraction = 0f,
                            modifier = Modifier.weight(1f)
                        )
                    }

                    VideoConnectedButtonGroup(
                        url = url,
                        volume = volume,
                        onVolumeChange = onVolumeChange,
                        onVolumeMenuOpenChanged = onVolumeMenuOpenChanged,
                        onFullscreenClick = onFullscreenClick,
                        compact = true
                    )
                }
            } else {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Text(
                        text = formatDuration(currentTime),
                        style = MaterialTheme.typography.labelMedium,
                        color = Color.White.copy(alpha = 0.9f)
                    )
                    VideoSlider(
                        value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                        onValueChange = onSeek,
                        onValueChangeFinished = onSeekFinished,
                        bufferedFraction = 0f,
                        modifier = Modifier.weight(1f)
                    )
                    Text(
                        text = formatDuration(duration, unknownWhenZero = true),
                        style = MaterialTheme.typography.labelMedium,
                        color = Color.White.copy(alpha = 0.9f),
                        textAlign = TextAlign.End
                    )
                }

                Spacer(Modifier.height(8.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Row(
                        modifier = Modifier.weight(1f),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        if (title != null) {
                            Column(modifier = Modifier.weight(1f, fill = false)) {
                                Text(
                                    text = title,
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold,
                                    color = Color.White,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    modifier = if (!Settings.shared.marqueeEnabled) {
                                        Modifier
                                    } else {
                                        Modifier.basicMarquee(
                                            iterations = if (getPlatformName() == "windows") 1 else Int.MAX_VALUE
                                        )
                                    }
                                )
                                if (subtitle != null) {
                                    Text(
                                        text = subtitle,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Color.White.copy(alpha = 0.7f),
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                        }
                    }

                    VideoConnectedButtonGroup(
                        url = url,
                        volume = volume,
                        onVolumeChange = onVolumeChange,
                        onVolumeMenuOpenChanged = onVolumeMenuOpenChanged,
                        onFullscreenClick = onFullscreenClick,
                        compact = false
                    )
                }
            }
        }
    }
}

@Composable
private fun VideoConnectedButtonGroup(
    url: String,
    volume: Float,
    onVolumeChange: (Float) -> Unit,
    onVolumeMenuOpenChanged: (Boolean) -> Unit,
    onFullscreenClick: (() -> Unit)?,
    compact: Boolean
) {
    val groupHeight = 40.dp
    val iconSize = if (compact) 22.dp else 20.dp
    val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
    var showVolumeMenu by remember { mutableStateOf(false) }

    LaunchedEffect(showVolumeMenu) {
        onVolumeMenuOpenChanged(showVolumeMenu)
    }

    DisposableEffect(Unit) {
        onDispose { onVolumeMenuOpenChanged(false) }
    }

    val volumeIcon = when {
        volume > 0.66f -> Icons.AutoMirrored.Filled.VolumeUp
        volume > 0.33f -> Icons.AutoMirrored.Filled.VolumeDown
        volume > 0f -> Icons.AutoMirrored.Filled.VolumeMute
        else -> Icons.AutoMirrored.Filled.VolumeOff
    }

    Row(
        modifier = Modifier.height(groupHeight),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box {
            VideoActionButton(
                onClick = { showVolumeMenu = !showVolumeMenu },
                modifier = Modifier.size(if (compact) 40.dp else groupHeight),
                contentDescription = "Volume"
            ) {
                Icon(volumeIcon, null, modifier = Modifier.size(iconSize))
            }
            DropdownMenu(
                expanded = showVolumeMenu,
                onDismissRequest = { showVolumeMenu = false }
            ) {
                Column(modifier = Modifier.width(160.dp).padding(12.dp)) {
                    Text("Volume", style = MaterialTheme.typography.labelMedium)
                    Slider(
                        value = volume,
                        onValueChange = onVolumeChange,
                        valueRange = 0f..1f
                    )
                }
            }
        }

        if (!compact) {
            VideoActionButton(
                onClick = { setClipboardText(url) },
                modifier = Modifier.size(groupHeight),
                contentDescription = "Copy URL"
            ) {
                Icon(Icons.Filled.ContentCopy, null, modifier = Modifier.size(iconSize - 2.dp))
            }
        }

        if (!compact) {
            VideoActionButton(
                onClick = { uriHandler.openUri(url) },
                modifier = Modifier.size(groupHeight),
                contentDescription = "Download"
            ) {
                Icon(Icons.Filled.Download, null, modifier = Modifier.size(iconSize))
            }
        }

        if (onFullscreenClick != null && !compact) {
            VideoActionButton(
                onClick = onFullscreenClick,
                modifier = Modifier.size(groupHeight),
                contentDescription = "Fullscreen"
            ) {
                Icon(Icons.Filled.Fullscreen, null, modifier = Modifier.size(iconSize + 4.dp))
            }
        }
    }
}

@Composable
private fun VideoActionButton(
    onClick: () -> Unit,
    modifier: Modifier,
    contentDescription: String,
    content: @Composable () -> Unit,
) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(6.dp),
        color = Color.Black.copy(alpha = 0.62f),
        contentColor = Color.White,
        modifier = modifier.semantics { this.contentDescription = contentDescription }
    ) {
        Box(contentAlignment = Alignment.Center) {
            content()
        }
    }
}

private class DesktopVideoPlayer(
    private val repeat: Boolean,
    private val onFrame: (ImageBitmap) -> Unit,
    private val onEnded: () -> Unit = {},
    private val onError: (String) -> Unit = {},
    private val onSeekLanded: ((Long) -> Unit)? = null,
) {
    private var grabber: FFmpegFrameGrabber? = null
    private var audioRenderer: AudioRenderer? = null

    @Volatile
    private var isRunning = false

    @Volatile
    private var isPaused = false

    @Volatile
    private var closeRequested = true

    @Volatile
    private var pendingSeekMs: Long? = null

    @Volatile
    private var discardAudioUntilMs: Long? = null

    @Volatile
    private var positionProvider: () -> Long = { 0L }

    @Volatile
    private var volume = 1.0f

    private var decodeThread: Thread? = null

    private var targetWidth = 0
    private var targetHeight = 0
    
    @Volatile
    private var loadJobId = 0L

    private val lifecycleLock = Any()

    @Volatile
    private var videoClockStartNanos = 0L

    @Volatile
    private var videoClockStartMs = 0L

    @Volatile
    private var pausedAtNanos = 0L

    @Volatile
    private var lastVideoPositionMs = 0L

    @Volatile
    private var lastVideoTimestampMs = -1L

    @Volatile
    private var videoTimestampOffsetMs: Long? = null

    val hasOwnAudio: Boolean
        get() = audioRenderer != null

    fun load(
        url: String,
        headers: Map<String, String> = emptyMap(),
        positionProvider: () -> Long,
    ): Boolean {
        close()
        FFmpegLogCallback.set()
        this.positionProvider = positionProvider
        val currentLoadJobId = System.nanoTime()
        loadJobId = currentLoadJobId
        var lastFailure: Throwable? = null

        val sourceUrls = videoSourceCandidates(url)
        for (sourceUrl in sourceUrls) {
            // Some mobile videos contain an audio stream that Java Sound cannot open on the current Windows device.
            for (videoOnly in listOf(false, true)) {
                if (loadJobId != currentLoadJobId) return false
                var g: FFmpegFrameGrabber? = null
                try {
                    val candidateGrabber = FFmpegFrameGrabber(sourceUrl)
                    g = candidateGrabber
                    candidateGrabber.apply {
                        if (videoOnly) audioStream = -2
                        if (sourceUrl.startsWith("http://") || sourceUrl.startsWith("https://")) {
                            options["reconnect"] = "1"
                            options["reconnect_streamed"] = "1"
                            options["reconnect_delay_max"] = "5"
                            options["timeout"] = "30000000"
                            options["rw_timeout"] = "30000000"
                            options["user_agent"] =
                                headers.entries.firstOrNull { it.key.equals("User-Agent", ignoreCase = true) }?.value
                                    ?: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
                            headers
                                .filterKeys { !it.equals("User-Agent", ignoreCase = true) }
                                .takeIf { it.isNotEmpty() }
                                ?.entries
                                ?.joinToString(separator = "\r\n", postfix = "\r\n") { "${it.key}: ${it.value}" }
                                ?.let { options["headers"] = it }

                            // MP4 files recorded by phones often put moov and stream metadata late in the file. Give FFmpeg a large enough probe window without forcing a full download before the first frame.
                            options["probesize"] = "50000000"
                            options["analyzeduration"] = "30000000"
                        }
                        sampleFormat = AV_SAMPLE_FMT_S16
                        imageMode = FrameGrabber.ImageMode.COLOR
                        start()
                    }
                    if (loadJobId != currentLoadJobId) {
                        candidateGrabber.release()
                        return false
                    }
                    if (!candidateGrabber.hasVideo()) throw IllegalStateException("No video stream found")

                    if (!videoOnly && candidateGrabber.hasAudio()) {
                        val sampleRate = candidateGrabber.sampleRate
                        val channels = candidateGrabber.audioChannels
                        if (sampleRate > 0 && channels > 0) {
                            audioRenderer = runCatching {
                                AudioRenderer(
                                    AudioFormat(sampleRate.toFloat(), 16, channels, true, false)
                                ).also { it.setVolume(volume) }
                            }.getOrElse { error ->
                                // Audio output is optional. Keep decoding video when Windows rejects an unusual channel layout or the selected output device is unavailable.
                                println("[VideoPlayer] Audio output unavailable: ${error.message}")
                                null
                            }
                        }
                    }

                    val renderer = audioRenderer
                    // close() from the UI while this load was in flight already bumped loadJobId and found nothing to
                    // release, so these would leak. Publishing under the lock means close() sees either the old state or this one.
                    val published = synchronized(lifecycleLock) {
                        if (loadJobId != currentLoadJobId || audioRenderer !== renderer) {
                            false
                        } else {
                            grabber = candidateGrabber
                            true
                        }
                    }
                    if (!published) {
                        runCatching { candidateGrabber.release() }
                        if (renderer != null) {
                            synchronized(lifecycleLock) { if (audioRenderer === renderer) audioRenderer = null }
                            runCatching { renderer.close() }
                        }
                        return false
                    }
                    g = null

                    val nativeWidth = candidateGrabber.imageWidth
                    val nativeHeight = candidateGrabber.imageHeight
                    if (nativeWidth > 0 && nativeHeight > 0) {
                        val maxDecodeHeight = if (Settings.shared.desktopLowMemoryMode) MAX_DECODE_HEIGHT else Int.MAX_VALUE
                        val scale = if (nativeHeight > maxDecodeHeight) {
                            maxDecodeHeight.toFloat() / nativeHeight
                        } else {
                            1f
                        }
                        targetWidth = (nativeWidth * scale).roundToInt().coerceAtLeast(2)
                        targetHeight = (nativeHeight * scale).roundToInt().coerceAtLeast(2)
                        candidateGrabber.imageWidth = targetWidth
                        candidateGrabber.imageHeight = targetHeight
                    }
                    lastVideoPositionMs = positionProvider()
                    lastVideoTimestampMs = -1L
                    videoTimestampOffsetMs = null
                    return true
                } catch (error: Throwable) {
                    lastFailure = error
                    println("[VideoPlayer] Could not open video source: ${error.message}")
                    runCatching { g?.release() }
                    runCatching { audioRenderer?.close() }
                    audioRenderer = null
                }
            }
        }

        if (lastFailure != null) {
            println("[VideoPlayer] Unable to load video after all retries: ${lastFailure.message}")
        }
        close()
        return false
    }

    private fun videoSourceCandidates(url: String): List<String> {
        if (url.isBlank()) return emptyList()
        return buildList {
            add(url)
            when {
                url.contains("media.discordapp.net", ignoreCase = true) ->
                    add(url.replace(Regex("(?i)media\\.discordapp\\.net"), "cdn.discordapp.com"))
                url.contains("cdn.discordapp.com", ignoreCase = true) ->
                    add(url.replace(Regex("(?i)cdn\\.discordapp\\.com"), "media.discordapp.net"))
            }
        }.distinct()
    }

    fun play(startPaused: Boolean = false) {
        if (decodeThread?.isAlive == true) return
        val g = grabber ?: return
        closeRequested = false
        isRunning = true
        isPaused = startPaused
        pausedAtNanos = 0L
        if (!startPaused) {
            audioRenderer?.resume()
        } else {
            audioRenderer?.pause()
        }
        decodeThread =
            Thread {
                var initialVideoFrameRendered = false
                try {
                    if (isRunning) {
                        val startMs = positionProvider()
                        if (startMs > INITIAL_SEEK_MIN_MS) {
                            try {
                                g.setTimestamp((startMs - INITIAL_SEEK_LEAD_MS) * 1000, true)
                                onSeekLanded?.invoke(g.timestamp / 1000)
                            } catch (e: Exception) {
                                println("[VideoPlayer] Initial seek failed: ${e.message}")
                            }
                        }
                        audioRenderer?.resetPosition(startMs)
                        discardAudioUntilMs = startMs
                        videoClockStartMs = startMs
                        videoClockStartNanos = System.nanoTime()
                        lastVideoPositionMs = startMs
                        lastVideoTimestampMs = -1L
                        videoTimestampOffsetMs = null
                    }
                    while (isRunning) {
                        val seekTarget = pendingSeekMs
                        if (seekTarget != null) {
                            pendingSeekMs = null
                            try {
                                g.setTimestamp(seekTarget * 1000, true)
                                onSeekLanded?.invoke(g.timestamp / 1000)
                            } catch (e: Exception) {
                                println("[VideoPlayer] Seek failed: ${e.message}")
                            }
                            audioRenderer?.resetPosition(seekTarget)
                            discardAudioUntilMs = seekTarget
                            videoClockStartMs = seekTarget
                            videoClockStartNanos = System.nanoTime()
                            lastVideoPositionMs = seekTarget
                            lastVideoTimestampMs = -1L
                            videoTimestampOffsetMs = null
                            initialVideoFrameRendered = false
                        }
                        if (isPaused && initialVideoFrameRendered) {
                            Thread.sleep(10)
                            continue
                        }
                        val frame = g.grabFrame(true, true, true, false, false)
                        if (frame == null) {
                            if (repeat && isRunning) {
                                g.setTimestamp(0, true)
                                videoClockStartMs = 0L
                                videoClockStartNanos = System.nanoTime()
                                lastVideoPositionMs = 0L
                                lastVideoTimestampMs = -1L
                                videoTimestampOffsetMs = null
                                initialVideoFrameRendered = false
                                onSeekLanded?.invoke(0L)
                                continue
                            }
                            break
                        }
                        when {
                            frame.samples != null && audioRenderer != null -> {
                                if (!isPaused) writeAudio(frame)
                            }
                            frame.image != null -> {
                                val force = isPaused || !initialVideoFrameRendered
                                val rendered = renderVideo(frame, forceRender = force, isInitialFrame = !initialVideoFrameRendered)
                                if (rendered) {
                                    initialVideoFrameRendered = true
                                }
                            }
                        }
                    }
                } catch (e: InterruptedException) {
                    println("[VideoPlayer] Interrupted: ${e.message}")
                } catch (e: Exception) {
                    println("[VideoPlayer] Exception in decode loop: ${e.stackTraceToString()}")
                    if (!closeRequested) {
                        onError("Unable to play this video")
                    }
                } finally {
                    isRunning = false
                    if (!closeRequested) onEnded()
                }
            }.apply {
                priority = Thread.NORM_PRIORITY
                name = "VideoDecode-Thread"
                start()
            }
    }

    fun pause() {
        if (!isPaused) pausedAtNanos = System.nanoTime()
        isPaused = true
        audioRenderer?.pause()
    }

    fun resume() {
        if (isPaused) {
            val pauseStart = pausedAtNanos
            if (pauseStart != 0L && videoClockStartNanos != 0L) {
                videoClockStartNanos += (System.nanoTime() - pauseStart).coerceAtLeast(0L)
            }
            pausedAtNanos = 0L
        }
        if (decodeThread?.isAlive != true) {
            play(startPaused = false)
        } else {
            audioRenderer?.resume()
            isPaused = false
        }
    }

    fun seekTo(positionMs: Long) {
        pendingSeekMs = positionMs.coerceAtLeast(0L)
    }

    fun isPlaying(): Boolean = isRunning && !isPaused

    fun setVolume(volume: Float) {
        this.volume = volume.coerceIn(0.0f, 1.0f)
        audioRenderer?.setVolume(this.volume)
    }
    
    fun getDuration(): Long {
        val g = grabber ?: return 0L
        val containerDurationMs = g.getLengthInTime().div(1000)
        if (containerDurationMs > 0L) return containerDurationMs

        val streamDurationMs = g.getVideoDurationInTime().div(1000)
        if (streamDurationMs > 0L) return streamDurationMs

        val frameRate = g.frameRate
        val frameCount = g.getLengthInFrames()
        return if (frameRate > 0.0 && frameCount > 0) {
            (frameCount * 1000.0 / frameRate).roundToInt().toLong()
        } else {
            0L
        }
    }
    
    fun getCurrentPosition(): Long {
        // The audio device is not a reliable clock on Windows: a muted or unavailable line can report a position of zero forever.
        return lastVideoPositionMs
    }

    fun close() {
        closeRequested = true
        isRunning = false
        isPaused = false
        decodeThread?.interrupt()
        try {
            decodeThread?.join(300)
        } catch (e: InterruptedException) {
        }
        decodeThread = null
        val renderer: AudioRenderer?
        val g: FFmpegFrameGrabber?
        synchronized(lifecycleLock) {
            loadJobId = System.nanoTime()
            renderer = audioRenderer
            audioRenderer = null
            g = grabber
            grabber = null
        }
        runCatching { renderer?.close() }
        runCatching { g?.release() }
        pendingSeekMs = null
        videoClockStartNanos = 0L
        videoClockStartMs = 0L
        pausedAtNanos = 0L
        lastVideoPositionMs = 0L
        lastVideoTimestampMs = -1L
        videoTimestampOffsetMs = null
        pixelBuffer = null
        srcBuffer = null
    }

    private fun writeAudio(frame: Frame) {
        val frameMs = frame.timestamp / 1000
        val discardUntil = discardAudioUntilMs
        if (discardUntil != null && frameMs < discardUntil) {
            return
        }

        val samples = frame.samples ?: return
        if (samples.isEmpty()) return
        val renderer = audioRenderer ?: return
        when (val buffer = samples[0]) {
            is ShortBuffer -> {
                val byteArray = ByteArray(buffer.remaining() * 2)
                ByteBuffer.wrap(byteArray).order(ByteOrder.LITTLE_ENDIAN).asShortBuffer().put(buffer)
                renderer.writePcmData(byteArray)
            }

            is ByteBuffer -> {
                val byteArray = ByteArray(buffer.remaining())
                buffer.get(byteArray)
                renderer.writePcmData(byteArray)
            }

            else -> Unit
        }
    }

    private fun renderVideo(frame: Frame, forceRender: Boolean = false, isInitialFrame: Boolean = false): Boolean {
        // Keep video on its own monotonic clock. Java Sound's frame position can remain at zero on Windows, which previously made every frame after the first wait five seconds or get dropped.
        val frameMs = normalizedVideoTimestamp(frame)
        if (isInitialFrame && videoClockStartNanos != 0L) {
            videoClockStartMs = frameMs
            videoClockStartNanos = System.nanoTime()
        }
        if (!forceRender && videoClockStartNanos != 0L) {
            val targetNanos = videoClockStartNanos +
                (frameMs - videoClockStartMs).coerceAtLeast(0L) * 1_000_000L
            val waitNanos = targetNanos - System.nanoTime()
            if (waitNanos > 0L) {
                Thread.sleep(waitNanos / 1_000_000L, (waitNanos % 1_000_000L).toInt())
            } else if (-waitNanos > DROP_THRESHOLD_MS * 1_000_000L && !isInitialFrame) {
                return false
            }
        }

        val bitmap = toBitmap(frame) ?: return false
        lastVideoPositionMs = frameMs
        onFrame(bitmap)
        return true
    }

    private fun normalizedVideoTimestamp(frame: Frame): Long {
        val rawMs = frame.timestamp / 1000L
        val frameRate = grabber?.frameRate ?: 0.0
        val frameStepMs = if (frameRate > 0.0) {
            (1000.0 / frameRate).roundToInt().coerceAtLeast(1).toLong()
        } else {
            DEFAULT_FRAME_STEP_MS
        }
        val hasValidTimestamp = rawMs in 0L..MAX_REASONABLE_TIMESTAMP_MS
        if (lastVideoTimestampMs < 0L && hasValidTimestamp) {
            videoTimestampOffsetMs = rawMs - videoClockStartMs
        }
        val adjustedMs = if (hasValidTimestamp) {
            rawMs - (videoTimestampOffsetMs ?: 0L)
        } else {
            Long.MIN_VALUE
        }
        val timestamp = if (
            adjustedMs in 0L..MAX_REASONABLE_TIMESTAMP_MS &&
            (lastVideoTimestampMs < 0L || adjustedMs >= lastVideoTimestampMs)
        ) {
            adjustedMs
        } else if (lastVideoTimestampMs >= 0L) {
            lastVideoTimestampMs + frameStepMs
        } else {
            videoClockStartMs
        }
        lastVideoTimestampMs = timestamp
        return timestamp
    }

    private var srcBuffer: ByteArray? = null
    private var pixelBuffer: ByteArray? = null

    private fun toBitmap(frame: Frame): ImageBitmap? {
        val buffer = frame.image?.get(0) as? ByteBuffer ?: return null
        val width = frame.imageWidth
        val height = frame.imageHeight
        val stride = frame.imageStride
        if (width <= 0 || height <= 0 || stride <= 0) return null
        
        val requiredPixelSize = width * height * 4
        if (pixelBuffer?.size != requiredPixelSize) {
            pixelBuffer = ByteArray(requiredPixelSize)
        }
        val pixels = pixelBuffer!!
        
        buffer.clear()
        val requiredSrcSize = buffer.remaining()
        if (srcBuffer == null || srcBuffer!!.size < requiredSrcSize) {
            srcBuffer = ByteArray(requiredSrcSize)
        }
        val srcBytes = srcBuffer!!
        buffer.get(srcBytes, 0, requiredSrcSize)
        
        var dst = 0
        for (y in 0 until height) {
            var src = y * stride
            for (x in 0 until width) {
                val b = srcBytes[src].toInt() and 0xFF
                val g = srcBytes[src + 1].toInt() and 0xFF
                val r = srcBytes[src + 2].toInt() and 0xFF
                pixels[dst++] = b.toByte()
                pixels[dst++] = g.toByte()
                pixels[dst++] = r.toByte()
                pixels[dst++] = 0xFF.toByte()
                src += 3
            }
        }
        // installPixels copies into memory the Bitmap owns, so the reused pixelBuffer can't change a displayed frame.
        // Building the Bitmap directly avoids Image.toComposeImageBitmap(), which would make a second native copy.
        val bitmap = Bitmap()
        val installed = bitmap.installPixels(
            ImageInfo(width, height, ColorType.N32, ColorAlphaType.PREMUL, ColorSpace.sRGB),
            pixels,
            width * 4,
        )
        if (!installed) {
            bitmap.close()
            return null
        }
        bitmap.setImmutable()
        retireFrame(bitmap)
        return bitmap.asComposeImageBitmap()
    }

    // Skia pixels are native memory that the GC can't see, and decoding produces only tiny Java garbage, so frames
    // left to the Cleaner pile up at roughly width*height*4 bytes per frame while a video plays. Frames are closed
    // a few frames after being replaced, on the EDT, so no composition or draw can still be using them.
    private val recentFrames = ArrayDeque<Bitmap>()

    private fun retireFrame(bitmap: Bitmap) {
        synchronized(recentFrames) {
            recentFrames.addLast(bitmap)
            while (recentFrames.size > RETAINED_FRAMES) {
                val old = recentFrames.removeFirst()
                SwingUtilities.invokeLater { old.close() }
            }
        }
    }

    private companion object {
        const val MAX_DECODE_HEIGHT = 720
        const val RETAINED_FRAMES = 3
        const val DROP_THRESHOLD_MS = 150L
        const val DEFAULT_FRAME_STEP_MS = 33L
        const val MAX_REASONABLE_TIMESTAMP_MS = 24L * 60L * 60L * 1_000L
        const val INITIAL_SEEK_MIN_MS = 1_000L
        const val INITIAL_SEEK_LEAD_MS = 0L
    }
}

private fun formatDuration(ms: Long, unknownWhenZero: Boolean = false): String {
    if (unknownWhenZero && ms <= 0L) return "--:--"
    val totalSeconds = ms / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "${minutes}:${seconds.toString().padStart(2, '0')}"
}
