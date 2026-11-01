package me.lampu.lampcord.shared.imaging

import coil3.Image
import coil3.ImageLoader
import coil3.decode.DecodeResult
import coil3.decode.Decoder
import coil3.fetch.SourceFetchResult
import coil3.request.Options
import okio.Buffer
import okio.BufferedSource
import okio.use
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.Codec
import org.jetbrains.skia.Color
import org.jetbrains.skia.Data
import org.jetbrains.skia.FilterMipmap
import org.jetbrains.skia.FilterMode
import org.jetbrains.skia.MipmapMode
import org.jetbrains.skia.Paint
import org.jetbrains.skia.Rect
import org.jetbrains.skia.SamplingMode
import kotlin.time.TimeSource

internal class IosAnimatedDecoder(
    private val source: BufferedSource,
    private val timeSource: TimeSource = TimeSource.Monotonic,
    private val animates: Boolean = true
) : Decoder {

    override suspend fun decode(): DecodeResult {
        val data = source.readByteArray()
        val codec = Codec.makeFromData(Data.makeFromBytes(data))
        val delays = gifFrameDelays(data)
        return DecodeResult(
            image = SkiaAnimatedImage(
                SkiaCodecFrameSource(codec, codec.width, codec.height, delays),
                timeSource,
                animates
            ),
            isSampled = false
        )
    }

    class Factory(
        private val timeSource: TimeSource = TimeSource.Monotonic
    ) : Decoder.Factory {
        override fun create(
            result: SourceFetchResult,
            options: Options,
            imageLoader: ImageLoader
        ): Decoder? {
            val source = result.source.source()
            if (!isAnimated(source)) return null
            return IosAnimatedDecoder(source, timeSource, options.allowsAnimation())
        }
    }
}

private const val HEADER_BYTES = 64L
private const val DEFAULT_GIF_DELAY_MS = 100

private fun isAnimated(source: BufferedSource): Boolean {
    val head = Buffer()
    source.peek().use { peeked -> peeked.read(head, HEADER_BYTES) }
    val bytes = head.snapshot().toByteArray()

    if (bytes.asciiEquals(0, "GIF87a") || bytes.asciiEquals(0, "GIF89a")) return true

    if (bytes.asciiEquals(0, "RIFF") && bytes.asciiEquals(8, "WEBP")) {
        return bytes.size > 20 && bytes.asciiEquals(12, "VP8X") && (bytes[20].toInt() and 0x02) != 0
    }

    return false
}

private fun gifFrameDelays(bytes: ByteArray): IntArray? {
    if (!bytes.asciiEquals(0, "GIF87a") && !bytes.asciiEquals(0, "GIF89a")) return null
    if (bytes.size < 13) return null

    var pos = 13
    val globalFlags = bytes[10].toInt() and 0xff
    if (globalFlags and 0x80 != 0) {
        pos += 3 * (1 shl ((globalFlags and 0x07) + 1))
    }

    val delays = mutableListOf<Int>()
    var pendingDelay = DEFAULT_GIF_DELAY_MS

    while (pos < bytes.size) {
        when (bytes[pos].toInt() and 0xff) {
            0x3B -> break
            0x21 -> {
                if (pos + 1 >= bytes.size) break
                val label = bytes[pos + 1].toInt() and 0xff
                pos += 2
                if (label == 0xF9 && pos < bytes.size) {
                    val blockSize = bytes[pos].toInt() and 0xff
                    if (blockSize >= 4 && pos + 1 + blockSize <= bytes.size) {
                        val centiseconds = (bytes[pos + 2].toInt() and 0xff) or
                            ((bytes[pos + 3].toInt() and 0xff) shl 8)
                        pendingDelay = if (centiseconds > 0) centiseconds * 10 else DEFAULT_GIF_DELAY_MS
                    }
                    pos += 1 + blockSize
                    if (pos < bytes.size && bytes[pos].toInt() == 0) pos++
                } else {
                    pos = skipSubBlocks(bytes, pos)
                }
            }
            0x2C -> {
                if (pos + 9 >= bytes.size) break
                val imageFlags = bytes[pos + 9].toInt() and 0xff
                pos += 10
                if (imageFlags and 0x80 != 0) {
                    pos += 3 * (1 shl ((imageFlags and 0x07) + 1))
                }
                if (pos >= bytes.size) break
                pos++
                pos = skipSubBlocks(bytes, pos)
                delays += pendingDelay
                pendingDelay = DEFAULT_GIF_DELAY_MS
            }
            else -> pos++
        }
    }

    return delays.takeIf { it.isNotEmpty() }?.toIntArray()
}

