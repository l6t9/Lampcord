package me.lampu.lampcord.shared.voice

import androidx.compose.ui.graphics.ImageBitmap
import java.io.ByteArrayOutputStream

internal enum class VideoCodec(val payloadType: Int, val rtxPayloadType: Int, val priority: Int) {
    H264(101, 102, 1000),
    VP8(105, 106, 2000);

    companion object {
        fun forPayload(type: Int) = entries.firstOrNull { it.payloadType == type }
        fun forRtx(type: Int) = entries.firstOrNull { it.rtxPayloadType == type }
        val payloadTypes = entries.flatMap { listOf(it.payloadType, it.rtxPayloadType) }.toSet()
    }
}

/** [arrivedAt] is when the frame's first packet arrived, in [System.nanoTime] units. */
internal class EncodedVideoFrame(val codec: VideoCodec, val timestamp: Int, val packets: List<ByteArray>, val arrivedAt: Long = 0L)

internal class VideoPoll(val frames: List<EncodedVideoFrame>, val nacks: List<Int>, val lost: Boolean)

/** Decodes whole encoded frames for one sender. Implementations are used from a single thread. */
internal interface VideoDecoder : AutoCloseable {
    val codec: VideoCodec
    /** Returns the decoded picture, null when the decoder needs more input, and throws on corrupt input. */
    fun decode(frame: ByteArray): ImageBitmap?
}

internal expect fun createVideoDecoder(codec: VideoCodec): VideoDecoder

/**
 * Reorders the RTP packets of one video SSRC into whole frames. Gaps are reported for NACK once
 * they outlive normal reordering, and abandoned (marking the stream lost until the next keyframe)
 * once retransmission had time to arrive.
 */
internal class VideoJitterBuffer(
    private val nackAfterNanos: Long = 10_000_000L,
    private val giveUpAfterNanos: Long = 250_000_000L
) {
    private class Packet(val codec: VideoCodec, val timestamp: Int, val marker: Boolean, val payload: ByteArray, val arrivedAt: Long)

    private val packets = HashMap<Long, Packet>()
    private val missing = HashMap<Long, Long>()
    private val nacked = HashSet<Long>()
    private var highest = -1L
    private var next = -1L
    private var lost = false

    private fun unwrap(sequence: Int): Long {
        if (highest < 0) return (1L shl 20) + sequence
        var value = (highest and 0xffffL.inv()) or sequence.toLong()
        if (value - highest > 32768) value -= 65536
        else if (highest - value > 32768) value += 65536
        return value
    }

    fun insert(codec: VideoCodec, sequence: Int, timestamp: Int, marker: Boolean, payload: ByteArray, now: Long) {
        val index = unwrap(sequence)
        if (next >= 0 && (index - highest > 1000 || highest - index > 1000)) reset()
        if (next < 0) {
            next = index
            highest = index - 1
        }
        if (index < next || packets.containsKey(index)) return
        if (packets.size >= 2048) {
            reset()
            return insert(codec, sequence, timestamp, marker, payload, now)
        }
        packets[index] = Packet(codec, timestamp, marker, payload, now)
        if (index > highest) {
            for (gap in highest + 1 until index) missing[gap] = now
            highest = index
        }
        missing.remove(index)
        nacked.remove(index)
    }

    fun poll(now: Long): VideoPoll {
        val frames = mutableListOf<EncodedVideoFrame>()
        while (next >= 0 && next <= highest) {
            if (!packets.containsKey(next)) {
                val since = missing[next] ?: now
                if (now - since < giveUpAfterNanos) break
                val resume = packets.keys.filter { it > next }.minOrNull() ?: (highest + 1)
                for (index in next until resume) { missing.remove(index); nacked.remove(index) }
                next = resume
                lost = true
                continue
            }
            var end = next
            while (packets[end]?.marker == false) end++
            if (!packets.containsKey(end)) break
            val last = packets.getValue(end)
            val payloads = mutableListOf<ByteArray>()
            var arrivedAt = last.arrivedAt
            for (index in next..end) {
                val packet = packets.remove(index)!!
                if (packet.timestamp == last.timestamp && packet.codec == last.codec) {
                    payloads += packet.payload
                    arrivedAt = minOf(arrivedAt, packet.arrivedAt)
                }
            }
            frames += EncodedVideoFrame(last.codec, last.timestamp, payloads, arrivedAt)
            next = end + 1
        }
        val nacks = missing.entries
            .filter { (index, since) -> index >= next && now - since >= nackAfterNanos && index !in nacked }
            .map { it.key }.sorted()
        nacked += nacks
        val result = VideoPoll(frames, nacks.map { (it and 0xffff).toInt() }, lost)
        lost = false
        return result
    }

    fun reset() {
        packets.clear(); missing.clear(); nacked.clear()
        highest = -1L; next = -1L
        lost = true
    }
}

private val START_CODE = byteArrayOf(0, 0, 0, 1)

/**
 * Rebuilds the encoded frame DAVE encrypted from its RTP payloads. H264 becomes Annex B with
 * four-byte start codes, matching what libdave assumes the receiver reconstructs.
 */
internal fun depacketize(codec: VideoCodec, payloads: List<ByteArray>): ByteArray? {
    val out = ByteArrayOutputStream(payloads.sumOf { it.size } + 64)
    for (payload in payloads) {
        if (payload.isEmpty()) return null
        when (codec) {
            VideoCodec.H264 -> when (payload[0].toInt() and 0x1f) {
                in 1..23 -> { out.write(START_CODE); out.write(payload) }
                24 -> {
                    var offset = 1
                    while (offset < payload.size) {
                        if (offset + 2 > payload.size) return null
                        val size = unsignedShort(payload, offset)
                        offset += 2
                        if (size == 0 || offset + size > payload.size) return null
                        out.write(START_CODE)
                        out.write(payload, offset, size)
                        offset += size
                    }
                }
                28 -> {
                    if (payload.size < 3) return null
                    val header = payload[1].toInt()
                    if (header and 0x80 != 0) {
                        out.write(START_CODE)
                        out.write((payload[0].toInt() and 0xe0) or (header and 0x1f))
                    }
                    out.write(payload, 2, payload.size - 2)
                }
                else -> return null
            }
            VideoCodec.VP8 -> {
                var offset = 1
                if (payload[0].toInt() and 0x80 != 0) {
                    if (payload.size < 2) return null
                    val extension = payload[1].toInt()
                    offset = 2
                    if (extension and 0x80 != 0) {
                        if (offset >= payload.size) return null
                        offset += if (payload[offset].toInt() and 0x80 != 0) 2 else 1
                    }
                    if (extension and 0x40 != 0) offset++
                    if (extension and 0x30 != 0) offset++
                }
                if (offset >= payload.size) return null
                out.write(payload, offset, payload.size - offset)
            }
        }
    }
    return out.toByteArray().takeIf { it.isNotEmpty() }
}

internal fun isKeyframe(codec: VideoCodec, frame: ByteArray): Boolean = when (codec) {
    VideoCodec.VP8 -> frame.isNotEmpty() && frame[0].toInt() and 1 == 0
    VideoCodec.H264 -> {
        var keyframe = false
        var i = 0
        while (i + 3 < frame.size && !keyframe) {
            if (frame[i].toInt() == 0 && frame[i + 1].toInt() == 0 && frame[i + 2].toInt() == 1) {
                val type = frame[i + 3].toInt() and 0x1f
                keyframe = type == 5 || type == 7
                i += 3
            } else i++
        }
        keyframe
    }
}
