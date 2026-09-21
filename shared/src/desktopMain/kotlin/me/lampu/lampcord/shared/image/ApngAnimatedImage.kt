package me.lampu.lampcord.shared.image

import com.github.panpf.sketch.AnimatedImage
import com.github.panpf.sketch.Bitmap
import com.github.panpf.sketch.createBitmap
import com.github.panpf.sketch.util.Rect
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import javax.imageio.ImageIO
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType

/**
 * [AnimatedImage] implementation that composites the raw APNG frame rectangles into full canvases
 * using the APNG dispose/alpha-blend semantics, then hands each frame to Sketch as a raster image.
 *
 * Frame rectangles are decoded lazily with the ImageIO reader only when the animation first plays.
 */
class ApngAnimatedImage(
    private val bytes: ByteArray,
    private val info: ApngInfo,
) : AnimatedImage {

    override val width: Int = info.width
    override val height: Int = info.height

    override val frameCount: Int = info.numFrames

    override val repeatCount: Int = if (info.numPlays == 0) -1 else (info.numPlays - 1)

    override val frameDurations: Array<Int> =
        info.frames.map { delayMs(it.delayNum, it.delayDen) }.toTypedArray()

    override val cacheDecodeTimeoutFrame: Boolean = false

    override val byteCount: Long = width.toLong() * height * 4

    override val shareable: Boolean = false

    override var animatedTransformation: ((Any, Rect) -> Unit)? = null
    override var animationStartCallback: (() -> Unit)? = null
    override var animationEndCallback: (() -> Unit)? = null

    private val frames: Array<IntArray> by lazy { compositeFrames() }

    override fun checkValid(): Boolean = true

    override fun equals(other: Any?): Boolean = this === other

    override fun hashCode(): Int = System.identityHashCode(this)

    override fun createFrameBitmap(width: Int, height: Int): Bitmap = createBitmap(width, height)

    override fun readFrame(bitmap: Bitmap, frameIndex: Int) {
        val argb = frames[frameIndex]
        val rgba = ByteArray(argb.size * 4)
        var i = 0
        for (p in argb) {
            val a = (p ushr 24) and 0xFF
            val r = (p ushr 16) and 0xFF
            val g = (p ushr 8) and 0xFF
            val b = p and 0xFF
            val ra = r * a / 255
            val ga = g * a / 255
            val ba = b * a / 255
            rgba[i] = ra.toByte()
            rgba[i + 1] = ga.toByte()
            rgba[i + 2] = ba.toByte()
            rgba[i + 3] = a.toByte()
            i += 4
        }
        val skiaInfo = org.jetbrains.skia.ImageInfo(
            width, height, ColorType.RGBA_8888, ColorAlphaType.PREMUL, null
        )
        val raster = org.jetbrains.skia.Image.makeRaster(skiaInfo, rgba, width * 4)
        try {
            Canvas(bitmap).drawImage(raster, 0.0f, 0.0f, null)
        } finally {
            raster.close()
        }
    }

    private fun compositeFrames(): Array<IntArray> {
        val reader = findReader() ?: error("No APNG-capable ImageIO reader registered")
        return try {
            reader.setInput(ImageIO.createImageInputStream(ByteArrayInputStream(bytes)), true, true)
            val frames = Array(info.numFrames) { IntArray(0) }
            var canvas = IntArray(width * height)
            var previous = IntArray(0)
            for (i in 0 until info.numFrames) {
                if (i > 0) {
                    when (info.frames[i - 1].disposeOp) {
                        1 -> clearRegion(canvas, info.frames[i - 1])
                        2 -> canvas = previous.copyOf()
                    }
                }
                previous = canvas.copyOf()
                val frame = reader.read(i)
                blit(canvas, info.frames[i], frame)
                frames[i] = canvas.copyOf()
            }
            frames
        } finally {
            reader.dispose()
        }
    }

    private fun findReader(): javax.imageio.ImageReader? {
        val stream = ImageIO.createImageInputStream(ByteArrayInputStream(bytes)) ?: return null
        return ImageIO.getImageReaders(stream)
            .asSequence()
            .firstOrNull { reader ->
                reader.originatingProvider
                    ?.getDescription(null)
                    ?.contains("APNG", ignoreCase = true) == true
            }
    }

    private fun blit(canvas: IntArray, meta: ApngFrameMeta, img: BufferedImage) {
        val srcW = img.width.coerceAtMost(meta.width)
        val srcH = img.height.coerceAtMost(meta.height)
        for (j in 0 until srcH) {
            val cy = meta.y + j
            if (cy < 0 || cy >= height) continue
            var dst = cy * width + meta.x
            for (k in 0 until srcW) {
                val cx = meta.x + k
                if (cx < 0 || cx >= width) {
                    dst++
                    continue
                }
                canvas[dst] = if (meta.blendOp == 0) {
                    img.getRGB(k, j)
                } else {
                    compositeOver(canvas[dst], img.getRGB(k, j))
                }
                dst++
            }
        }
    }

    private fun compositeOver(dst: Int, src: Int): Int {
        val sa = (src ushr 24) and 0xFF
        if (sa == 0xFF) return src
        if (sa == 0) return dst
        val da = (dst ushr 24) and 0xFF
        val srcA = sa / 255f
        val dstA = da / 255f
        val outA = srcA + dstA * (1f - srcA)
        if (outA <= 0f) return 0
        val sr = (src ushr 16) and 0xFF
        val sg = (src ushr 8) and 0xFF
        val sb = src and 0xFF
        val dr = (dst ushr 16) and 0xFF
        val dg = (dst ushr 8) and 0xFF
        val db = dst and 0xFF
        val factor = dstA * (1f - srcA)
        val outR = ((sr * srcA) + (dr * factor)) / outA
        val outG = ((sg * srcA) + (dg * factor)) / outA
        val outB = ((sb * srcA) + (db * factor)) / outA
        return ((outA * 255f).toInt() shl 24) or
                (outR.toInt() shl 16) or
                (outG.toInt() shl 8) or
                outB.toInt()
    }

    private fun clearRegion(canvas: IntArray, meta: ApngFrameMeta) {
        val startX = maxOf(0, meta.x)
        val endX = minOf(width, meta.x + meta.width)
        if (endX <= startX) return
        for (j in 0 until meta.height) {
            val cy = meta.y + j
            if (cy < 0 || cy >= height) continue
            val start = cy * width + startX
            for (k in startX until endX) {
                canvas[start + (k - startX)] = 0
            }
        }
    }

    private fun delayMs(num: Int, den: Int): Int {
        if (num == 0) return 100
        val d = if (den == 0) 100 else den
        return (num * 1000L / d).toInt().coerceAtLeast(10)
    }

    override fun toString(): String = "ApngAnimatedImage(" +
            "width=$width, height=$height, frameCount=$frameCount, repeatCount=$repeatCount)"
}