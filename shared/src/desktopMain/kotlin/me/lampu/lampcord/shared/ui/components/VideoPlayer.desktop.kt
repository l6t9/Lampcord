package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toComposeImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.playback.ffmpeg.AudioRenderer
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegFrameGrabber
import me.lampu.lampcord.shared.playback.ffmpeg.FFmpegLogCallback
import me.lampu.lampcord.shared.playback.ffmpeg.Frame
import me.lampu.lampcord.shared.playback.ffmpeg.FrameGrabber
import me.lampu.lampcord.shared.ui.icons.Icons
import org.bytedeco.ffmpeg.global.avutil.AV_SAMPLE_FMT_S16
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorSpace
import org.jetbrains.skia.ColorType
import org.jetbrains.skia.Image as SkiaImage
import org.jetbrains.skia.ImageInfo
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.ShortBuffer
import javax.sound.sampled.AudioFormat
import kotlin.math.roundToInt

@Composable
actual fun VideoPlayer(
    url: String,
    modifier: Modifier,
    loop: Boolean,
    showControls: Boolean
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
            kotlinx.coroutines.delay(50)
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
            CircularProgressIndicator(color = Color.White)
        }

        // Overlay controls
        if (showControls) {
            AnimatedVisibility(
                visible = isHovered || !isPlaying,
                enter = fadeIn(),
                exit = fadeOut(),
                modifier = Modifier.align(Alignment.BottomCenter)
            ) {
                Surface(
                    color = Color.Black.copy(alpha = 0.6f),
                    shape = RoundedCornerShape(topStart = 12.dp, topEnd = 12.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(84.dp)
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        // Progress Slider
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Text(
                                text = formatDuration(currentTime),
                                style = MaterialTheme.typography.labelSmall,
                                color = Color.White
                            )
                            Slider(
                                value = if (duration > 0) (currentTime.toFloat() / duration).coerceIn(0f, 1f) else 0f,
                                onValueChange = { 
                                    val target = (it * duration).toLong()
                                    currentTime = target
                                    player.seekTo(target)
                                },
                                modifier = Modifier.weight(1f),
                                colors = SliderDefaults.colors(
                                    thumbColor = Color.White,
                                    activeTrackColor = MaterialTheme.colorScheme.primary,
                                    inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                                )
                            )
                            Text(
                                text = formatDuration(duration),
                                style = MaterialTheme.typography.labelSmall,
                                color = Color.White
                            )
                        }

                        // Buttons Row
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                IconButton(onClick = { isPlaying = !isPlaying }) {
                                    Icon(
                                        imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                                        contentDescription = "Play/Pause",
                                        tint = Color.White
                                    )
                                }
                                
                                Spacer(Modifier.width(8.dp))
                                
                                Icon(
                                    imageVector = Icons.AutoMirrored.Filled.VolumeUp,
                                    contentDescription = "Volume",
                                    tint = Color.White,
                                    modifier = Modifier.size(20.dp)
                                )
                                Slider(
                                    value = volume,
                                    onValueChange = { volume = it },
                                    modifier = Modifier.width(100.dp).padding(horizontal = 8.dp),
                                    colors = SliderDefaults.colors(
                                        thumbColor = Color.White,
                                        activeTrackColor = Color.White,
                                        inactiveTrackColor = Color.White.copy(alpha = 0.3f)
                                    )
                                )
                            }
                        }
                    }
                }
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
            if (audioClock() >= frameMs - SYNC_AHEAD_ALLOWANCE_MS) return true
            if (waited >= MAX_WAIT_MS) return true
            Thread.sleep(5)
            waited += 5
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
        const val SYNC_AHEAD_ALLOWANCE_MS = 0L
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
