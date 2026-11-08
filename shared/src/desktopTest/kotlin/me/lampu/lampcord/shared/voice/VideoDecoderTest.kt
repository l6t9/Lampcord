package me.lampu.lampcord.shared.voice

import androidx.compose.ui.graphics.toPixelMap
import org.bytedeco.ffmpeg.avutil.AVDictionary
import org.bytedeco.ffmpeg.global.avcodec.*
import org.bytedeco.ffmpeg.global.avutil.*
import org.junit.Assume.assumeNotNull
import kotlin.test.*
import org.junit.Test

class VideoDecoderTest {
    /** Encodes solid-color frames with FFmpeg's VP8 encoder and returns each encoded frame. */
    private fun encodeSolid(width: Int, height: Int, y: Int, u: Int, v: Int, count: Int): List<ByteArray> {
        val codec = avcodec_find_encoder(AV_CODEC_ID_VP8)
        assumeNotNull(codec)
        val context = avcodec_alloc_context3(codec)!!
        val frame = av_frame_alloc()!!
        val packet = av_packet_alloc()!!
        try {
            context.width(width); context.height(height)
            context.pix_fmt(AV_PIX_FMT_YUV420P)
            context.time_base(av_make_q(1, 30))
            assertTrue(avcodec_open2(context, codec, null as AVDictionary?) >= 0)
            frame.format(AV_PIX_FMT_YUV420P); frame.width(width); frame.height(height)
            assertTrue(av_frame_get_buffer(frame, 0) >= 0)
            val encoded = mutableListOf<ByteArray>()
            fun drain() {
                while (avcodec_receive_packet(context, packet) >= 0) {
                    encoded += ByteArray(packet.size()).also { packet.data().position(0).get(it) }
                    av_packet_unref(packet)
                }
            }
            repeat(count) { index ->
                for ((plane, value, rows) in listOf(Triple(0, y, height), Triple(1, u, height / 2), Triple(2, v, height / 2))) {
                    val data = frame.data(plane)
                    for (row in 0 until rows) for (column in 0 until frame.linesize(plane)) data.put(row.toLong() * frame.linesize(plane) + column, value.toByte())
                }
                frame.pts(index.toLong())
                assertTrue(avcodec_send_frame(context, frame) >= 0)
                drain()
            }
            avcodec_send_frame(context, null)
            drain()
            return encoded
        } finally {
            avcodec_free_context(context)
            av_frame_free(frame)
            av_packet_free(packet)
        }
    }

    @Test fun decodesToCorrectPixels() {
        // BT.601 limited-range red.
        val frames = encodeSolid(width = 96, height = 54, y = 81, u = 90, v = 240, count = 3)
        assertTrue(isKeyframe(VideoCodec.VP8, frames.first()))
        val decoder = createVideoDecoder(VideoCodec.VP8)
        try {
            val images = frames.mapNotNull { decoder.decode(it) }
            assertTrue(images.isNotEmpty())
            for (image in images) {
                assertEquals(96, image.width)
                assertEquals(54, image.height)
                val pixels = image.toPixelMap()
                for ((x, y) in listOf(0 to 0, 48 to 27, 95 to 53, 10 to 40)) {
                    val color = pixels[x, y]
                    assertTrue(color.red > 0.85f && color.green < 0.15f && color.blue < 0.15f, "pixel $x,$y was $color")
                }
            }
        } finally {
            decoder.close()
        }
    }
}
