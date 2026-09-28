package me.lampu.lampcord.shared.imaging

import coil3.ImageLoader
import coil3.PlatformContext
import coil3.asImage
import coil3.decode.DecodeResult
import coil3.decode.Decoder
import coil3.fetch.SourceFetchResult
import coil3.request.Options
import coil3.size.Dimension
import coil3.size.Precision
import coil3.size.Scale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okio.Buffer
import okio.BufferedSource
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.Image
import org.jetbrains.skia.Rect
import org.jetbrains.skia.SamplingMode

internal class DesktopStillDecoder(
    private val options: Options,
    private val bytes: ByteArray,
) : Decoder {

    override suspend fun decode(): DecodeResult = withContext(Dispatchers.IO) {
        val image = Image.makeFromEncoded(bytes) ?: error("Skia could not read the image")
        try {
            val srcWidth = image.width
            val srcHeight = image.height

            val (outWidth, outHeight) = scaledSize(srcWidth, srcHeight)
            val bitmap = resize(image, outWidth, outHeight)
            bitmap.setImmutable()

            DecodeResult(
                image = bitmap.asImage(),
                isSampled = bitmap.width < srcWidth || bitmap.height < srcHeight
            )
        } finally {
            image.close()
        }
    }

    private fun scaledSize(srcWidth: Int, srcHeight: Int): Pair<Int, Int> {
        val target = options.size
        val targetWidth = (target.width as? Dimension.Pixels)?.px ?: srcWidth
        val targetHeight = (target.height as? Dimension.Pixels)?.px ?: srcHeight

        var multiplier = when (options.scale) {
            Scale.FILL -> maxOf(
                targetWidth.toDouble() / srcWidth,
                targetHeight.toDouble() / srcHeight
            )

            else -> minOf(
                targetWidth.toDouble() / srcWidth,
                targetHeight.toDouble() / srcHeight
            )
        }

        if (options.precision == Precision.INEXACT) {
            multiplier = multiplier.coerceAtMost(1.0)
        }

        return (multiplier * srcWidth).toInt()
            .coerceIn(1, srcWidth) to (multiplier * srcHeight).toInt().coerceIn(1, srcHeight)
    }

    class Factory : Decoder.Factory {
        override fun create(
            result: SourceFetchResult,
            options: Options,
            imageLoader: ImageLoader
        ): Decoder? {
            val source = result.source.source() ?: return null

            val head = Buffer()
            source.peek().use { it.read(head, HEADER_BYTES) }
            if (!isSkiaStill(head.snapshot().toByteArray())) return null
            return DesktopStillDecoder(options, source.readByteArray())
        }
    }
}

private fun resize(image: Image, dstWidth: Int, dstHeight: Int): Bitmap {
    if (image.width == dstWidth && image.height == dstHeight) {
        val exact = Bitmap()
        if (exact.allocN32Pixels(dstWidth, dstHeight, false)) {
            Canvas(exact).drawImage(image, 0f, 0f)
        }
        return exact
    }

    val steps = mutableListOf<Image>()
    var current = image
    var width = image.width
    var height = image.height

    while (width > dstWidth * 2 || height > dstHeight * 2) {
        val nextWidth = maxOf(dstWidth, width / 2)
        val nextHeight = maxOf(dstHeight, height / 2)
        if (nextWidth >= width || nextHeight >= height) break

        val step = drawScaled(current, nextWidth, nextHeight, SamplingMode.CATMULL_ROM)
        steps += Image.makeFromBitmap(step)
        current = steps.last()
        width = nextWidth
        height = nextHeight
    }

    val result = drawScaled(current, dstWidth, dstHeight, SamplingMode.CATMULL_ROM)
    steps.forEach { it.close() }
    return result
}

private fun drawScaled(
    source: Image,
    dstWidth: Int,
    dstHeight: Int,
    sampling: SamplingMode
): Bitmap {
    val bitmap = Bitmap()
    if (!bitmap.allocN32Pixels(dstWidth, dstHeight, false)) return bitmap
    Canvas(bitmap).drawImageRect(
        source,
        Rect.makeWH(source.width.toFloat(), source.height.toFloat()),
        Rect.makeWH(dstWidth.toFloat(), dstHeight.toFloat()),
        sampling,
        null,
        false
    )
    return bitmap
}

private const val HEADER_BYTES = 16L

private fun isSkiaStill(bytes: ByteArray): Boolean {
    if (bytes.size >= 8 && PNG_SIGNATURE.indices.all { bytes[it] == PNG_SIGNATURE[it] }) return true
    if (bytes.size >= 3 && bytes[0] == 0xFF.toByte() && bytes[1] == 0xD8.toByte() && bytes[2] == 0xFF.toByte()) {
        return true
    }
    if (bytes.size >= 6) {
        when (String(bytes, 0, 6, Charsets.US_ASCII)) {
            "GIF87a", "GIF89a" -> return true
        }
    }
    if (bytes.size >= 12 &&
        String(bytes, 0, 4, Charsets.US_ASCII) == "RIFF" &&
        String(bytes, 8, 4, Charsets.US_ASCII) == "WEBP"
    ) {
        return true
    }
    if (bytes.size >= 2 && bytes[0] == 'B'.code.toByte() && bytes[1] == 'M'.code.toByte()) {
        return true
    }
    return false
}

private val PNG_SIGNATURE = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)
