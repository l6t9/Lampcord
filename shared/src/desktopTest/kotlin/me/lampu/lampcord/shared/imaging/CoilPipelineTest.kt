package me.lampu.lampcord.shared.imaging

import coil3.ImageLoader
import coil3.Extras
import coil3.PlatformContext
import coil3.request.CachePolicy
import coil3.decode.DataSource
import coil3.decode.Decoder
import coil3.decode.ImageSource
import coil3.fetch.SourceFetchResult
import coil3.request.Options
import coil3.size.Dimension
import coil3.size.Precision
import coil3.size.Scale
import coil3.size.Size
import kotlinx.coroutines.runBlocking
import okio.Buffer
import okio.FileSystem
import org.jetbrains.skia.Bitmap
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class CoilPipelineTest {

    private val loader = ImageLoader.Builder(PlatformContext.INSTANCE)
        .components {
            platformImageDecoders().forEach { add(it) }
        }
        .build()

    @Test
    fun `animated formats are claimed by the animated decoder`() {
        assertAnimated("anim.gif")
        assertAnimated("anim.webp")
        assertAnimated("anim.png")
    }

    @Test
    fun `still formats are claimed by the still decoder`() {
        val decoder = assertNotNull(decoderFor("checker.png", 16, 16))
        assertTrue(decoder is DesktopStillDecoder, "checker went to ${decoder::class.simpleName}")
    }

    @Test
    fun `a downscaled checkerboard is smoothed rather than point sampled`() = runBlocking {
        val decoder = assertNotNull(decoderFor("checker.png", 16, 16))
        val bitmap = bitmapOf(decoder)

        assertEquals(16, bitmap.width)
        assertEquals(16, bitmap.height)

        val pixmap = assertNotNull(bitmap.peekPixels())
        var darkest = 255
        var brightest = 0
        for (y in 0 until bitmap.height) {
            for (x in 0 until bitmap.width) {
                val value = pixmap.getColor(x, y) and 0xFF
                darkest = minOf(darkest, value)
                brightest = maxOf(brightest, value)
            }
        }
        assertTrue(darkest > 90, "expected smoothing, darkest channel was $darkest")
        assertTrue(brightest < 170, "expected smoothing, brightest channel was $brightest")
    }

    @Test
    fun `a small image is not scaled up to fill a large request`() = runBlocking {
        val decoder = assertNotNull(decoderFor("checker.png", 512, 512))

        assertEquals(128, bitmapOf(decoder).width)
    }

    @Test
    fun `unrecognised data falls through to coil's own decoder`() {
        val decoder = assertNotNull(decoderForBytes("<svg/>".toByteArray(), 16, 16))
        assertTrue(
            decoder !is DesktopStillDecoder && decoder !is DesktopAnimatedDecoder,
            "unknown data was claimed by ${decoder::class.simpleName}"
        )
    }

    private fun assertAnimated(name: String) {
        val decoder = assertNotNull(decoderFor(name, 16, 16), "no decoder claimed $name")
        assertTrue(decoder is DesktopAnimatedDecoder, "$name went to ${decoder::class.simpleName}")
    }

    private fun decoderFor(name: String, width: Int, height: Int): Decoder? =
        decoderForBytes(fixture(name), width, height)

    private fun decoderForBytes(bytes: ByteArray, width: Int, height: Int): Decoder? {
        val fetchResult = SourceFetchResult(
            source = ImageSource(Buffer().apply { write(bytes) }, FileSystem.SYSTEM),
            mimeType = null,
            dataSource = DataSource.MEMORY
        )
        val options = Options(
            context = PlatformContext.INSTANCE,
            size = Size(Dimension.Pixels(width), Dimension.Pixels(height)),
            scale = Scale.FIT,
            precision = Precision.EXACT,
            diskCacheKey = null,
            fileSystem = FileSystem.SYSTEM,
            memoryCachePolicy = CachePolicy.ENABLED,
            diskCachePolicy = CachePolicy.ENABLED,
            networkCachePolicy = CachePolicy.ENABLED,
            extras = Extras.Builder().build()
        )
        return assertNotNull(
            loader.components.newDecoder(fetchResult, options, loader)
        ).first
    }

    private suspend fun bitmapOf(decoder: Decoder): Bitmap {
        val image = assertNotNull(assertNotNull(decoder.decode()).image)
        return (image as coil3.BitmapImage).bitmap
    }

    private fun fixture(name: String): ByteArray =
        checkNotNull(javaClass.getResourceAsStream("/imaging/$name")) { "missing $name" }
            .use { it.readBytes() }
}
