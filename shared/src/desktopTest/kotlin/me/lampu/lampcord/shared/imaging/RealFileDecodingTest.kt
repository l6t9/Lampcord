package me.lampu.lampcord.shared.imaging

import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import coil3.Image
import org.jetbrains.skia.Codec
import org.jetbrains.skia.Data
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class RealFileDecodingTest {

    private val red = 0xFFFF0000.toInt()
    private val green = 0xFF00FF00.toInt()
    private val blue = 0xFF0000FF.toInt()

    @Test
    fun `decodes an apng written by another encoder`() {
        val bytes = fixture("anim.png")
        val source = assertNotNull(ApngFrameSource.parse(bytes))

        assertEquals(16, source.width)
        assertEquals(16, source.height)
        assertEquals(3, source.frameCount)
        assertColor(source, 0, red)
        assertColor(source, 1, green)
        assertColor(source, 2, blue)
    }

    @Test
    fun `decodes an animated gif into a drawable target`() {
        val source = codecSource("anim.gif")
        assertEquals(3, source.frameCount)

        assertRenderedColor(source, 0, red, tolerance = 0)
        assertRenderedColor(source, 1, green, tolerance = 0)
        assertRenderedColor(source, 2, blue, tolerance = 0)
    }

    @Test
    fun `decodes an animated webp into a drawable target`() {
        val source = codecSource("anim.webp")
        assertEquals(3, source.frameCount)

        val tolerance = 12
        assertRenderedColor(source, 0, red, tolerance)
        assertRenderedColor(source, 1, green, tolerance)
        assertRenderedColor(source, 2, blue, tolerance)
    }

    @Test
    fun `draws an apng through the animated image`() {
        val source = assertNotNull(ApngFrameSource.parse(fixture("anim.png")))
        val target = drawFrame(SkiaAnimatedImage(source), source.width, source.height)

        assertColorEquals(red, centerOf(target, source.width, source.height), "apng draw", tolerance = 0)
    }

    @Test
    fun `draws an animated gif through the animated image`() {
        val source = codecSource("anim.gif")
        val target = drawFrame(SkiaAnimatedImage(source), source.width, source.height)
        assertColorEquals(red, centerOf(target, source.width, source.height), "gif draw", tolerance = 0)
    }

    @Test
    fun `paints an apng through coil's own painter`() {
        val source = assertNotNull(ApngFrameSource.parse(fixture("anim.png")))
        // Compose never calls Image.draw directly; it goes through Coil's asPainter, which

        val image: Image = SkiaAnimatedImage(source)
        assertEquals(source.width, image.width)
        assertEquals(source.height, image.height)

        val target = Bitmap()
        assertEquals(true, target.allocN32Pixels(source.width, source.height, true))
        image.draw(Canvas(target))
        assertColorEquals(red, centerOf(target, source.width, source.height), "apng painter", tolerance = 0)
    }

    private fun drawFrame(image: SkiaAnimatedImage, width: Int, height: Int): Bitmap {
        val target = Bitmap()
        assertEquals(true, target.allocN32Pixels(width, height, true))

        image.draw(Canvas(target))
        return target
    }

    private fun centerOf(bitmap: Bitmap, width: Int, height: Int): Int =
        assertNotNull(bitmap.peekPixels()).getColor(width / 2, height / 2)

    private fun codecSource(name: String): SkiaCodecFrameSource {
        val bytes = fixture(name)
        val codec = assertNotNull(Codec.makeFromData(Data.makeFromBytes(bytes)))
        return SkiaCodecFrameSource(codec, codec.width, codec.height)
    }

    private fun assertRenderedColor(
        source: SkiaCodecFrameSource,
        frame: Int,
        expected: Int,
        tolerance: Int
    ) {
        val target = Bitmap()
        assertEquals(true, target.allocN32Pixels(source.width, source.height, true))
        source.renderFrame(target, frame)

        val actual = assertNotNull(target.peekPixels()).getColor(source.width / 2, source.height / 2)
        assertColorEquals(expected, actual, "$frame", tolerance)
    }

    private fun assertColor(source: ApngFrameSource, frame: Int, expected: Int) {
        val image = assertNotNull(source.readyImage(frame))
        val actual = assertNotNull(image.peekPixels())
            .getColor(source.width / 2, source.height / 2)
        assertColorEquals(expected, actual, "frame $frame", tolerance = 0)
    }

    private fun assertColorEquals(expected: Int, actual: Int, label: String, tolerance: Int) {
        for (shift in listOf(0, 8, 16)) {
            val want = (expected ushr shift) and 0xFF
            val got = (actual ushr shift) and 0xFF
            assertTrue(
                kotlin.math.abs(want - got) <= tolerance,
                "$label channel $shift: expected $want, was $got"
            )
        }
    }

    private fun fixture(name: String): ByteArray =
        checkNotNull(javaClass.getResourceAsStream("/imaging/$name")) {
            "missing test fixture $name"
        }.use { it.readBytes() }
}
