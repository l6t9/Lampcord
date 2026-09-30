package me.lampu.lampcord.shared.imaging

import coil3.ImageLoader
import coil3.decode.DecodeResult
import coil3.decode.Decoder
import coil3.fetch.SourceFetchResult
import coil3.request.Options
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okio.Buffer
import okio.BufferedSource
import org.jetbrains.skia.Codec
import org.jetbrains.skia.Data

internal class DesktopAnimatedDecoder(
    private val source: okio.BufferedSource,
    private val timeSource: kotlin.time.TimeSource = kotlin.time.TimeSource.Monotonic,
    private val animates: Boolean = true,
) : Decoder {

    override suspend fun decode(): DecodeResult = withContext(Dispatchers.IO) {
        val data = source.readByteArray()
        if (hasApngChunk(data)) {
            val frames = ApngFrameSource.parse(data)
                ?: error("Failed to parse APNG")
            DecodeResult(SkiaAnimatedImage(frames, timeSource, animates), isSampled = false)
        } else {
            val codec = Codec.makeFromData(Data.makeFromBytes(data))
            val frameSource = SkiaCodecFrameSource(codec, codec.width, codec.height)
            DecodeResult(SkiaAnimatedImage(frameSource, timeSource, animates), isSampled = false)
        }
    }

    class Factory(
        private val timeSource: kotlin.time.TimeSource = kotlin.time.TimeSource.Monotonic
    ) : Decoder.Factory {
        override fun create(
            result: SourceFetchResult,
            options: Options,
            imageLoader: ImageLoader
        ): Decoder? {
            val source = result.source.source()

            if (!isAnimatable(source)) return null
            return DesktopAnimatedDecoder(source, timeSource, options.allowsAnimation())
        }
    }
}

private val PNG_SIGNATURE = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)

private const val HEADER_BYTES = 64L

private fun isAnimatable(source: BufferedSource): Boolean {
    val head = Buffer()
    source.peek().use { peeked -> peeked.read(head, HEADER_BYTES) }
    val bytes = head.snapshot().toByteArray()

    if (bytes.startsWith(PNG_SIGNATURE)) return hasApngChunk(bytes)

    if (bytes.size >= 6) {
        val header = String(bytes, 0, 6, Charsets.US_ASCII)
        if (header == "GIF87a" || header == "GIF89a") return true
    }

    if (bytes.size >= 14 &&
        String(bytes, 0, 4, Charsets.US_ASCII) == "RIFF" &&
        String(bytes, 8, 4, Charsets.US_ASCII) == "WEBP"
    ) {
        return String(bytes, 12, 4, Charsets.US_ASCII) == "VP8X" && (bytes[20].toInt() and 0x02) != 0
    }
    return false
}

private fun hasApngChunk(bytes: ByteArray): Boolean {
    var pos = 8
    while (pos + 8 <= bytes.size) {
        val length = readIntBigEndian(bytes, pos)
        if (length < 0 || pos + 12L + length > bytes.size) return false
        val type = String(bytes, pos + 4, 4, Charsets.US_ASCII)
        if (type == "acTL") return true
        if (type == "IDAT") return false
        pos += 12 + length
    }
    return false
}

private fun readIntBigEndian(bytes: ByteArray, offset: Int): Int =
    ((bytes[offset].toInt() and 0xFF) shl 24) or
        ((bytes[offset + 1].toInt() and 0xFF) shl 16) or
        ((bytes[offset + 2].toInt() and 0xFF) shl 8) or
        (bytes[offset + 3].toInt() and 0xFF)

private fun ByteArray.startsWith(prefix: ByteArray): Boolean {
    if (size < prefix.size) return false
    for (index in prefix.indices) {
        if (this[index] != prefix[index]) return false
    }
    return true
}
