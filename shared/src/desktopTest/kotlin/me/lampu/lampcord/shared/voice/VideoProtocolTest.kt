package me.lampu.lampcord.shared.voice

import java.nio.ByteBuffer
import kotlin.test.*
import org.junit.Test

class VideoProtocolTest {
    private fun bytes(vararg values: Int) = ByteArray(values.size) { values[it].toByte() }

    @Test fun h264Depacketization() {
        val sps = bytes(0x67, 0x42, 0xc0, 0x1f)
        val pps = bytes(0x68, 0xce, 0x3c)
        val stapA = bytes(0x78) + bytes(0, sps.size) + sps + bytes(0, pps.size) + pps
        val idrStart = bytes(0x7c, 0x85, 1, 2)
        val idrMiddle = bytes(0x7c, 0x05, 3)
        val idrEnd = bytes(0x7c, 0x45, 4, 5)
        val slice = bytes(0x41, 9, 9)
        val start = bytes(0, 0, 0, 1)
        val frame = depacketize(VideoCodec.H264, listOf(stapA, idrStart, idrMiddle, idrEnd, slice))!!
        assertContentEquals(start + sps + start + pps + start + bytes(0x65, 1, 2, 3, 4, 5) + start + slice, frame)
        assertTrue(isKeyframe(VideoCodec.H264, frame))
        assertFalse(isKeyframe(VideoCodec.H264, depacketize(VideoCodec.H264, listOf(slice))!!))
        assertNull(depacketize(VideoCodec.H264, listOf(bytes(0x78, 0, 9, 1))))
        assertNull(depacketize(VideoCodec.H264, listOf(bytes(0x7d, 0x85, 1))))
        assertNull(depacketize(VideoCodec.H264, listOf(ByteArray(0))))
    }

    @Test fun vp8Depacketization() {
        // X with a 15-bit picture ID and TL0PICIDX, then a keyframe header.
        val first = bytes(0x90, 0xc0, 0x81, 0x23, 0x07, 0x10, 0x02, 0x00)
        val second = bytes(0x00, 0xaa, 0xbb)
        val frame = depacketize(VideoCodec.VP8, listOf(first, second))!!
        assertContentEquals(bytes(0x10, 0x02, 0x00, 0xaa, 0xbb), frame)
        assertTrue(isKeyframe(VideoCodec.VP8, frame))
        assertFalse(isKeyframe(VideoCodec.VP8, bytes(0x31)))
        assertNull(depacketize(VideoCodec.VP8, listOf(bytes(0x90, 0x80))))
    }

    @Test fun jitterBufferReordersAcrossWrap() {
        val buffer = VideoJitterBuffer()
        val t = 1_000_000_000L
        buffer.insert(VideoCodec.VP8, 65534, 100, false, bytes(1), t)
        buffer.insert(VideoCodec.VP8, 0, 100, true, bytes(3), t)
        assertTrue(buffer.poll(t).frames.isEmpty())
        buffer.insert(VideoCodec.VP8, 65535, 100, false, bytes(2), t + 1_000_000)
        val first = buffer.poll(t + 1_000_000)
        assertEquals(1, first.frames.size)
        assertEquals(listOf(1, 2, 3), first.frames[0].packets.map { it[0].toInt() })

        buffer.insert(VideoCodec.VP8, 1, 200, true, bytes(4), t + 2_000_000)
        buffer.insert(VideoCodec.VP8, 3, 300, true, bytes(6), t + 3_000_000)
        val waiting = buffer.poll(t + 3_000_000)
        assertEquals(listOf(200), waiting.frames.map { it.timestamp })
        assertTrue(waiting.nacks.isEmpty(), "reordering gets a grace period before NACK")
        assertFalse(waiting.lost)
        assertEquals(listOf(2), buffer.poll(t + 20_000_000).nacks)
        assertTrue(buffer.poll(t + 30_000_000).nacks.isEmpty(), "each gap is NACKed once")

        buffer.insert(VideoCodec.VP8, 2, 300, false, bytes(5), t + 40_000_000)
        val repaired = buffer.poll(t + 40_000_000)
        assertEquals(listOf(listOf(5, 6)), repaired.frames.map { frame -> frame.packets.map { it[0].toInt() } })
        assertFalse(repaired.lost)

        buffer.insert(VideoCodec.VP8, 5, 400, true, bytes(8), t + 50_000_000)
        assertTrue(buffer.poll(t + 100_000_000).frames.isEmpty())
        val abandoned = buffer.poll(t + 400_000_000)
        assertTrue(abandoned.lost)
        assertEquals(listOf(400), abandoned.frames.map { it.timestamp })
    }

