package me.lampu.lampcord.shared.image

import com.github.panpf.sketch.decode.Decoder
import com.github.panpf.sketch.decode.ImageInfo
import com.github.panpf.sketch.fetch.FetchResult
import com.github.panpf.sketch.request.ImageData
import com.github.panpf.sketch.request.RequestContext
import com.github.panpf.sketch.request.disallowAnimatedImage
import com.github.panpf.sketch.source.DataSource
import com.github.panpf.sketch.source.toByteArray
import com.github.panpf.sketch.util.Size

private const val MIME_TYPE_APNG = "image/apng"

actual fun apngDecoderFactory(): Decoder.Factory = ApngDecoder.Factory()

/**
 * Decode APNG animated images on the JVM desktop target.
 *
 * The APNG frame data is decoded with the pure-JVM `com.tianscar.imageio:imageio-apng` ImageIO
 * plugin; the fcTL metadata is parsed directly from the chunks so that frames can be composited
 * with APNG dispose/blend semantics into full canvases.
 */
class ApngDecoder(
    private val requestContext: RequestContext,
    private val dataSource: DataSource,
) : Decoder {

    private var _imageInfo: ImageInfo? = null

    override suspend fun getImageInfo(): ImageInfo {
        _imageInfo?.let { return it }
        val info = parseApngInfo(dataSource.toByteArray()) ?: error("Not an APNG file")
        return ImageInfo(Size(info.width, info.height), MIME_TYPE_APNG).also {
            _imageInfo = it
        }
    }

    override suspend fun decode(): ImageData {
        val bytes = dataSource.toByteArray()
        val info = parseApngInfo(bytes) ?: error("Not an APNG file")
        val size = Size(info.width, info.height)
        val imageInfo = ImageInfo(size, MIME_TYPE_APNG)
        val animatedImage = ApngAnimatedImage(bytes, info)
        return ImageData(
            image = animatedImage,
            imageInfo = imageInfo,
            dataFrom = dataSource.dataFrom,
            resize = requestContext.computeResize(size),
            transformeds = null,
            extras = null,
        )
    }

    class Factory : Decoder.Factory {

        override val key: String = "ApngDecoder"
        override val sortWeight: Int = 20

        override fun create(
            requestContext: RequestContext,
            fetchResult: FetchResult,
        ): ApngDecoder? {
            if (requestContext.request.disallowAnimatedImage == true) return null
            if (!isApplicable(fetchResult)) return null
            return ApngDecoder(requestContext, fetchResult.dataSource)
        }

        private fun isApplicable(fetchResult: FetchResult): Boolean {
            return isApngFile(fetchResult.headerBytes)
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            return other != null && this::class == other::class
        }

        override fun hashCode(): Int {
            return this::class.hashCode()
        }

        override fun toString(): String = "ApngDecoder"
    }
}