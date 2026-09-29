package me.lampu.lampcord.shared.imaging

import coil3.Extras
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.decode.DataSource
import coil3.decode.ImageSource
import coil3.fetch.SourceFetchResult
import coil3.request.CachePolicy
import coil3.request.Options
import coil3.size.Dimension
import coil3.size.Precision
import coil3.size.Scale
import coil3.size.Size
import kotlinx.coroutines.runBlocking
import okio.Buffer
import okio.FileSystem
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.Canvas
import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeMark
import kotlin.time.TimeSource

class AnimatedDecodeTest {

    private val loader = ImageLoader.Builder(PlatformContext.INSTANCE)
        .components { platformImageDecoders().forEach { add(it) } }
        .build()

    @Test
    fun `a gif decodes into the frame buffer with real content`() = runBlocking {
        val clock = ManualClock()
        val image = decodeAnimated("anim.gif", clock)

        val first = drawToBitmap(image)
        assertNotBlank(first, "decoded frame was blank")

        clock.now = 10.seconds
        val later = drawToBitmap(image)

        assertNotBlank(later, "advanced frame was blank")
        assertTrue(!first.sameAs(later), "frames did not advance after 10s, the animation is frozen")
    }

    @Test
    fun `an animated webp also decodes into the frame buffer`() = runBlocking {
        val image = decodeAnimated("anim.webp", ManualClock())
        assertNotBlank(drawToBitmap(image), "webp decoded blank")
    }

    @Test
    fun `an animated png decodes into the frame buffer`() = runBlocking {
        val image = decodeAnimated("anim.png", ManualClock())
        assertNotBlank(drawToBitmap(image), "apng decoded blank")
    }

    private suspend fun decodeAnimated(name: String, timeSource: TimeSource): SkiaAnimatedImage {
        val bytes = checkNotNull(javaClass.getResourceAsStream("/imaging/$name")) { "missing $name" }
            .use { it.readBytes() }
        val fetchResult = SourceFetchResult(
            source = ImageSource(Buffer().apply { write(bytes) }, FileSystem.SYSTEM),
            mimeType = null,
            dataSource = DataSource.MEMORY
        )
        val options = Options(
            context = PlatformContext.INSTANCE,
            size = Size(Dimension.Pixels(64), Dimension.Pixels(64)),
            scale = Scale.FIT,
            precision = Precision.EXACT,
            diskCacheKey = null,
            fileSystem = FileSystem.SYSTEM,
            memoryCachePolicy = CachePolicy.ENABLED,
            diskCachePolicy = CachePolicy.ENABLED,
            networkCachePolicy = CachePolicy.ENABLED,
            extras = Extras.Builder().build()
        )
        val decoder = assertNotNull(loader.components.newDecoder(fetchResult, options, loader)).first
        val source = assertNotNull(fetchResult.source.source())
        val decoded = assertNotNull(DesktopAnimatedDecoder(source, timeSource).decode()).image
        assertTrue(decoded is SkiaAnimatedImage, "$name produced ${decoded::class.simpleName}")
        return decoded as SkiaAnimatedImage
    }

    private fun drawToBitmap(image: SkiaAnimatedImage): Bitmap {
        val bitmap = Bitmap()
        check(bitmap.allocN32Pixels(image.width, image.height, false))
        val canvas = Canvas(bitmap)
        canvas.clear(0x00000000)
        image.draw(canvas)
        bitmap.notifyPixelsChanged()
        return bitmap
    }

    private fun assertNotBlank(bitmap: Bitmap, message: String) {
        val pixels = assertNotNull(bitmap.peekPixels())
        var visible = 0
        for (y in 0 until bitmap.height) {
            for (x in 0 until bitmap.width) {
                if ((pixels.getColor(x, y) ushr 24) != 0) visible++
            }
        }
        assertTrue(visible > 0, message)
    }

    private fun Bitmap.sameAs(other: Bitmap): Boolean {
        if (width != other.width || height != other.height) return false
        val a = assertNotNull(peekPixels())
        val b = assertNotNull(other.peekPixels())
        for (y in 0 until height) {
            for (x in 0 until width) {
                if (a.getColor(x, y) != b.getColor(x, y)) return false
            }
        }
        return true
    }

    private class ManualClock : TimeSource {
        var now: Duration = Duration.ZERO
        override fun markNow(): TimeMark = object : TimeMark {
            private val start = now
            override fun elapsedNow(): Duration = now - start
        }
    }
}
