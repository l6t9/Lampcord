package me.lampu.lampcord.shared.imaging

import org.jetbrains.skia.Color
import java.io.ByteArrayOutputStream
import java.util.zip.CRC32
import java.util.zip.Deflater
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ApngFrameSourceTest {

    private val red = Color.makeARGB(255, 255, 0, 0)
    private val green = Color.makeARGB(255, 0, 255, 0)
    private val blue = Color.makeARGB(255, 0, 0, 255)

    @Test
    fun `parses frames and durations when the first control precedes IDAT`() {
        val bytes = buildApng(
            frames = listOf(red, green, blue),
            firstControlBeforeIdat = true
        )

        val source = assertNotNull(ApngFrameSource.parse(bytes))
        assertEquals(4, source.width)
        assertEquals(4, source.height)
        assertEquals(3, source.frameCount)
        assertColor(source, 0, red)
        assertColor(source, 1, green)
        assertColor(source, 2, blue)
    }

    @Test
    fun `picks frames from the correct point in the timeline`() {
        val bytes = buildApng(
            frames = listOf(red, green, blue),
            firstControlBeforeIdat = true,
            delays = listOf(100, 200, 300)
        )

        val source = assertNotNull(ApngFrameSource.parse(bytes))
        assertEquals(0, source.frameIndexAt(0))
        assertEquals(0, source.frameIndexAt(99))
        assertEquals(1, source.frameIndexAt(100))
        assertEquals(1, source.frameIndexAt(299))
        assertEquals(2, source.frameIndexAt(300))

        assertEquals(0, source.frameIndexAt(600))
    }

    @Test
    fun `parses frames when the default image is not part of the animation`() {
        val bytes = buildApng(
            frames = listOf(red, green, blue),
            firstControlBeforeIdat = false
        )

        val source = assertNotNull(ApngFrameSource.parse(bytes))
        assertEquals(3, source.frameCount)
        assertColor(source, 0, red)
        assertColor(source, 2, blue)
    }

    @Test
    fun `rejects a plain png that has no animation`() {
        val plain = buildPng(4, 4, red)
        assertNull(ApngFrameSource.parse(plain))
    }

    @Test
    fun `rejects data that is not a png at all`() {
        assertNull(ApngFrameSource.parse("GIF89a not really".toByteArray()))
    }

    @Test
    fun `falls back to a usable delay when a frame has none`() {
        val bytes = buildApng(
            frames = listOf(red, green),
            firstControlBeforeIdat = true,
            delays = listOf(0, 0)
        )

        val source = assertNotNull(ApngFrameSource.parse(bytes))

        assertEquals(0, source.frameIndexAt(50))
        assertEquals(1, source.frameIndexAt(150))
    }

    private fun assertColor(source: ApngFrameSource, frame: Int, expected: Int) {
        val image = assertNotNull(source.readyImage(frame))
        val pixmap = assertNotNull(image.peekPixels())
        val actual = pixmap.getColor(2, 2)
        assertEquals(
            (expected.toLong() and 0xFFFFFFFFL),
            (actual.toLong() and 0xFFFFFFFFL),
            "frame $frame"
        )
    }

    private fun buildApng(
        frames: List<Int>,
        firstControlBeforeIdat: Boolean,
        delays: List<Int> = frames.indices.map { 100 }
    ): ByteArray {
        val out = ByteArrayOutputStream()
        out.writeBytes(PNG_SIGNATURE)
        out.writeBytes(chunk("IHDR", ihdr(4, 4)))
        out.writeBytes(chunk("acTL", int32(frames.size) + int32(0)))

        var sequence = 0
        if (firstControlBeforeIdat) {
            out.writeBytes(chunk("fcTL", fcTL(sequence++, delays[0])))
            out.writeBytes(idat(4, 4, frames[0]))
            out.writeBytes(chunk("IEND", ByteArray(0)))
        } else {

            out.writeBytes(idat(4, 4, frames[0]))
            out.writeBytes(chunk("IEND", ByteArray(0)))
        }

        for (index in (if (firstControlBeforeIdat) 1 else 0) until frames.size) {
            out.writeBytes(chunk("fcTL", fcTL(sequence++, delays[index])))

            out.writeBytes(chunk("fdAT", int32(sequence++) + scanlines(4, 4, frames[index])))
        }
        return out.toByteArray()
    }

    private fun fcTL(sequence: Int, delayMs: Int): ByteArray =
        int32(sequence) +
            int32(4) + int32(4) +
            int32(0) + int32(0) +
            short16(delayMs) + short16(1000) +
            byteArrayOf(0, 0)

    private fun buildPng(width: Int, height: Int, color: Int): ByteArray {
        val out = ByteArrayOutputStream()
        out.writeBytes(PNG_SIGNATURE)
        out.writeBytes(chunk("IHDR", ihdr(width, height)))
        out.writeBytes(idat(width, height, color))
        out.writeBytes(chunk("IEND", ByteArray(0)))
        return out.toByteArray()
    }

    private fun idat(width: Int, height: Int, color: Int): ByteArray =
        chunk("IDAT", scanlines(width, height, color))

    private fun scanlines(width: Int, height: Int, color: Int): ByteArray {
        val raw = ByteArrayOutputStream()
        val pixel = byteArrayOf(
            (color ushr 16 and 0xFF).toByte(),
            (color ushr 8 and 0xFF).toByte(),
            (color and 0xFF).toByte(),
            (color ushr 24 and 0xFF).toByte()
        )
        repeat(height) {
            raw.write(0)
            repeat(width) { raw.writeBytes(pixel) }
        }

        val deflater = Deflater()
        deflater.setInput(raw.toByteArray())
        deflater.finish()
        val compressed = ByteArrayOutputStream()
        val buffer = ByteArray(1024)
        while (!deflater.finished()) {
            compressed.write(buffer, 0, deflater.deflate(buffer))
        }
        deflater.end()
        return compressed.toByteArray()
    }

    private fun ihdr(width: Int, height: Int): ByteArray =
        int32(width) + int32(height) +
            byteArrayOf(8, 6, 0, 0, 0)

    private fun chunk(type: String, data: ByteArray): ByteArray {
        val body = type.toByteArray(Charsets.US_ASCII) + data
        val crc = CRC32().apply { update(body) }.value
        return int32(data.size) + body + int32(crc.toInt())
    }

    private fun int32(value: Int) = byteArrayOf(
        (value ushr 24 and 0xFF).toByte(),
        (value ushr 16 and 0xFF).toByte(),
        (value ushr 8 and 0xFF).toByte(),
        (value and 0xFF).toByte()
    )

    private fun short16(value: Int) = byteArrayOf(
        (value ushr 8 and 0xFF).toByte(),
        (value and 0xFF).toByte()
    )

    private companion object {
        val PNG_SIGNATURE = byteArrayOf(
            0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A
        )
    }
}
