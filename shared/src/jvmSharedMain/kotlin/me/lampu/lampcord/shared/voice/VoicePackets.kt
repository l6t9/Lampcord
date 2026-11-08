package me.lampu.lampcord.shared.voice

import java.nio.ByteBuffer
import java.security.SecureRandom

internal val OPUS_SILENCE = byteArrayOf(0xf8.toByte(), 0xff.toByte(), 0xfe.toByte())
internal const val AES_MODE = "aead_aes256_gcm_rtpsize"
internal const val XCHACHA_MODE = "aead_xchacha20_poly1305_rtpsize"

internal const val OPUS_PAYLOAD_TYPE = 120

internal data class VoicePacket(
    val ssrc: Int,
    val sequence: Int,
    val frame: ByteArray,
    val payloadType: Int = OPUS_PAYLOAD_TYPE,
    val timestamp: Int = 0,
    val marker: Boolean = false
)
internal data class VoicePacketStats(
    val received: Long,
    val invalidRtp: Long,
    val nonAudio: Long,
    val malformed: Long,
    val authFailed: Long,
    val nonceRepeats: Long
)

internal class VoicePackets(
    private val key: ByteArray,
    private val aes: Boolean,
    private val ssrc: Int,
    private val payloadTypes: Set<Int> = setOf(OPUS_PAYLOAD_TYPE)
) {
    private val epochNanos = System.nanoTime()
    private var counter = 0L
    private var sequence = SecureRandom().nextInt(65536)
    private var timestamp = SecureRandom().nextInt()
    private var packetCount = 0L
    private var octetCount = 0L
    private val replay = mutableMapOf<Int, ReplayWindow>()
    private var received = 0L
    private var invalidRtp = 0L
    private var nonAudio = 0L
    private var malformed = 0L
    private var authFailed = 0L
    private var nonceRepeats = 0L

    init { require(key.size == 32) }

    fun startsTalkspurt(now: Long = System.nanoTime()) {
        val elapsed = now - epochNanos
        if (elapsed <= 0L) return
        timestamp = ((elapsed / 1_000_000_000.0) * 48_000.0).toLong().toInt()
    }

    fun encrypt(frame: ByteArray): ByteArray {
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val header = ByteBuffer.allocate(12).put(0x80.toByte()).put(120.toByte())
            .putShort(sequence++.toShort()).putInt(timestamp).putInt(ssrc).array()
        timestamp += 960
        packetCount++
        octetCount += frame.size
        return encryptPacket(header, frame)
    }

    fun senderReport(): ByteArray {
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val header = ByteBuffer.allocate(8)
            .put(0x80.toByte())
            .put(200.toByte())
            .putShort(6)
            .putInt(ssrc)
            .array()
        val now = System.currentTimeMillis() / 1000.0 + 2_208_988_800.0
        val seconds = now.toLong()
        val fraction = ((now - seconds) * 4_294_967_296.0).toLong()
        val body = ByteBuffer.allocate(20)
            .putInt(seconds.toInt())
            .putInt(fraction.toInt())
            .putInt(timestamp)
            .putInt(packetCount.toInt())
            .putInt(octetCount.toInt())
            .array()
        return encryptPacket(header, body)
    }

    /** RTCP picture loss indication (RFC 4585 6.3.1) asking [media] for a keyframe. */
    fun pictureLoss(media: Int): ByteArray {
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val header = ByteBuffer.allocate(8).put(0x81.toByte()).put(206.toByte()).putShort(2).putInt(ssrc).array()
        return encryptPacket(header, ByteBuffer.allocate(4).putInt(media).array())
    }

    /** RTCP generic NACK (RFC 4585 6.2.1) for 16-bit sequence numbers of [media], given in arrival order. */
    fun nack(media: Int, sequences: List<Int>): ByteArray? {
        if (sequences.isEmpty()) return null
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val items = mutableListOf<Int>()
        val sorted = sequences.distinct()
        var index = 0
        while (index < sorted.size && items.size < 16) {
            val pid = sorted[index++]
            var mask = 0
            while (index < sorted.size) {
                val distance = (sorted[index] - pid) and 65535
                if (distance !in 1..16) break
                mask = mask or (1 shl (distance - 1))
                index++
            }
            items += (pid shl 16) or mask
        }
        val header = ByteBuffer.allocate(8).put(0x81.toByte()).put(205.toByte()).putShort((items.size + 2).toShort()).putInt(ssrc).array()
        val body = ByteBuffer.allocate(4 + items.size * 4).putInt(media)
        items.forEach { body.putInt(it) }
        return encryptPacket(header, body.array())
    }

    /**
     * RTCP receiver estimated maximum bitrate (draft-alvestrand-rmcat-remb). Without bandwidth
     * feedback the SFU assumes a slow receiver, queues packets and asks senders for less.
     */
    fun estimatedBitrate(bitsPerSecond: Long, media: List<Int>): ByteArray? {
        if (media.isEmpty()) return null
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val sources = media.take(255)
        var mantissa = bitsPerSecond.coerceAtLeast(0)
        var exponent = 0
        while (mantissa > 0x3ffff) { mantissa = mantissa shr 1; exponent++ }
        val header = ByteBuffer.allocate(8).put(0x8f.toByte()).put(206.toByte()).putShort((sources.size + 4).toShort()).putInt(ssrc).array()
        val body = ByteBuffer.allocate(12 + sources.size * 4).putInt(0).put("REMB".toByteArray(Charsets.US_ASCII))
            .putInt((sources.size shl 24) or (exponent shl 18) or mantissa.toInt())
        sources.forEach { body.putInt(it) }
        return encryptPacket(header, body.array())
    }

    private fun encryptPacket(header: ByteArray, payload: ByteArray): ByteArray {
        val suffix = ByteBuffer.allocate(4).putInt(counter++.toInt()).array()
        val nonce = suffix.copyOf(if (aes) 12 else 24)
        val encrypted = checkNotNull(NativeVoice.aead(true, aes, key, nonce, header, payload))
        return header + encrypted + suffix
    }

    fun decrypt(packet: ByteArray): VoicePacket? {
        received++
        if (packet.size < 12 || packet[0].toInt() and 0xc0 != 0x80) {
            invalidRtp++
            return null
        }
        val payloadType = packet[1].toInt() and 0x7f
        if (payloadType !in payloadTypes) {
            nonAudio++
            return null
        }
        if (packet.size < 32) {
            malformed++
            return null
        }
        val extension = packet[0].toInt() and 0x10 != 0
        val base = 12 + (packet[0].toInt() and 15) * 4
        val headerSize = base + if (extension) 4 else 0
        if (headerSize + 20 > packet.size) {
            malformed++
            return null
        }
        val ssrc = ByteBuffer.wrap(packet, 8, 4).int
        val sequence = unsignedShort(packet, 2)
        val suffix = packet.copyOfRange(packet.size - 4, packet.size)
        val nonce = suffix.copyOf(if (aes) 12 else 24)
        val payload = NativeVoice.aead(false, aes, key, nonce, packet.copyOf(headerSize), packet.copyOfRange(headerSize, packet.size - 4))
        if (payload == null) {
            authFailed++
            return null
        }
        val count = ByteBuffer.wrap(suffix).int.toLong() and 0xffffffffL
        if (replay.size >= 1000 && ssrc !in replay) {
            malformed++
            return null
        }
        if (!replay.getOrPut(ssrc) { ReplayWindow() }.accept(count)) nonceRepeats++
        val start = if (extension) unsignedShort(packet, base + 2) * 4 else 0
        val padding = if (packet[0].toInt() and 0x20 != 0) {
            val size = payload.lastOrNull()?.toInt()?.and(255) ?: return null
            if (size == 0) {
                malformed++
                return null
            }
            size
        } else 0
        val end = payload.size - padding
        if (padding > payload.size || start >= end) {
            malformed++
            return null
        }
        val timestamp = ByteBuffer.wrap(packet, 4, 4).int
        val marker = packet[1].toInt() and 0x80 != 0
        return VoicePacket(ssrc, sequence, payload.copyOfRange(start, end), payloadType, timestamp, marker)
    }

    fun takeStats(): VoicePacketStats {
        val result = VoicePacketStats(received, invalidRtp, nonAudio, malformed, authFailed, nonceRepeats)
        received = 0
        invalidRtp = 0
        nonAudio = 0
        malformed = 0
        authFailed = 0
        nonceRepeats = 0
        return result
    }

    fun forget(ssrc: Int) { replay.remove(ssrc) }
    fun close() { key.fill(0); replay.clear() }
}

