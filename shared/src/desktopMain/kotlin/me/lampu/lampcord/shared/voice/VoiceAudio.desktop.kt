package me.lampu.lampcord.shared.voice

import me.lampu.lampcord.shared.settings.Settings
import javax.sound.sampled.AudioFormat
import javax.sound.sampled.AudioSystem
import javax.sound.sampled.DataLine
import javax.sound.sampled.Mixer
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
            val inputMixer = voiceMixer(Settings.shared.voiceInputDevice)
                ?.takeIf { it.isLineSupported(DataLine.Info(TargetDataLine::class.java, captureFormat)) }
            input = inputMixer?.let { mixer ->
                runCatching {
                    mixer.getLine(DataLine.Info(TargetDataLine::class.java, captureFormat)) as TargetDataLine
                }.getOrNull()
            } ?: AudioSystem.getTargetDataLine(captureFormat)
            input!!.open(captureFormat, 1920 * 3)
            input!!.start()
            val outputMixer = voiceMixer(Settings.shared.voiceOutputDevice)
                ?.takeIf { it.isLineSupported(DataLine.Info(SourceDataLine::class.java, format)) }
            output = outputMixer?.let { mixer ->
                runCatching {
                    mixer.getLine(DataLine.Info(SourceDataLine::class.java, format)) as SourceDataLine
                }.getOrNull()
            } ?: AudioSystem.getSourceDataLine(format)
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
        val gain = Settings.shared.voiceInputVolume
        for (i in 0 until 960) {
            val raw = ((readBuffer[i * 2].toInt() and 255) or (readBuffer[i * 2 + 1].toInt() shl 8)).toShort()
            val sample = (raw * gain).toInt().coerceIn(-32768, 32767).toShort()
            pcm[i * 2] = sample; pcm[i * 2 + 1] = sample
        }
        readBuffer.fill(0)
    }

    override fun write(pcm: ShortArray) {
        val gain = Settings.shared.voiceOutputVolume
        for (i in pcm.indices) {
            val sample = (pcm[i] * gain).toInt().coerceIn(-32768, 32767)
            writeBuffer[i * 2] = sample.toByte()
            writeBuffer[i * 2 + 1] = (sample shr 8).toByte()
        }
        var offset = 0
        while (offset < writeBuffer.size && !closed) {
            val count = checkNotNull(output).write(writeBuffer, offset, writeBuffer.size - offset)
            check(count > 0) { "Audio output stopped" }
            offset += count
        }
        check(!closed)
    }

    override fun setSpeaker(enabled: Boolean) = Unit

    @Synchronized override fun close() {
        if (closed) return
        closed = true
        input?.stop(); input?.close()
        output?.stop(); output?.flush(); output?.close()
    }
}

private fun voiceMixer(id: String): Mixer? {
    val index = id.toIntOrNull() ?: return null
    val info = AudioSystem.getMixerInfo().getOrNull(index) ?: return null
    return runCatching { AudioSystem.getMixer(info) }.getOrNull()
}
