@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.toComposeImageBitmap
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.onPointerEvent
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupPositionProvider
import androidx.compose.ui.window.PopupProperties
import me.lampu.lampcord.shared.model.Attachment
import me.lampu.lampcord.shared.model.DiscordMedia
import me.lampu.lampcord.shared.playback.ffmpeg.AudioRenderer
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegFrameGrabber
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegLogCallback
import me.lampu.lampcord.shared.playback.ffmpeg.Frame
import me.lampu.lampcord.shared.playback.ffmpeg.FrameGrabber
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import org.bytedeco.ffmpeg.global.avutil.AV_SAMPLE_FMT_S16
import org.jetbrains.skia.*
import org.jetbrains.skia.Image as SkiaImage
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.ShortBuffer
import javax.sound.sampled.AudioFormat
import kotlin.math.roundToInt
import kotlinx.coroutines.delay

@Composable
actual fun VideoPlayer(
    url: String,
    modifier: Modifier,
    loop: Boolean,
    showControls: Boolean,
    title: String?,
    subtitle: String?,
    compact: Boolean,
    onFullscreenClick: (() -> Unit)?
) {
    var videoFrame by remember { mutableStateOf<ImageBitmap?>(null) }
    var isPlaying by remember { mutableStateOf(false) }
    var currentTime by remember { mutableLongStateOf(0L) }
    var duration by remember { mutableLongStateOf(0L) }
    var volume by remember { mutableFloatStateOf(1.0f) }
    var isResolving by remember { mutableStateOf(true) }

    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()

    val player = remember {
        DesktopVideoPlayer(
            onFrame = { videoFrame = it },
            onEnded = {
                if (!loop) {
                    isPlaying = false
                }
            }
        )
    }

    LaunchedEffect(url) {
        isResolving = true
        val success = player.load(url, headers = mapOf("User-Agent" to "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")) {
            currentTime
        }
        isResolving = false
        if (success) {
            duration = player.getDuration()
            player.setVolume(volume)
            player.play(startPaused = false)
            isPlaying = true
        }
    }

    LaunchedEffect(isPlaying) {
        if (isPlaying) player.resume() else player.pause()
    }

    LaunchedEffect(volume) {
        player.setVolume(volume)
    }

    DisposableEffect(Unit) {
        onDispose {
            player.close()
        }
    }

    // Progress polling
    LaunchedEffect(isPlaying) {
        while (isPlaying) {
            currentTime = player.getCurrentPosition()
            delay(50)
        }
    }

    Box(
        modifier = modifier
            .background(Color.Black)
            .hoverable(interactionSource),
        contentAlignment = Alignment.Center
    ) {
        val frame = videoFrame
        if (frame != null) {
            Image(
                bitmap = frame,
                contentDescription = null,
                contentScale = ContentScale.Fit,
                modifier = Modifier
                    .fillMaxSize()
                    .clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                        isPlaying = !isPlaying
                    }
            )
        } else if (isResolving) {
            ContainedLoadingIndicator(indicatorColor = Color.White)
        }

        // Discord style Top Right Download Button (Compact mode)
        if (compact && showControls) {
            val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
            AnimatedVisibility(
                visible = isHovered,
                enter = fadeIn(),
                exit = fadeOut(),
                modifier = Modifier.align(Alignment.TopEnd).padding(8.dp)
            ) {
                Surface(
                    onClick = { uriHandler.openUri(url) },
                    shape = RoundedCornerShape(4.dp),
                    color = Color.Black.copy(alpha = 0.6f),
                    modifier = Modifier.size(32.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Filled.Download,
                            contentDescription = "Download",
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                }
            }
        }

        // Metrolist style Hoverable Play Controls
        if (showControls) {
            HoverablePlayControls(
                isPlaying = isPlaying,
                isHovered = isHovered,
                compact = compact,
                onTogglePlay = { isPlaying = !isPlaying }
            )

            // Metrolist style Bottom Video Controls
            AnimatedVisibility(
                visible = isHovered || !isPlaying,
                enter = fadeIn(),
                exit = fadeOut(),
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
                    onTogglePlay = { isPlaying = !isPlaying },
                    onSeek = { 
                        val target = (it * duration).toLong()
                        currentTime = target
                        player.seekTo(target)
                    },
                    onVolumeChange = { volume = it },
                    onFullscreenClick = onFullscreenClick
                )
            }
        }
    }
}