    @Test fun jitterBufferDropsPartialFramesOfOtherTimestamps() {
        val buffer = VideoJitterBuffer()
        buffer.insert(VideoCodec.H264, 10, 1, false, bytes(1), 0)
        buffer.insert(VideoCodec.H264, 11, 2, true, bytes(2), 0)
        assertEquals(listOf(listOf(2)), buffer.poll(0).frames.map { frame -> frame.packets.map { it[0].toInt() } })
    }

    @Test fun rtcpFeedbackAndVideoRtp() {
        val key = ByteArray(32) { (it * 3).toByte() }
        for (aes in listOf(true, false)) {
            val packets = VoicePackets(key.copyOf(), aes, 7, setOf(OPUS_PAYLOAD_TYPE) + VideoCodec.payloadTypes)
            fun open(packet: ByteArray): ByteArray {
                val nonce = packet.copyOfRange(packet.size - 4, packet.size).copyOf(if (aes) 12 else 24)
                return NativeVoice.aead(false, aes, key, nonce, packet.copyOf(8), packet.copyOfRange(8, packet.size - 4))!!
            }
            val nack = packets.nack(99, listOf(10, 11, 13, 40))!!
            assertEquals(205, nack[1].toInt() and 255)
            assertEquals(4, unsignedShort(nack, 2))
            val body = ByteBuffer.wrap(open(nack))
            assertEquals(99, body.int)
            assertEquals((10 shl 16) or 0b101, body.int)
            assertEquals(40 shl 16, body.int)
            assertNull(packets.nack(99, emptyList()))
            val pli = packets.pictureLoss(99)
            assertEquals(206, pli[1].toInt() and 255)
            assertEquals(99, ByteBuffer.wrap(open(pli)).int)
            val remb = packets.estimatedBitrate(20_000_000, listOf(99, 100))!!
            assertEquals(15, remb[0].toInt() and 0x1f)
            assertEquals(206, remb[1].toInt() and 255)
            assertEquals(6, unsignedShort(remb, 2))
            val estimate = ByteBuffer.wrap(open(remb))
            assertEquals(0, estimate.int)
            assertEquals("REMB", ByteArray(4).also { estimate.get(it) }.toString(Charsets.US_ASCII))
            val word = estimate.int
            assertEquals(2, word ushr 24)
            val bitrate = (word and 0x3ffff).toLong() shl ((word ushr 18) and 0x3f)
            assertTrue(bitrate in 19_900_000..20_000_000, "bitrate $bitrate")
            assertEquals(listOf(99, 100), listOf(estimate.int, estimate.int))
            assertNull(packets.estimatedBitrate(20_000_000, emptyList()))

            val header = ByteBuffer.allocate(12).put(0x80.toByte()).put((0x80 or VideoCodec.H264.payloadType).toByte())
                .putShort(5).putInt(90_000).putInt(43).array()
            val nonce = ByteArray(if (aes) 12 else 24).also { it[3] = 1 }
            val payload = bytes(0x65, 1, 2, 3)
            val packet = header + NativeVoice.aead(true, aes, key, nonce, header, payload)!! + nonce.copyOf(4)
            val video = packets.decrypt(packet)!!
            assertEquals(VideoCodec.H264.payloadType, video.payloadType)
            assertEquals(90_000, video.timestamp)
            assertTrue(video.marker)
            assertContentEquals(payload, video.frame)
            assertNull(VoicePackets(key.copyOf(), aes, 7).decrypt(packet), "audio-only transports reject video")
            packets.close()
        }
    }
}