internal fun unsignedShort(bytes: ByteArray, offset: Int): Int =
    ((bytes[offset].toInt() and 255) shl 8) or (bytes[offset + 1].toInt() and 255)

internal class ReplayWindow {
    private var highest = -1L
    private var seen = 0UL
    fun accept(counter: Long): Boolean {
        if (counter > highest) {
            val distance = counter - highest
            seen = if (distance >= 64) 1UL else (seen shl distance.toInt()) or 1UL
            highest = counter
            return true
        }
        val distance = highest - counter
        if (distance >= 64 || seen and (1UL shl distance.toInt()) != 0UL) return false
        seen = seen or (1UL shl distance.toInt())
        return true
    }
}

internal class VoicePlayout {
    private val frames = mutableMapOf<Int, ByteArray>()
    private var next: Int? = null
    private var firstAt = 0L
    private var lastAt = 0L
    private var pcm = ShortArray(0)
    private var offset = 0

    fun offer(sequence: Int, frame: ByteArray, now: Long) {
        if (now - lastAt > 300_000_000L) {
            frames.clear(); next = sequence; firstAt = now; pcm = ShortArray(0); offset = 0
        }
        val expected = next ?: sequence.also { next = it; firstAt = now }
        val ahead = (sequence - expected) and 65535
        if (ahead >= 32768) return
        if (ahead >= 50 || frames.size >= 50) {
            frames.clear(); next = sequence; firstAt = now; pcm = ShortArray(0); offset = 0
        }
        frames.putIfAbsent(sequence, frame)
        lastAt = now
    }

    fun mix(into: IntArray, now: Long, decode: (ByteArray?) -> ShortArray?) {
        if (next == null || now - firstAt < 60_000_000L || now - lastAt > 300_000_000L) return
        var out = 0
        while (out < into.size) {
            if (offset == pcm.size) {
                val sequence = next!!
                next = (sequence + 1) and 65535
                pcm = decode(frames.remove(sequence)) ?: ShortArray(1920)
                if (pcm.isEmpty()) pcm = ShortArray(1920)
                offset = 0
            }
            val count = minOf(into.size - out, pcm.size - offset)
            repeat(count) { into[out++] += pcm[offset++].toInt() }
        }
    }
}