private fun skipSubBlocks(bytes: ByteArray, start: Int): Int {
    var pos = start
    while (pos < bytes.size) {
        val size = bytes[pos].toInt() and 0xff
        pos++
        if (size == 0) break
        pos += size
    }
    return pos
}

private fun ByteArray.asciiEquals(offset: Int, text: String): Boolean {
    if (offset < 0 || offset + text.length > size) return false
    for (index in text.indices) {
        if (this[offset + index].toInt() != text[index].code) return false
    }
    return true
}

private class SkiaCodecFrameSource(
    private val codec: Codec,
    override val width: Int,
    override val height: Int,
    delaysMs: IntArray? = null
) : FrameSource {

    private val durations = IntArray(codec.frameCount) { index ->
        delaysMs?.getOrNull(index)?.takeIf { it > 0 }
            ?: codec.getFrameInfo(index).duration.takeIf { it > 0 }
            ?: DEFAULT_FRAME_DURATION_MS
    }

    private val cumulative = LongArray(durations.size + 1).also { cumulative ->
        for (index in durations.indices) {
            cumulative[index + 1] = cumulative[index] + durations[index]
        }
    }
    private val totalDurationMs = cumulative.last().coerceAtLeast(1L)

    override val frameCount: Int get() = durations.size
    override val byteSize: Long = 4L * width * height

    override fun frameIndexAt(elapsedMs: Long): Int {
        if (durations.size <= 1) return 0
        val time = elapsedMs % totalDurationMs
        var low = 0
        var high = durations.size - 1
        while (low < high) {
            val mid = (low + high + 1) / 2
            if (cumulative[mid] <= time) low = mid else high = mid - 1
        }
        return low
    }

    override fun renderFrame(target: Bitmap, frameIndex: Int) {
        codec.readPixels(target, frameIndex)
    }

    private companion object {
        const val DEFAULT_FRAME_DURATION_MS = DEFAULT_GIF_DELAY_MS
    }
}

private interface FrameSource {
    val width: Int
    val height: Int
    val byteSize: Long
    val frameCount: Int

    fun frameIndexAt(elapsedMs: Long): Int
    fun renderFrame(target: Bitmap, frameIndex: Int)
}

private val MIPMAP_SAMPLING: SamplingMode = FilterMipmap(FilterMode.LINEAR, MipmapMode.LINEAR)

private class SkiaAnimatedImage(
    private val frameSource: FrameSource,
    timeSource: TimeSource = TimeSource.Monotonic,
    private val animates: Boolean = true
) : Image {

    private val startedAt = timeSource.markNow()
    private val buffers = arrayOfNulls<Bitmap>(2)
    private val paint = Paint().apply { isAntiAlias = true }

    private var activeSlot = 0
    private var lastFrame = -1
    private var cachedImage: org.jetbrains.skia.Image? = null

    override val width: Int get() = frameSource.width
    override val height: Int get() = frameSource.height
    override val size: Long get() = frameSource.byteSize
    override val shareable: Boolean = false

    private fun buffer(slot: Int): Bitmap {
        buffers[slot]?.let { return it }
        val bitmap = Bitmap()
        check(bitmap.allocN32Pixels(frameSource.width, frameSource.height, false)) {
            "Failed to allocate ${frameSource.width}x${frameSource.height} frame buffer"
        }
        Canvas(bitmap).clear(Color.TRANSPARENT)
        bitmap.setImmutable()
        buffers[slot] = bitmap
        return bitmap
    }

    override fun draw(canvas: Canvas) {
        val frame = if (animates) {
            frameSource.frameIndexAt(startedAt.elapsedNow().inWholeMilliseconds)
        } else {
            0
        }

        val image = if (frame == lastFrame) {
            cachedImage ?: org.jetbrains.skia.Image.makeFromBitmap(buffer(activeSlot))
                .also { cachedImage = it }
        } else {
            val slot = 1 - activeSlot
            val target = buffer(slot)
            frameSource.renderFrame(target, frame)
            target.notifyPixelsChanged()
            activeSlot = slot
            lastFrame = frame

            cachedImage?.close()
            org.jetbrains.skia.Image.makeFromBitmap(target).also { cachedImage = it }
        }

        val bounds = Rect.makeWH(width.toFloat(), height.toFloat())
        canvas.drawImageRect(image, bounds, bounds, MIPMAP_SAMPLING, paint, false)
    }
}
