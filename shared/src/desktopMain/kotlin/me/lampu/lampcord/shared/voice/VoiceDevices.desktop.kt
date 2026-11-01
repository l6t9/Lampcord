package me.lampu.lampcord.shared.voice

import javax.sound.sampled.AudioFormat
import javax.sound.sampled.AudioSystem
import javax.sound.sampled.DataLine
import javax.sound.sampled.SourceDataLine
import javax.sound.sampled.TargetDataLine

private val inputFormat = AudioFormat(48000f, 16, 1, true, false)
private val outputFormat = AudioFormat(48000f, 16, 2, true, false)

actual fun listVoiceInputDevices(): List<VoiceDevice> =
    listVoiceDevices(TargetDataLine::class.java, inputFormat)

actual fun listVoiceOutputDevices(): List<VoiceDevice> =
    listVoiceDevices(SourceDataLine::class.java, outputFormat)

private fun listVoiceDevices(
    lineClass: Class<out javax.sound.sampled.DataLine>,
    format: AudioFormat
): List<VoiceDevice> = buildList {
    add(VoiceDevice("", "System default"))
    AudioSystem.getMixerInfo().forEachIndexed { index, info ->
        val mixer = AudioSystem.getMixer(info)
        if (mixer.isLineSupported(DataLine.Info(lineClass, format))) {
            add(VoiceDevice(index.toString(), info.name))
        }
    }
}
