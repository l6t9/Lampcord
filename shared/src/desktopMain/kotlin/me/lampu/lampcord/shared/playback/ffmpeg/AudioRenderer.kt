package me.lampu.lampcord.shared.playback.ffmpeg

import java.nio.ByteBuffer
import java.nio.ByteOrder
import javax.sound.sampled.AudioFormat
import javax.sound.sampled.AudioSystem
import javax.sound.sampled.DataLine
import javax.sound.sampled.FloatControl
import javax.sound.sampled.SourceDataLine
import kotlin.math.log10

class AudioRenderer(
    private val audioFormat: AudioFormat,
) {
    private val line: SourceDataLine
    private val masterGainControl: FloatControl?

    @Volatile
    private var volume = 1.0f

    @Volatile
    private var isPlaying = false

    @Suppress("PLATFORM_CLASS_MAPPED_TO_KOTLIN")
    private val safeLock = Object()

    init {
        val info = DataLine.Info(SourceDataLine::class.java, audioFormat)
        line =
            (AudioSystem.getLine(info) as SourceDataLine).apply {
                val latency = 0.15 // 150ms buffer for OS/GC protection
                val bufferSize = (latency * audioFormat.sampleRate * audioFormat.frameSize).toInt()
                open(audioFormat, bufferSize)
                start()
                isPlaying = true
            }

        masterGainControl =
            if (line.isControlSupported(FloatControl.Type.MASTER_GAIN)) {
                line.getControl(FloatControl.Type.MASTER_GAIN) as? FloatControl
            } else {
                null
            }
    }

    fun writePcmData(data: ByteArray?) {
        if (data == null || data.isEmpty()) return
        if (!line.isOpen || !isPlaying) return

        val currentVol = volume
        val processedData =
            if (masterGainControl == null && currentVol != 1.0f) {
                applyVolume(data, currentVol)
            } else {
                data
            }

        try {
            if (line.isOpen && isPlaying) {
                line.write(processedData, 0, processedData.size)
            }
        } catch (_: Exception) {
        }
    }

    private fun applyVolume(
        data: ByteArray,
        volume: Float,
    ): ByteArray {
        val originalBuffer = ByteBuffer.wrap(data).order(ByteOrder.LITTLE_ENDIAN)
        val originalShorts = originalBuffer.asShortBuffer()

        val result = ByteArray(data.size)
        val resultBuffer = ByteBuffer.wrap(result).order(ByteOrder.LITTLE_ENDIAN)
        val resultShorts = resultBuffer.asShortBuffer()

        val limit = originalShorts.limit()
        for (i in 0 until limit) {
            val sample = originalShorts[i]
            val scaled = (sample * volume).toInt()
            val clamped = scaled.coerceIn(Short.MIN_VALUE.toInt(), Short.MAX_VALUE.toInt())
            resultShorts.put(i, clamped.toShort())
        }
        return result
    }

    @Volatile
    private var positionAnchorMediaMs = 0L

    @Volatile
    private var positionAnchorLineFrame = 0L

    fun resetPosition(mediaMs: Long) {
        positionAnchorMediaMs = mediaMs
        positionAnchorLineFrame = line.getLongFramePosition()
    }

    fun getPlayedPositionMs(): Long {
        val sampleRate = audioFormat.sampleRate
        return if (line.isOpen && sampleRate > 0f) {
            positionAnchorMediaMs + (line.getLongFramePosition() - positionAnchorLineFrame) * 1000L / sampleRate.toLong()
        } else {
            0L
        }
    }

    fun pause() {
        synchronized(safeLock) {
            isPlaying = false
            if (line.isOpen && line.isActive) {
                Thread {
                    try {
                        line.stop()
                    } catch (_: Exception) {}
                }.start()
            }
        }
    }

    fun resume() {
        synchronized(safeLock) {
            isPlaying = true
            if (line.isOpen && !line.isActive) {
                Thread {
                    try {
                        line.start()
                    } catch (_: Exception) {}
                }.start()
            }
        }
    }

    fun flush() {
        synchronized(safeLock) {
            if (line.isOpen) {
                line.flush()
            }
        }
    }

    fun setVolume(volume: Float) {
        val clampedVol = volume.coerceIn(0.0f, 1.0f)
        this.volume = clampedVol
        masterGainControl?.let { control ->
            try {
                if (clampedVol <= 0.0001f) {
                    control.value = control.minimum
                } else {
                    val dB = (20.0f * log10(clampedVol)).coerceIn(control.minimum, control.maximum)
                    control.value = dB
                }
            } catch (_: Exception) {
                // Ignore control failure and fallback
            }
        }
    }

    fun getVolume(): Float = this.volume

    fun isPlaying(): Boolean = this.isPlaying

    fun close() {
        synchronized(safeLock) {
            isPlaying = false
            if (line.isOpen) {
                line.flush()
                line.stop()
                line.close()
            }
        }
    }
}
