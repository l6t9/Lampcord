package me.lampu.lampcord.shared.imaging

import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Codec

internal class SkiaCodecFrameSource(
    private val codec: Codec,
    override val width: Int,
    override val height: Int,
) : FrameSource {

    private val durations: IntArray = IntArray(codec.frameCount) { index ->

        codec.getFrameInfo(index).duration.takeIf { it > 0 } ?: DEFAULT_FRAME_DURATION_MS
    }

    private val cumulative: LongArray = LongArray(durations.size + 1)
    private val totalDurationMs: Long

    init {
        for (index in durations.indices) {
            cumulative[index + 1] = cumulative[index] + durations[index]
        }
        totalDurationMs = cumulative.last().coerceAtLeast(1L)
    }

    override val frameCount: Int get() = durations.size

    override val byteSize: Long = 4L * width * height

    override fun frameIndexAt(elapsedMs: Long): Int {
        if (durations.size <= 1) return 0
        var time = elapsedMs % totalDurationMs

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
        target.notifyPixelsChanged()
    }

    private companion object {
        const val DEFAULT_FRAME_DURATION_MS = 100
    }
}
