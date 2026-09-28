package me.lampu.lampcord.shared.image

import com.github.panpf.sketch.AnimatedImage
import com.github.panpf.sketch.Bitmap
import com.github.panpf.sketch.createBitmap
import com.github.panpf.sketch.util.Rect
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import javax.imageio.ImageIO
import javax.imageio.ImageReadParam
import kotlin.math.max
import kotlin.math.roundToInt
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType

// [AnimatedImage] implementation that composites the raw APNG frame rectangles into full canvases using the APNG dispose/alpha-blend semantics, then hands each frame to Sketch as a raster image.
private const val MAX_DECODE_DIMENSION = 512

class ApngAnimatedImage(
    private val bytes: ByteArray,
    private val info: ApngInfo,
) : AnimatedImage {

    override val width: Int
    override val height: Int

    private val sampleX: Int
    private val sampleY: Int

    init {
        val maxDim = max(info.width, info.height)
        val scale = if (maxDim > MAX_DECODE_DIMENSION) MAX_DECODE_DIMENSION.toFloat() / maxDim else 1f
        val dw = max(1, (info.width * scale).roundToInt())
        val dh = max(1, (info.height * scale).roundToInt())
        sampleX = max(1, ceilDiv(info.width, dw))
        sampleY = max(1, ceilDiv(info.height, dh))
        width = ceilDiv(info.width, sampleX)
        height = ceilDiv(info.height, sampleY)
    }

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

    // Frame 0 is decoded on its own so that a static render (disallowAnimatedImage still asks for the first frame) never materializes the whole animation.
    private val frame0: IntArray by lazy { decodeFrame0() }

    private val frames: Array<IntArray> by lazy { compositeFrames() }

    override fun checkValid(): Boolean = true

    override fun equals(other: Any?): Boolean = this === other

    override fun hashCode(): Int = System.identityHashCode(this)

    override fun createFrameBitmap(width: Int, height: Int): Bitmap = createBitmap(width, height)

    override fun readFrame(bitmap: Bitmap, frameIndex: Int) {
        val argb = if (frameIndex == 0) frame0 else frames[frameIndex]
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

private fun decodeFrame0(): IntArray {
        val reader = findReader() ?: return IntArray(width * height)
        return try {
            reader.setInput(ImageIO.createImageInputStream(ByteArrayInputStream(bytes)), true, true)
            val out = IntArray(width * height)
            blit(out, info.frames[0], reader.read(0, readParam(reader)))
            out
        } catch (_: javax.imageio.IIOException) {
            IntArray(width * height)
        } finally {
            reader.dispose()
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
                blit(canvas, info.frames[i], reader.read(i, readParam(reader)))
                frames[i] = canvas.copyOf()
            }
            frames
        } catch (_: javax.imageio.IIOException) {
            // A truncated/partial download shouldn't crash the list; fall back to transparent frames like the frame-0 path does.
            Array(info.numFrames) { IntArray(width * height) }
        } finally {
            reader.dispose()
        }
    }

    private fun readParam(reader: javax.imageio.ImageReader): ImageReadParam =
        reader.defaultReadParam.apply { setSourceSubsampling(sampleX, sampleY, 0, 0) }

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

    private fun scaleCoord(v: Int, isX: Boolean): Int {
        val factor = if (isX) width.toFloat() / info.width else height.toFloat() / info.height
        return (v * factor).roundToInt()
    }

    private fun blit(canvas: IntArray, meta: ApngFrameMeta, img: BufferedImage) {
        val x0 = maxOf(0, scaleCoord(meta.x, true))
        for (j in 0 until img.height) {
            val cy = scaleCoord(meta.y + j * sampleY, isX = false)
            if (cy < 0 || cy >= height) continue
            var dst = cy * width + x0
            for (k in 0 until img.width) {
                val cx = x0 + scaleCoord(k * sampleX, isX = true)
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
        val x0 = maxOf(0, scaleCoord(meta.x, true))
        val x1 = minOf(width, scaleCoord(meta.x + meta.width, true))
        if (x1 <= x0) return
        for (j in 0 until meta.height) {
            val cy = scaleCoord(meta.y + j, isX = false)
            if (cy < 0 || cy >= height) continue
            val start = cy * width + x0
            for (k in x0 until x1) {
                canvas[start + (k - x0)] = 0
            }
        }
    }

    private fun ceilDiv(a: Int, b: Int): Int = (a + b - 1) / b

    private fun delayMs(num: Int, den: Int): Int {
        if (num == 0) return 100
        val d = if (den == 0) 100 else den
        return (num * 1000L / d).toInt().coerceAtLeast(10)
    }

    override fun toString(): String = "ApngAnimatedImage(" +
            "width=$width, height=$height, decode=${MAX_DECODE_DIMENSION}px cap, frameCount=$frameCount, repeatCount=$repeatCount)"
}