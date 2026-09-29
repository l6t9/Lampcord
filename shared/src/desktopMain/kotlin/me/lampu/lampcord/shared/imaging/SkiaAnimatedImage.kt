package me.lampu.lampcord.shared.imaging

import coil3.Image
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.Color
import org.jetbrains.skia.FilterMipmap
import org.jetbrains.skia.FilterMode
import org.jetbrains.skia.MipmapMode
import org.jetbrains.skia.Paint
import org.jetbrains.skia.Rect
import org.jetbrains.skia.SamplingMode
import kotlin.time.TimeSource

internal interface FrameSource {
    val width: Int
    val height: Int

    val byteSize: Long

    val frameCount: Int

    fun frameIndexAt(elapsedMs: Long): Int

    fun renderFrame(target: Bitmap, frameIndex: Int)

    fun readyImage(frameIndex: Int): org.jetbrains.skia.Image? = null
}

private val MIPMAP_SAMPLING: SamplingMode = FilterMipmap(FilterMode.LINEAR, MipmapMode.LINEAR)

internal class SkiaAnimatedImage(
    private val frameSource: FrameSource,
    timeSource: TimeSource = TimeSource.Monotonic,
    private val animates: Boolean = true,
) : Image {

    private val startedAt = timeSource.markNow()
    private val buffers = arrayOfNulls<Bitmap>(2)
    private val paint = Paint().apply { isAntiAlias = true }

    private var activeSlot = 0
    private var lastFrame = -1
    private var cachedImage: org.jetbrains.skia.Image? = null
    private val lock = Any()

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

        val image = frameSource.readyImage(frame) ?: synchronized(lock) {
            if (frame == lastFrame) {
                cachedImage ?: org.jetbrains.skia.Image.makeFromBitmap(buffer(activeSlot))
                    .also { cachedImage = it }
            } else {

                val slot = 1 - activeSlot
                val target = buffer(slot)
                frameSource.renderFrame(target, frame)
                target.notifyPixelsChanged()
                activeSlot = slot
                lastFrame = frame
                org.jetbrains.skia.Image.makeFromBitmap(target).also { cachedImage = it }
            }
        }

        val bounds = Rect.makeWH(width.toFloat(), height.toFloat())
        canvas.drawImageRect(image, bounds, bounds, MIPMAP_SAMPLING, paint, false)
    }
}
