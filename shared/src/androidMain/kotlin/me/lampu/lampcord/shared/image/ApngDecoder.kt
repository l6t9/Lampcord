package me.lampu.lampcord.shared.image

import android.os.Build
import com.github.panpf.sketch.decode.Decoder
import com.github.panpf.sketch.decode.internal.ImageDecoderAnimatedDecoder
import com.github.panpf.sketch.fetch.FetchResult
import com.github.panpf.sketch.request.RequestContext
import com.github.panpf.sketch.request.disallowAnimatedImage
import com.github.panpf.sketch.source.DataSource

actual fun apngDecoderFactory(): Decoder.Factory = ApngDecoder.Factory()

// Decode APNG animated images using Android's [android.graphics.ImageDecoder], which animates APNG natively. Mirrors ImageDecoderAnimatedWebpDecoder/ImageDecoderGifDecoder.
@androidx.annotation.RequiresApi(Build.VERSION_CODES.P)
class ApngDecoder(
    requestContext: RequestContext,
    dataSource: DataSource,
) : ImageDecoderAnimatedDecoder(requestContext, dataSource) {

    companion object {
        const val SORT_WEIGHT = 20
    }

    class Factory : Decoder.Factory {

        override val key: String = "ApngDecoder"
        override val sortWeight: Int = SORT_WEIGHT

        override fun create(
            requestContext: RequestContext,
            fetchResult: FetchResult,
        ): ApngDecoder? {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P) return null
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