@Composable
private fun HoverablePlayControls(
    isPlaying: Boolean,
    isHovered: Boolean,
    compact: Boolean,
    onTogglePlay: () -> Unit
) {
    AnimatedVisibility(
        visible = isHovered && !isPlaying,
        enter = fadeIn(),
        exit = fadeOut(),
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

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
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
    onTogglePlay: () -> Unit,
    onSeek: (Float) -> Unit,
    onVolumeChange: (Float) -> Unit,
    onFullscreenClick: (() -> Unit)?
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = Color.Black.copy(alpha = 0.5f), // Semi-translucent for better effect
        shape = if (compact) RectangleShape else RoundedCornerShape(topStart = 12.dp, topEnd = 12.dp)
    ) {
        Column(
            modifier = Modifier
                .padding(horizontal = if (compact) 12.dp else 24.dp, vertical = if (compact) 8.dp else 20.dp)
        ) {
            if (compact) {
                // Single line Discord-like layout
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
                        text = "${formatDuration(currentTime)} / ${formatDuration(duration)}",
                        style = MaterialTheme.typography.labelSmall,
                        color = Color.White.copy(alpha = 0.9f)
                    )

                    Slider(
                        value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                        onValueChange = onSeek,
                        modifier = Modifier.weight(1f),
                        colors = SliderDefaults.colors(
                            thumbColor = Color.White,
                            activeTrackColor = MaterialTheme.colorScheme.primary,
                            inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                        )
                    )

                    VideoConnectedButtonGroup(
                        url = url,
                        volume = volume,
                        onVolumeChange = onVolumeChange,
                        onFullscreenClick = onFullscreenClick,
                        compact = true
                    )
                }
            } else {
                // Slider Row
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Text(
                        text = formatDuration(currentTime),
                        style = MaterialTheme.typography.labelMedium,
                        color = Color.White.copy(alpha = 0.9f)
                    )
                    Slider(
                        value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                        onValueChange = onSeek,
                        modifier = Modifier.weight(1f),
                        colors = SliderDefaults.colors(
                            thumbColor = Color.White,
                            activeTrackColor = MaterialTheme.colorScheme.primary,
                            inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                        )
                    )
                    Text(
                        text = formatDuration(duration),
                        style = MaterialTheme.typography.labelMedium,
                        color = Color.White.copy(alpha = 0.9f),
                        textAlign = TextAlign.End
                    )
                }

                Spacer(Modifier.height(8.dp))

                // Info and Buttons Row
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    // Left: Info
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
                                    modifier = Modifier.basicMarquee()
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

                    // Right: Actions
                    VideoConnectedButtonGroup(
                        url = url,
                        volume = volume,
                        onVolumeChange = onVolumeChange,
                        onFullscreenClick = onFullscreenClick,
                        compact = false
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun VideoConnectedButtonGroup(
    url: String,
    volume: Float,
    onVolumeChange: (Float) -> Unit,
    onFullscreenClick: (() -> Unit)?,
    compact: Boolean
) {
    val groupHeight = if (compact) 32.dp else 40.dp
    val iconSize = if (compact) 18.dp else 20.dp
    val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current

    val volumeInteractionSource = remember { MutableInteractionSource() }
    val isVolumeHovered by volumeInteractionSource.collectIsHoveredAsState()
    val sliderInteractionSource = remember { MutableInteractionSource() }
    val isSliderHovered by sliderInteractionSource.collectIsHoveredAsState()

    var showSlider by remember { mutableStateOf(false) }
    var isInteracting by remember { mutableStateOf(false) }
    var previousVolume by remember { mutableStateOf(0.5f) }

    LaunchedEffect(isVolumeHovered, isSliderHovered) {
        if (isVolumeHovered || isSliderHovered) {
            showSlider = true
        } else {
            delay(150)
            showSlider = false
        }
    }

    val volumeIcon = when {
        volume > 0.66f -> Icons.AutoMirrored.Filled.VolumeUp
        volume > 0.33f -> Icons.AutoMirrored.Filled.VolumeDown
        volume > 0f -> Icons.AutoMirrored.Filled.VolumeMute
        else -> Icons.AutoMirrored.Filled.VolumeOff
    }

    ButtonGroup(
        overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
        modifier = Modifier.height(groupHeight)
    ) {
        // Volume Toggle (Leading)
        customItem(
            buttonGroupContent = {
                Box(Modifier.fillMaxSize()) {
                    ToggleButton(
                        checked = volume > 0f,
                        onCheckedChange = { 
                            if (volume > 0f) {
                                previousVolume = volume
                                onVolumeChange(0f)
                            } else {
                                onVolumeChange(previousVolume)
                            }
                        },
                        shapes = ButtonGroupDefaults.connectedLeadingButtonShapes(),
                        colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                        contentPadding = PaddingValues(0.dp),
                        modifier = Modifier
                            .fillMaxSize()
                            .hoverable(volumeInteractionSource)
                            .onPointerEvent(PointerEventType.Scroll) { event ->
                                val delta = event.changes.firstOrNull()?.scrollDelta?.y ?: 0f
                                if (delta != 0f) {
                                    isInteracting = true
                                    onVolumeChange((volume - delta * 0.05f).coerceIn(0f, 1f))
                                }
                            }
                    ) {
                        Icon(volumeIcon, null, modifier = Modifier.size(iconSize))
                    }

                    if (showSlider) {
                        val positionProvider = remember {
                            object : PopupPositionProvider {
                                override fun calculatePosition(
                                    anchorBounds: androidx.compose.ui.unit.IntRect,
                                    windowSize: androidx.compose.ui.unit.IntSize,
                                    layoutDirection: androidx.compose.ui.unit.LayoutDirection,
                                    popupContentSize: androidx.compose.ui.unit.IntSize,
                                ): IntOffset {
                                    val x = anchorBounds.left + (anchorBounds.width - popupContentSize.width) / 2
                                    val y = anchorBounds.top - popupContentSize.height - 8
                                    return IntOffset(x, y)
                                }
                            }
                        }

                        Popup(
                            popupPositionProvider = positionProvider,
                            onDismissRequest = { showSlider = false },
                            properties = PopupProperties(focusable = false),
                        ) {
                            Surface(
                                modifier = Modifier
                                    .requiredSize(width = 48.dp, height = 116.dp)
                                    .shadow(8.dp, RoundedCornerShape(12.dp))
                                    .hoverable(sliderInteractionSource),
                                shape = RoundedCornerShape(12.dp),
                                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                            ) {
                                Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(8.dp)) {
                                    MetrolistVerticalVolumeSlider(
                                        value = volume,
                                        onVolumeChange = onVolumeChange,
                                        modifier = Modifier.fillMaxSize(),
                                        isInteracting = isInteracting,
                                        onInteractionChange = { isInteracting = it }
                                    )
                                }
                            }
                        }
                    }
                }
            },
            menuContent = {
                DropdownMenuItem(
                    text = { Text("Volume") },
                    onClick = { if (volume > 0f) onVolumeChange(0f) else onVolumeChange(0.5f) },
                    leadingIcon = { Icon(volumeIcon, null) }
                )
            }
        )

        // URL Copy (Only in non-compact)
        if (!compact) {
            customItem(
                buttonGroupContent = {
                    ToggleButton(
                        checked = false,
                        onCheckedChange = { setClipboardText(url) },
                        shapes = ButtonGroupDefaults.connectedMiddleButtonShapes(),
                        colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                        contentPadding = PaddingValues(0.dp),
                        modifier = Modifier.fillMaxSize()
                    ) {
                        Icon(Icons.Filled.ContentCopy, "Copy URL", modifier = Modifier.size(iconSize - 2.dp))
                    }
                },
                menuContent = {
                    DropdownMenuItem(
                        text = { Text("Copy URL") },
                        onClick = { setClipboardText(url) },
                        leadingIcon = { Icon(Icons.Filled.ContentCopy, null) }
                    )
                }
            )
        }

        // Download (Only in non-compact)
        if (!compact) {
            customItem(
                buttonGroupContent = {
                    ToggleButton(
                        checked = false,
                        onCheckedChange = { uriHandler.openUri(url) },
                        shapes = if (onFullscreenClick == null) ButtonGroupDefaults.connectedTrailingButtonShapes() else ButtonGroupDefaults.connectedMiddleButtonShapes(),
                        colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                        contentPadding = PaddingValues(0.dp),
                        modifier = Modifier.fillMaxSize()
                    ) {
                        Icon(Icons.Filled.Download, "Download", modifier = Modifier.size(iconSize))
                    }
                },
                menuContent = {
                    DropdownMenuItem(
                        text = { Text("Download") },
                        onClick = { uriHandler.openUri(url) },
                        leadingIcon = { Icon(Icons.Filled.Download, null) }
                    )
                }
            )
        }

        // Fullscreen (Trailing)
        if (onFullscreenClick != null) {
            customItem(
                buttonGroupContent = {
                    ToggleButton(
                        checked = false,
                        onCheckedChange = { onFullscreenClick() },
                        shapes = ButtonGroupDefaults.connectedTrailingButtonShapes(),
                        colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                        contentPadding = PaddingValues(0.dp),
                        modifier = Modifier.fillMaxSize()
                    ) {
                        Icon(Icons.Filled.Fullscreen, "Fullscreen", modifier = Modifier.size(iconSize + 4.dp))
                    }
                },
                menuContent = {
                    DropdownMenuItem(
                        text = { Text("Fullscreen") },
                        onClick = { onFullscreenClick() },
                        leadingIcon = { Icon(Icons.Filled.Fullscreen, null) }
                    )
                }
            )
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun MetrolistVerticalVolumeSlider(
    value: Float,
    onVolumeChange: (Float) -> Unit,
    modifier: Modifier = Modifier,
    accentColor: Color = MaterialTheme.colorScheme.primary,
    isInteracting: Boolean = false,
    onInteractionChange: (Boolean) -> Unit = {},
) {
    val trackWidth = 24.dp

    Box(modifier = modifier) {
        // Track
        Box(
            modifier = Modifier
                .width(trackWidth)
                .fillMaxHeight()
                .align(Alignment.Center)
                .clip(RoundedCornerShape(trackWidth / 2))
                .background(accentColor.copy(alpha = 0.2f)),
        )

        // Progress
        Box(
            modifier = Modifier
                .width(trackWidth)
                .fillMaxHeight(value)
                .align(Alignment.BottomCenter)
                .clip(RoundedCornerShape(bottomStart = trackWidth / 2, bottomEnd = trackWidth / 2))
                .background(accentColor),
        )

        // Handle and Input
        Box(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(Unit) {
                    detectDragGestures { change, _ ->
                        change.consume()
                        onInteractionChange(true)
                        val newValue = 1f - (change.position.y / size.height).coerceIn(0f, 1f)
                        onVolumeChange(newValue)
                    }
                }
        )

        // Value tooltip
        AnimatedVisibility(
            visible = isInteracting,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier.align(Alignment.Center),
        ) {
            Surface(
                shape = RoundedCornerShape(6.dp),
                color = MaterialTheme.colorScheme.surface,
                shadowElevation = 2.dp,
            ) {
                Text(
                    text = "${(value * 100).roundToInt()}%",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                )
            }
        }
    }
}

/**
 * Exact port of Metrolist feature branch VideoPlayer implementation.
 */
private class DesktopVideoPlayer(
    private val onFrame: (ImageBitmap) -> Unit,
    private val onEnded: () -> Unit = {},
    private val onSeekLanded: ((Long) -> Unit)? = null,
) {
    private var grabber: FFmpegFrameGrabber? = null
    private var audioRenderer: AudioRenderer? = null

    @Volatile
    private var isRunning = false

    @Volatile
    private var isPaused = false

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

    /** True when the loaded stream contains its own audio track. */
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
        return try {
            val g =
                FFmpegFrameGrabber(url).apply {
                    if (url.startsWith("http://") || url.startsWith("https://")) {
                        options["reconnect"] = "1"
                        options["reconnect_streamed"] = "1"
                        options["reconnect_delay_max"] = "5"
                        options["timeout"] = "15000000"
                        options["rw_timeout"] = "15000000"
                        options["user_agent"] =
                            headers.entries.firstOrNull { it.key.equals("User-Agent", ignoreCase = true) }?.value
                                ?: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
                        headers
                            .filterKeys { !it.equals("User-Agent", ignoreCase = true) }
                            .takeIf { it.isNotEmpty() }
                            ?.entries
                            ?.joinToString(separator = "\r\n", postfix = "\r\n") { "${it.key}: ${it.value}" }
                            ?.let { options["headers"] = it }
                        
                        options["probesize"] = "2000000" // 2MB
                        options["analyzeduration"] = "2000000" // 2 seconds
                    }
                    sampleFormat = AV_SAMPLE_FMT_S16
                    imageMode = FrameGrabber.ImageMode.COLOR
                    start()
                }
            if (loadJobId != currentLoadJobId) {
                g.release()
                return false
            }
            if (!g.hasVideo()) {
                g.release()
                return false
            }
            if (g.hasAudio()) {
                val audioFormat =
                    AudioFormat(
                        g.sampleRate.toFloat(),
                        16,
                        g.audioChannels,
                        true,
                        false,
                    )
                audioRenderer = AudioRenderer(audioFormat).also { it.setVolume(volume) }
            }
            val nativeWidth = g.imageWidth
            val nativeHeight = g.imageHeight
            val scale = if (nativeHeight > MAX_DECODE_HEIGHT) MAX_DECODE_HEIGHT.toFloat() / nativeHeight else 1f
            targetWidth = (nativeWidth * scale).roundToInt().coerceAtLeast(2)
            targetHeight = (nativeHeight * scale).roundToInt().coerceAtLeast(2)
            g.imageWidth = targetWidth
            g.imageHeight = targetHeight
            grabber = g
            true
        } catch (e: Exception) {
            e.printStackTrace()
            runCatching { close() }
            false
        }
    }

    fun play(startPaused: Boolean = false) {
        if (decodeThread?.isAlive == true) return
        val g = grabber ?: return
        isRunning = true
        isPaused = startPaused
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
                    }
                    while (isRunning) {
                        if (isPaused && initialVideoFrameRendered) {
                            Thread.sleep(10)
                            continue
                        }
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
                            initialVideoFrameRendered = false
                        }
                        val frame = g.grabFrame(true, true, true, false, false)
                        if (frame == null) {
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
                } finally {
                    isRunning = false
                    onEnded()
                }
            }.apply {
                priority = Thread.MAX_PRIORITY
                name = "VideoDecode-Thread"
                start()
            }
    }

    fun pause() {
        isPaused = true
        audioRenderer?.pause()
    }

    fun resume() {
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
    
    fun getDuration(): Long = grabber?.getLengthInTime()?.div(1000) ?: 0L
    
    fun getCurrentPosition(): Long {
        val renderer = audioRenderer
        return if (renderer != null) {
            renderer.getPlayedPositionMs()
        } else {
            grabber?.timestamp?.div(1000) ?: 0L
        }
    }

    fun close() {
        isRunning = false
        isPaused = false
        decodeThread?.interrupt()
        try {
            decodeThread?.join(300)
        } catch (e: InterruptedException) {
            // Ignored
        }
        decodeThread = null
        runCatching { audioRenderer?.close() }
        audioRenderer = null
        val g = grabber
        grabber = null
        runCatching { g?.release() }
        pendingSeekMs = null
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
        val frameMs = frame.timestamp / 1000
        val clock = audioClock()
        if (!forceRender) {
            if (clock < frameMs - SYNC_AHEAD_ALLOWANCE_MS) {
                if (!waitForAudioClock(frameMs)) return false
            }
            val diff = clock - frameMs
            if (diff > DROP_THRESHOLD_MS && !isInitialFrame) {
                return false
            }
        }
        val bitmap = toBitmap(frame)
        if (bitmap == null) {
            return false
        }
        onFrame(bitmap)
        return true
    }

    private fun audioClock(): Long {
        val renderer = audioRenderer
        return if (renderer != null) {
            renderer.getPlayedPositionMs()
        } else {
            positionProvider()
        }
    }

    private fun waitForAudioClock(frameMs: Long): Boolean {
        var waited = 0L
        while (isRunning && !isPaused) {
            if (pendingSeekMs != null) return false
            if (audioClock() >= frameMs - SYNC_AHEAD_ALLOWANCE_MS) return true
            if (waited >= MAX_WAIT_MS) return true
            Thread.sleep(1)
            waited += 1
        }
        return isRunning
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
        val image =
            SkiaImage.makeRaster(
                ImageInfo(width, height, ColorType.N32, ColorAlphaType.PREMUL, ColorSpace.sRGB),
                pixels,
                width * 4,
            )
        return image.toComposeImageBitmap()
    }

    private companion object {
        const val MAX_DECODE_HEIGHT = 720
        const val SYNC_AHEAD_ALLOWANCE_MS = 1L
        const val DROP_THRESHOLD_MS = 150L
        const val MAX_WAIT_MS = 5_000L
        const val INITIAL_SEEK_MIN_MS = 1_000L
        const val INITIAL_SEEK_LEAD_MS = 0L
    }
}

private fun formatDuration(ms: Long): String {
    val totalSeconds = ms / 1000
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "${minutes}:${seconds.toString().padStart(2, '0')}"
}
