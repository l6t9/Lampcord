package me.lampu.lampcord.shared.voice

import java.nio.ByteBuffer
import java.security.SecureRandom

internal val OPUS_SILENCE = byteArrayOf(0xf8.toByte(), 0xff.toByte(), 0xfe.toByte())
internal const val AES_MODE = "aead_aes256_gcm_rtpsize"
internal const val XCHACHA_MODE = "aead_xchacha20_poly1305_rtpsize"

internal data class VoicePacket(val ssrc: Int, val sequence: Int, val frame: ByteArray)

internal class VoicePackets(private val key: ByteArray, private val aes: Boolean, private val ssrc: Int) {
    private var counter = 0L
    private var sequence = SecureRandom().nextInt(65536)
    private var timestamp = SecureRandom().nextInt()
    private val replay = mutableMapOf<Int, ReplayWindow>()

    init { require(key.size == 32) }

    fun encrypt(frame: ByteArray): ByteArray {
        check(counter <= 0xffffffffL) { "Voice transport nonce exhausted; reconnect required" }
        val header = ByteBuffer.allocate(12).put(0x80.toByte()).put(120.toByte())
            .putShort(sequence++.toShort()).putInt(timestamp).putInt(ssrc).array()
        timestamp += 960
        val suffix = ByteBuffer.allocate(4).putInt(counter++.toInt()).array()
        val nonce = suffix.copyOf(if (aes) 12 else 24)
        val encrypted = checkNotNull(NativeVoice.aead(true, aes, key, nonce, header, frame))
        return header + encrypted + suffix
    }

    fun decrypt(packet: ByteArray): VoicePacket? {
        if (packet.size < 32 || packet[0].toInt() and 0xc0 != 0x80 || packet[1].toInt() and 0x7f != 120) return null
        val extension = packet[0].toInt() and 0x10 != 0
        val base = 12 + (packet[0].toInt() and 15) * 4
        val headerSize = base + if (extension) 4 else 0
        if (headerSize + 20 > packet.size) return null
        val ssrc = ByteBuffer.wrap(packet, 8, 4).int
        val sequence = unsignedShort(packet, 2)
        val suffix = packet.copyOfRange(packet.size - 4, packet.size)
        val nonce = suffix.copyOf(if (aes) 12 else 24)
        val payload = NativeVoice.aead(false, aes, key, nonce, packet.copyOf(headerSize), packet.copyOfRange(headerSize, packet.size - 4)) ?: return null
        val count = ByteBuffer.wrap(suffix).int.toLong() and 0xffffffffL
        if (replay.size >= 1000 && ssrc !in replay) return null
        if (!replay.getOrPut(ssrc) { ReplayWindow() }.accept(count)) return null
        val start = if (extension) unsignedShort(packet, base + 2) * 4 else 0
        val padding = if (packet[0].toInt() and 0x20 != 0) {
            val size = payload.lastOrNull()?.toInt()?.and(255) ?: return null
            if (size == 0) return null
            size
        } else 0
        val end = payload.size - padding
        if (start >= end) return null
        return VoicePacket(ssrc, sequence, payload.copyOfRange(start, end))
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
