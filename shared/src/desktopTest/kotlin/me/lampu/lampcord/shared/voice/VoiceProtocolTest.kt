package me.lampu.lampcord.shared.voice

import java.nio.ByteBuffer
import javax.crypto.Cipher
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec
import kotlin.test.*
import org.junit.Test

class VoiceProtocolTest {
    @Test fun audioProtocolChecks() {
        val key = ByteArray(32) { it.toByte() }
        val opus = byteArrayOf(0x78, 0x11, 0x22, 0x33)
        for (aes in listOf(true, false)) {
            val sender = VoicePackets(key.copyOf(), aes, 42)
            val receiver = VoicePackets(key.copyOf(), aes, 99)
            val first = sender.encrypt(opus)
            val second = sender.encrypt(opus)
            val tampered = first.copyOf().also { it[12] = (it[12].toInt() xor 1).toByte() }
            assertNull(receiver.decrypt(tampered))
            assertContentEquals(opus, receiver.decrypt(second)!!.frame)
            assertContentEquals(opus, receiver.decrypt(first)!!.frame) // Out of order is allowed.
            assertNull(receiver.decrypt(first)) // Authenticated replay is not.
            for (size in 0..first.size) receiver.decrypt(first.copyOf(size))

            // RTP extensions: CSRC list + clear extension preamble; extension content is encrypted.
            val header = ByteBuffer.allocate(20).put(0xb1.toByte()).put(120).putShort(7).putInt(960)
                .putInt(43).putInt(123).putShort(0xbede.toShort()).putShort(1).array()
            val payload = byteArrayOf(0x10, 0x7f, 0, 0) + opus + byteArrayOf(0, 2)
            val nonce = ByteArray(if (aes) 12 else 24)
            val ciphertext = NativeVoice.aead(true, aes, key, nonce, header, payload)!!
            assertContentEquals(opus, receiver.decrypt(header + ciphertext + nonce.copyOf(4))!!.frame)
            assertNull(NativeVoice.aead(false, aes, key, nonce, header.copyOf().also { it[2] = 1 }, ciphertext))
            if (aes) {
                val standard = Cipher.getInstance("AES/GCM/NoPadding")
                standard.init(Cipher.ENCRYPT_MODE, SecretKeySpec(key, "AES"), GCMParameterSpec(128, nonce))
                standard.updateAAD(header)
                assertContentEquals(standard.doFinal(payload), ciphertext)
            }
            sender.close(); receiver.close()
        }

        val native = NativeVoice.create("1234123412341234", "1234567890")
        try {
            assertTrue(NativeVoice.init(native, 1).isNotEmpty())
            val frame = NativeVoice.encode(native, ShortArray(1920))
            assertTrue(frame.isNotEmpty())
            assertNull(NativeVoice.encrypt(native, 42, frame)) // No MLS group, no audio transmission.
            assertNull(NativeVoice.decrypt(native, "5678567856785678", frame))
            assertFailsWith<IllegalStateException> { NativeVoice.init(native, 0) }
            assertFailsWith<IllegalStateException> { NativeVoice.encode(native, ShortArray(10)) }
        } finally { NativeVoice.destroy(native) }

        val jitter = VoicePlayout()
        val start = 1_000_000_000L
        jitter.offer(65535, byteArrayOf(1), start)
        jitter.offer(1, byteArrayOf(3), start + 40_000_000)
        jitter.offer(0, byteArrayOf(2), start + 20_000_000)
        val decoded = mutableListOf<Int?>()
        repeat(4) { tick ->
            val mixed = IntArray(1920)
            jitter.mix(mixed, start + 60_000_000 + tick * 20_000_000) {
                decoded += it?.first()?.toInt()
                ShortArray(1920) { 10 }
            }
            assertEquals(10, mixed[0])
        }
        assertEquals(listOf(1, 2, 3, null), decoded)
        val idle = IntArray(1920)
        jitter.mix(idle, start + 1_000_000_000) { error("Idle senders must not decode forever") }
        assertEquals(0, idle[0])
        val replay = ReplayWindow()
        assertTrue(replay.accept(100)); assertTrue(replay.accept(99)); assertFalse(replay.accept(99))
        assertFalse(replay.accept(0)); assertTrue(replay.accept(200)); assertFalse(replay.accept(100))
    }
}
