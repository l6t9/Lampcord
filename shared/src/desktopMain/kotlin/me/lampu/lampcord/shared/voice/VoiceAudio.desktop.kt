package me.lampu.lampcord.shared.voice

import javax.sound.sampled.AudioFormat
import javax.sound.sampled.AudioSystem
import javax.sound.sampled.SourceDataLine
import javax.sound.sampled.TargetDataLine

internal actual fun createVoiceAudio(): VoiceAudio = DesktopVoiceAudio()

private class DesktopVoiceAudio : VoiceAudio {
    private val format = AudioFormat(48000f, 16, 2, true, false)
    private var input: TargetDataLine? = null
    private var output: SourceDataLine? = null
    @Volatile private var closed = false
    private val readBuffer = ByteArray(1920)
    private val writeBuffer = ByteArray(3840)

    init {
        try {
            val captureFormat = AudioFormat(48000f, 16, 1, true, false)
            input = AudioSystem.getTargetDataLine(captureFormat)
            input!!.open(captureFormat, 1920 * 3)
            input!!.start()
            output = AudioSystem.getSourceDataLine(format)
            output!!.open(format, 3840 * 3)
            output!!.start()
        } catch (e: Exception) {
            close()
            throw e
        }
    }

    override fun read(pcm: ShortArray) {
        var offset = 0
        while (offset < readBuffer.size && !closed) {
            val count = checkNotNull(input).read(readBuffer, offset, readBuffer.size - offset)
            check(count > 0) { "Microphone stopped" }
            offset += count
        }
        check(!closed)
        for (i in 0 until 960) {
            val sample = ((readBuffer[i * 2].toInt() and 255) or (readBuffer[i * 2 + 1].toInt() shl 8)).toShort()
            pcm[i * 2] = sample; pcm[i * 2 + 1] = sample
        }
        readBuffer.fill(0)
    }

    override fun write(pcm: ShortArray) {
        for (i in pcm.indices) {
            writeBuffer[i * 2] = pcm[i].toByte()
            writeBuffer[i * 2 + 1] = (pcm[i].toInt() shr 8).toByte()
        }
        var offset = 0
        while (offset < writeBuffer.size && !closed) {
            val count = checkNotNull(output).write(writeBuffer, offset, writeBuffer.size - offset)
            check(count > 0) { "Audio output stopped" }
            offset += count
        }
        check(!closed)
    }

    override fun setSpeaker(enabled: Boolean) = Unit // Desktop uses the system-selected devices.

    @Synchronized override fun close() {
        if (closed) return
        closed = true
        input?.stop(); input?.close()
        output?.stop(); output?.flush(); output?.close()
    }
}
