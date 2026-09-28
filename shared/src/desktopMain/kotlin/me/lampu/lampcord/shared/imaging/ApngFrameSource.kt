package me.lampu.lampcord.shared.imaging

import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import org.jetbrains.skia.ClipMode
import org.jetbrains.skia.Color
import org.jetbrains.skia.Image
import org.jetbrains.skia.Rect
import org.jetbrains.skia.SamplingMode
import java.io.ByteArrayOutputStream
import java.util.zip.CRC32

internal class ApngFrameSource private constructor(
    override val width: Int,
    override val height: Int,
    private val frames: List<Image>,
    private val durations: IntArray,
) : FrameSource {

    private val cumulative = LongArray(frames.size + 1)
    private val totalDurationMs: Long

    init {
        for (index in durations.indices) {
            cumulative[index + 1] = cumulative[index] + durations[index]
        }
        totalDurationMs = cumulative.last().coerceAtLeast(1L)
    }

    override val frameCount: Int get() = frames.size

    override val byteSize: Long = 4L * width * height * frames.size

    override fun frameIndexAt(elapsedMs: Long): Int {
        if (frames.size <= 1) return 0
        val time = elapsedMs % totalDurationMs
        var low = 0
        var high = frames.size - 1
        while (low < high) {
            val mid = (low + high + 1) / 2
            if (cumulative[mid] <= time) low = mid else high = mid - 1
        }
        return low
    }

    override fun readyImage(frameIndex: Int): Image = frames[frameIndex]

    override fun renderFrame(target: Bitmap, frameIndex: Int) {
        Canvas(target).drawImage(frames[frameIndex], 0f, 0f)
    }

    internal companion object {
        private val SIGNATURE = byteArrayOf(
            0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A
        )

        private const val DISPOSE_BACKGROUND = 1
        private const val DISPOSE_PREVIOUS = 2
        private const val BLEND_SOURCE = 0

        private const val MAX_COMPOSITED_PIXELS = 24_000_000L

        fun parse(bytes: ByteArray): ApngFrameSource? {
            if (bytes.size < 8) return null
            for (index in SIGNATURE.indices) {
                if (bytes[index] != SIGNATURE[index]) return null
            }

            var width = 0
            var height = 0
            var ihdrData: ByteArray? = null
            val controls = mutableListOf<IntArray>()
            val frameData = mutableListOf<MutableList<ByteArray>>()

            var sawAcTL = false
            var seenFirstIdat = false

            var palette: ByteArray? = null
            var transparency: ByteArray? = null
            var pos = 8

            while (pos + 8 <= bytes.size) {
                val length = readInt(bytes, pos)
                if (length < 0 || pos + 12L + length > bytes.size) break
                val type = String(bytes, pos + 4, 4, Charsets.US_ASCII)
                val dataStart = pos + 8
                val dataEnd = dataStart + length
                val chunkEnd = dataEnd + 4

                when (type) {
                    "IHDR" -> {
                        width = readInt(bytes, dataStart)
                        height = readInt(bytes, dataStart + 4)
                        ihdrData = bytes.copyOfRange(dataStart, dataEnd)
                    }

                    "acTL" -> sawAcTL = true

                    "PLTE" -> palette = bytes.copyOfRange(dataStart, dataEnd)

                    "tRNS" -> transparency = bytes.copyOfRange(dataStart, dataEnd)

                    "fcTL" -> {

                        controls += intArrayOf(
                            readInt(bytes, dataStart + 4),
                            readInt(bytes, dataStart + 8),
                            readInt(bytes, dataStart + 12),
                            readInt(bytes, dataStart + 16),
                            readShort(bytes, dataStart + 20),
                            readShort(bytes, dataStart + 22),
                            bytes[dataStart + 24].toInt(),
                            bytes[dataStart + 25].toInt()
                        )
                        frameData.add(mutableListOf())
                    }

                    "IDAT" -> {
                        seenFirstIdat = true

                        frameData.firstOrNull()?.add(bytes.copyOfRange(dataStart, dataEnd))
                    }

                    "fdAT" -> if (length > 4) {

                        frameData.lastOrNull()?.add(bytes.copyOfRange(dataStart + 4, dataEnd))
                    }
                }

                pos = chunkEnd
            }

            if (!sawAcTL || ihdrData == null || width <= 0 || height <= 0) return null
            if (controls.isEmpty()) return null

            val ihdrTail = ihdrData.copyOfRange(8, 13)
            val frameStreams = mutableListOf<ByteArray>()
            val frameControls = mutableListOf<IntArray>()

            for (index in controls.indices) {
                val control = controls[index]
                val chunks = frameData.getOrNull(index).orEmpty()
                if (chunks.isEmpty()) continue
                val data = if (chunks.size == 1) chunks[0] else chunks.reduce { a, b -> a + b }
                frameStreams += buildPng(ihdrTail, control[0], control[1], data, palette, transparency)
                frameControls += control
            }

            if (frameStreams.isEmpty()) return null

            val keep = if (frameStreams.size.toLong() * width * height <= MAX_COMPOSITED_PIXELS) {
                frameStreams.size
            } else {
                1
            }

            val canvasBitmap = Bitmap()

            if (!canvasBitmap.allocN32Pixels(width, height, false)) return null
            val canvas = Canvas(canvasBitmap)

            canvas.clear(Color.TRANSPARENT)
            val composited = ArrayList<Image>(keep)
            val durations = IntArray(keep)
            var previous: Bitmap? = null

            for (index in 0 until keep) {
                val control = frameControls[index]
                val frameWidth = control[0]
                val frameHeight = control[1]
                val region = Rect.makeXYWH(
                    control[2].toFloat(), control[3].toFloat(),
                    frameWidth.toFloat(), frameHeight.toFloat()
                )

                if (control[6] == DISPOSE_PREVIOUS) previous = copyOf(canvasBitmap, width, height)

                if (control[7] == BLEND_SOURCE) {
                    canvas.save()
                    canvas.clipRect(region, ClipMode.INTERSECT, true)
                    canvas.clear(Color.TRANSPARENT)
                    canvas.restore()
                }

                val frameImage = Image.makeFromEncoded(frameStreams[index]) ?: break
                if (frameImage.width != frameWidth || frameImage.height != frameHeight) {

                    break
                }
                canvas.drawImageRect(
                    frameImage,
                    Rect.makeWH(frameWidth.toFloat(), frameHeight.toFloat()),
                    region,
                    SamplingMode.LINEAR,
                    null,
                    false
                )

                composited += Image.makeFromBitmap(copyOf(canvasBitmap, width, height))
                durations[index] = delayMillis(control[4], control[5])

                when (control[6]) {
                    DISPOSE_BACKGROUND -> {
                        canvas.save()
                        canvas.clipRect(region, ClipMode.INTERSECT, true)
                        canvas.clear(Color.TRANSPARENT)
                        canvas.restore()
                    }

                    DISPOSE_PREVIOUS -> previous?.let { saved ->
                        canvas.clear(Color.TRANSPARENT)
                        canvas.drawImage(Image.makeFromBitmap(saved), 0f, 0f)
                    }
                }
            }

            if (composited.isEmpty()) return null
            return ApngFrameSource(width, height, composited, durations)
        }

        private fun buildPng(
            ihdrTail: ByteArray,
            frameWidth: Int,
            frameHeight: Int,
            data: ByteArray,
            palette: ByteArray?,
            transparency: ByteArray?
        ): ByteArray {
            val out = ByteArrayOutputStream()
            out.writeBytes(SIGNATURE)
            out.writeBytes(chunk("IHDR", int32(frameWidth) + int32(frameHeight) + ihdrTail))

            palette?.let { out.writeBytes(chunk("PLTE", it)) }
            transparency?.let { out.writeBytes(chunk("tRNS", it)) }
            out.writeBytes(chunk("IDAT", data))
            out.writeBytes(chunk("IEND", ByteArray(0)))
            return out.toByteArray()
        }

        private fun chunk(type: String, data: ByteArray): ByteArray {
            val body = type.toByteArray(Charsets.US_ASCII) + data
            return int32(data.size) + body + int32(CRC32().apply { update(body) }.value.toInt())
        }

        private fun int32(value: Int) = byteArrayOf(
            (value ushr 24 and 0xFF).toByte(),
            (value ushr 16 and 0xFF).toByte(),
            (value ushr 8 and 0xFF).toByte(),
            (value and 0xFF).toByte()
        )

        private fun delayMillis(numerator: Int, denominator: Int): Int {
            if (numerator <= 0) return DEFAULT_DELAY_MS
            val den = if (denominator == 0) 100 else denominator
            return (numerator * 1000 / den).coerceIn(MIN_DELAY_MS, 10_000)
        }

        private fun copyOf(source: Bitmap, width: Int, height: Int): Bitmap {
            val copy = Bitmap()
            if (!copy.allocN32Pixels(width, height, false)) return copy
            Canvas(copy).apply {

                clear(Color.TRANSPARENT)
                drawImage(Image.makeFromBitmap(source), 0f, 0f)
            }
            return copy
        }

        private fun readInt(bytes: ByteArray, offset: Int): Int =
            ((bytes[offset].toInt() and 0xFF) shl 24) or
                ((bytes[offset + 1].toInt() and 0xFF) shl 16) or
                ((bytes[offset + 2].toInt() and 0xFF) shl 8) or
                (bytes[offset + 3].toInt() and 0xFF)

        private fun readShort(bytes: ByteArray, offset: Int): Int =
            ((bytes[offset].toInt() and 0xFF) shl 8) or (bytes[offset + 1].toInt() and 0xFF)

        private const val DEFAULT_DELAY_MS = 100
        private const val MIN_DELAY_MS = 10
    }
}
