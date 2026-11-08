package me.lampu.lampcord.shared.voice

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asComposeImageBitmap
import org.bytedeco.ffmpeg.avcodec.AVCodecContext
import org.bytedeco.ffmpeg.avutil.AVDictionary
import org.bytedeco.ffmpeg.global.avcodec.*
import org.bytedeco.ffmpeg.global.avutil.*
import org.bytedeco.ffmpeg.global.swscale.*
import org.bytedeco.ffmpeg.swscale.SwsContext
import org.bytedeco.javacpp.DoublePointer
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorSpace
import org.jetbrains.skia.ColorType
import org.jetbrains.skia.ImageInfo
import me.lampu.lampcord.shared.utils.Logging
import javax.swing.SwingUtilities

internal actual fun createVideoDecoder(codec: VideoCodec): VideoDecoder = DesktopVideoDecoder(codec)

private class DesktopVideoDecoder(override val codec: VideoCodec) : VideoDecoder {
    private val context: AVCodecContext
    private val packet = checkNotNull(av_packet_alloc())
    private val picture = checkNotNull(av_frame_alloc())
    private var scaler: SwsContext? = null
    private val converted = checkNotNull(av_frame_alloc())
    private var bytes = ByteArray(0)
    private var colors: Triple<Long, Int, Boolean>? = null
    private val retained = ArrayDeque<Bitmap>()

    init {
        val decoder = checkNotNull(avcodec_find_decoder(if (codec == VideoCodec.H264) AV_CODEC_ID_H264 else AV_CODEC_ID_VP8)) {
            "FFmpeg has no $codec decoder"
        }
        context = checkNotNull(avcodec_alloc_context3(decoder))
        context.thread_count(2)
        context.thread_type(2) // FF_THREAD_SLICE: frame threading would add a frame of latency per thread
        context.flags(context.flags() or AV_CODEC_FLAG_LOW_DELAY)
        check(avcodec_open2(context, decoder, null as AVDictionary?) >= 0) { "Could not open the $codec decoder" }
    }

    override fun decode(frame: ByteArray): ImageBitmap? {
        check(av_new_packet(packet, frame.size) >= 0)
        packet.data().position(0).put(frame, 0, frame.size)
        val sent = avcodec_send_packet(context, packet)
        av_packet_unref(packet)
        check(sent >= 0 || sent == AVERROR_EAGAIN()) { "Video decode failed ($sent)" }
        var image: ImageBitmap? = null
        while (true) {
            val received = avcodec_receive_frame(context, picture)
            if (received == AVERROR_EAGAIN() || received == AVERROR_EOF) break
            check(received >= 0) { "Video decode failed ($received)" }
            image = convert() ?: image
            av_frame_unref(picture)
        }
        return image
    }

    private fun convert(): ImageBitmap? {
        val sourceWidth = picture.width()
        val sourceHeight = picture.height()
        if (sourceWidth <= 0 || sourceHeight <= 0) return null
        val scale = minOf(1.0, MAX_HEIGHT.toDouble() / sourceHeight, MAX_WIDTH.toDouble() / sourceWidth)
        val width = (sourceWidth * scale).toInt().coerceAtLeast(2) and 1.inv()
        val height = (sourceHeight * scale).toInt().coerceAtLeast(2) and 1.inv()
        var format = picture.format()
        var fullRange = picture.color_range() == AVCOL_RANGE_JPEG
        when (format) {
            AV_PIX_FMT_YUVJ420P -> { format = AV_PIX_FMT_YUV420P; fullRange = true }
            AV_PIX_FMT_YUVJ422P -> { format = AV_PIX_FMT_YUV422P; fullRange = true }
            AV_PIX_FMT_YUVJ444P -> { format = AV_PIX_FMT_YUV444P; fullRange = true }
        }
        val matrix = when (picture.colorspace()) {
            AVCOL_SPC_BT709 -> SWS_CS_ITU709
            AVCOL_SPC_BT2020_NCL, AVCOL_SPC_BT2020_CL -> SWS_CS_BT2020
            else -> SWS_CS_ITU601
        }
        val context = sws_getCachedContext(
            scaler, sourceWidth, sourceHeight, format, width, height, AV_PIX_FMT_BGRA,
            SWS_BILINEAR, null, null, null as DoublePointer?
        ) ?: return null
        scaler = context
        // swscale ignores the frame's color tags, so a BT.709 or full-range stream needs telling.
        val key = Triple(context.address(), matrix, fullRange)
        if (key != colors) {
            sws_setColorspaceDetails(context, sws_getCoefficients(matrix), if (fullRange) 1 else 0, sws_getCoefficients(SWS_CS_DEFAULT), 1, 0, 1 shl 16, 1 shl 16)
            if (colors?.let { it.second != matrix || it.third != fullRange } != false) {
                Logging.i("Voice", "Video ${sourceWidth}x$sourceHeight colorspace=${picture.colorspace()} range=${picture.color_range()} format=${picture.format()}")
            }
            colors = key
        }
        val row = width * 4
        if (converted.width() != width || converted.height() != height) {
            av_frame_unref(converted)
            converted.format(AV_PIX_FMT_BGRA)
            converted.width(width)
            converted.height(height)
            check(av_frame_get_buffer(converted, 0) >= 0) { "Could not allocate a video frame" }
            bytes = ByteArray(row * height)
        }
        if (sws_scale(scaler, picture.data(), picture.linesize(), 0, sourceHeight, converted.data(), converted.linesize()) != height) return null
        val pixels = converted.data(0)
        val stride = converted.linesize(0)
        if (stride == row) {
            pixels.position(0).get(bytes, 0, bytes.size)
        } else {
            for (y in 0 until height) pixels.position(y.toLong() * stride).get(bytes, y * row, row)
        }
        val bitmap = Bitmap()
        if (!bitmap.installPixels(ImageInfo(width, height, ColorType.BGRA_8888, ColorAlphaType.OPAQUE, ColorSpace.sRGB), bytes, width * 4)) {
            bitmap.close()
            return null
        }
        bitmap.setImmutable()
        retained.addLast(bitmap)
        while (retained.size > RETAINED_FRAMES) {
            val old = retained.removeFirst()
            SwingUtilities.invokeLater { old.close() }
        }
        return bitmap.asComposeImageBitmap()
    }

    override fun close() {
        avcodec_free_context(context)
        av_packet_free(packet)
        av_frame_free(picture)
        scaler?.let { sws_freeContext(it) }
        scaler = null
        av_frame_free(converted)
        val old = retained.toList()
        retained.clear()
        SwingUtilities.invokeLater { old.forEach { it.close() } }
    }

    private companion object {
        const val MAX_WIDTH = 1920
        const val MAX_HEIGHT = 1080
        const val RETAINED_FRAMES = 4
    }
}
