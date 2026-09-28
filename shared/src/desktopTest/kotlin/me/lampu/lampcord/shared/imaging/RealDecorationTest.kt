package me.lampu.lampcord.shared.imaging

import kotlinx.coroutines.runBlocking
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class RealDecorationTest {

    @Test
    fun `a real decoration decodes to a non-blank frame`() {
        val bytes = fixture("deco_a_509a32a45886043d8a9c67474de95d2f.png")
        val source = ApngFrameSource.parse(bytes)
        assertNotNull(source, "failed to parse a real decoration")
        assertTrue(source.frameCount > 1, "decoration should be animated, got ${source.frameCount}")
        println("decoration ${source.width}x${source.height} frames=${source.frameCount}")

        val target = Bitmap()
        assertTrue(target.allocN32Pixels(source.width, source.height, true))
        SkiaAnimatedImage(source).draw(Canvas(target))

        val pixels = assertNotNull(target.peekPixels())
        var visible = 0
        var opaque = 0
        for (y in 0 until source.height) {
            for (x in 0 until source.width) {
                val color = pixels.getColor(x, y)
                if ((color ushr 24) != 0) {
                    opaque++
                    if ((color and 0xFFFFFF) != 0) visible++
                }
            }
        }
        val total = source.width * source.height
        println("opaque=$opaque visible=$visible of $total")
        assertTrue(opaque > total / 100, "frame 0 is empty: only $opaque of $total opaque")
        assertTrue(visible > total / 100, "frame 0 is fully transparent: $visible of $total")
    }

    @Test
    fun `a real decoration keeps its transparency`() {
        val source = assertNotNull(
            ApngFrameSource.parse(fixture("deco_a_509a32a45886043d8a9c67474de95d2f.png"))
        )
        val target = Bitmap()
        assertTrue(target.allocN32Pixels(source.width, source.height, false))
        SkiaAnimatedImage(source).draw(Canvas(target))

        val pixels = assertNotNull(target.peekPixels())
        var transparent = 0
        for (y in 0 until source.height) {
            for (x in 0 until source.width) {
                if ((pixels.getColor(x, y) ushr 24) == 0) transparent++
            }
        }
        val total = source.width * source.height

        println("transparent=$transparent of $total")
        assertTrue(
            transparent > total / 2,
            "decoration lost its transparency: only $transparent of $total are clear"
        )
    }

    @Test
    fun `every real decoration decodes`() = runBlocking {
        for (name in listOf(
            "deco_a_509a32a45886043d8a9c67474de95d2f.png",
            "deco_a_643e26a948548adb435b1078f273c426.png",
            "deco_a_22db6b911f5ad54f3939d665fe2a88cc.png"
        )) {
            val source = ApngFrameSource.parse(fixture(name))
            assertNotNull(source, "failed to parse $name")
            val target = Bitmap()
            assertTrue(target.allocN32Pixels(source.width, source.height, true))
            SkiaAnimatedImage(source).draw(Canvas(target))
            println("$name -> ${source.width}x${source.height} frames=${source.frameCount}")
        }
    }

    private fun fixture(name: String): ByteArray =
        checkNotNull(javaClass.getResourceAsStream("/imaging/$name")) { "missing $name" }
            .use { it.readBytes() }
}
