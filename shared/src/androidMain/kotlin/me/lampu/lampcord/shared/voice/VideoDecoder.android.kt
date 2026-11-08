package me.lampu.lampcord.shared.voice

import android.graphics.Bitmap
import android.media.Image
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Build
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap

internal actual fun createVideoDecoder(codec: VideoCodec): VideoDecoder = AndroidVideoDecoder(codec)

private class AndroidVideoDecoder(override val codec: VideoCodec) : VideoDecoder {
    private val decoder: MediaCodec
    private val info = MediaCodec.BufferInfo()
    private var presentation = 0L
    private var luma = ByteArray(0)
    private var blue = ByteArray(0)
    private var red = ByteArray(0)
    private var argb = IntArray(0)
    // YUV to RGB coefficients, scaled by 256, for the stream's color standard and range.
    private var lumaScale = 298
    private var lumaOffset = 16
    private var redV = 409
    private var greenU = 100
    private var greenV = 208
    private var blueU = 516
    // Frames are drawn by Compose after we hand them over, so overwrite only the oldest of a few.
    private val bitmaps = ArrayDeque<Bitmap>()

    init {
        val mime = if (codec == VideoCodec.H264) MediaFormat.MIMETYPE_VIDEO_AVC else MediaFormat.MIMETYPE_VIDEO_VP8
        decoder = MediaCodec.createDecoderByType(mime)
        try {
            val format = MediaFormat.createVideoFormat(mime, 1280, 720)
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible)
            if (Build.VERSION.SDK_INT >= 30) format.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
            decoder.configure(format, null, null, 0)
            decoder.start()
        } catch (e: Exception) {
            decoder.release()
            throw e
        }
    }

    override fun decode(frame: ByteArray): ImageBitmap? {
        val input = decoder.dequeueInputBuffer(50_000)
        check(input >= 0) { "Video decoder is not accepting input" }
        val buffer = checkNotNull(decoder.getInputBuffer(input))
        buffer.clear()
        check(buffer.capacity() >= frame.size) { "Video frame exceeds decoder buffer" }
        buffer.put(frame)
        decoder.queueInputBuffer(input, 0, frame.size, presentation, 0)
        presentation += 33_333
        var result: ImageBitmap? = null
        while (true) {
            val output = decoder.dequeueOutputBuffer(info, if (result == null) 10_000 else 0)
            if (output == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED) {
                applyColors(decoder.outputFormat)
                continue
            }
            if (output < 0) break
            try {
                decoder.getOutputImage(output)?.use { result = convert(it) ?: result }
            } finally {
                decoder.releaseOutputBuffer(output, false)
            }
        }
        return result
    }

    private fun applyColors(format: MediaFormat) {
        fun integer(key: String) = if (format.containsKey(key)) format.getInteger(key) else 0
        val bt709 = integer(MediaFormat.KEY_COLOR_STANDARD) == MediaFormat.COLOR_STANDARD_BT709
        val full = integer(MediaFormat.KEY_COLOR_RANGE) == MediaFormat.COLOR_RANGE_FULL
        lumaScale = if (full) 256 else 298
        lumaOffset = if (full) 0 else 16
        val coefficients = when {
            bt709 && full -> intArrayOf(403, 48, 120, 475)
            bt709 -> intArrayOf(459, 55, 136, 541)
            full -> intArrayOf(359, 88, 183, 454)
            else -> intArrayOf(409, 100, 208, 516)
        }
        redV = coefficients[0]; greenU = coefficients[1]; greenV = coefficients[2]; blueU = coefficients[3]
    }

    private fun convert(image: Image): ImageBitmap? {
        val crop = image.cropRect
        val width = crop.width()
        val height = crop.height()
        if (width <= 0 || height <= 0 || image.planes.size < 3) return null
        val (yPlane, uPlane, vPlane) = image.planes
        luma = copy(yPlane.buffer, luma)
        blue = copy(uPlane.buffer, blue)
        red = copy(vPlane.buffer, red)
        if (argb.size != width * height) argb = IntArray(width * height)
        val yRow = yPlane.rowStride
        val yPixel = yPlane.pixelStride
        val uvRow = uPlane.rowStride
        val uvPixel = uPlane.pixelStride
        var index = 0
        for (row in 0 until height) {
            val y = row + crop.top
            val yBase = y * yRow
            val uvBase = (y shr 1) * uvRow
            for (column in 0 until width) {
                val x = column + crop.left
                val c = ((luma[yBase + x * yPixel].toInt() and 255) - lumaOffset).coerceAtLeast(0) * lumaScale
                val uv = uvBase + (x shr 1) * uvPixel
                val d = (blue[uv].toInt() and 255) - 128
                val e = (red[uv].toInt() and 255) - 128
                val r = ((c + redV * e + 128) shr 8).coerceIn(0, 255)
                val g = ((c - greenU * d - greenV * e + 128) shr 8).coerceIn(0, 255)
                val b = ((c + blueU * d + 128) shr 8).coerceIn(0, 255)
                argb[index++] = (0xff shl 24) or (r shl 16) or (g shl 8) or b
            }
        }
        val bitmap = (if (bitmaps.size >= POOL_SIZE) bitmaps.removeFirst() else null)
            ?.takeIf { it.width == width && it.height == height }
            ?: Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        bitmap.setPixels(argb, 0, width, 0, 0, width, height)
        bitmaps.addLast(bitmap)
        return bitmap.asImageBitmap()
    }

    private fun copy(source: java.nio.ByteBuffer, target: ByteArray): ByteArray {
        val bytes = if (target.size == source.remaining()) target else ByteArray(source.remaining())
        source.get(bytes)
        return bytes
    }

    override fun close() {
        runCatching { decoder.stop() }
        decoder.release()
        bitmaps.clear()
    }

    private companion object {
        const val POOL_SIZE = 3
    }
}
