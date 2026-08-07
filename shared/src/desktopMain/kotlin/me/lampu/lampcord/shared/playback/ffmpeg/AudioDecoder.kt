package me.lampu.lampcord.shared.playback.ffmpeg

import org.bytedeco.ffmpeg.global.avutil.AV_SAMPLE_FMT_S16
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.ShortBuffer
import javax.sound.sampled.AudioFormat

class AudioDecoder(
    path: String,
    headers: Map<String, String> = emptyMap(),
) {
    val frameGrabber =
        FFmpegFrameGrabber(path).apply {
            if (path.startsWith("http://") || path.startsWith("https://")) {
                options["reconnect"] = "1"
                options["reconnect_streamed"] = "1"
                options["reconnect_delay_max"] = "5"
                options["user_agent"] =
                    headers.entries.firstOrNull { it.key.equals("User-Agent", ignoreCase = true) }?.value
                        ?: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
                headers
                    .filterKeys { !it.equals("User-Agent", ignoreCase = true) }
                    .takeIf { it.isNotEmpty() }
                    ?.entries
                    ?.joinToString(separator = "\r\n", postfix = "\r\n") { "${it.key}: ${it.value}" }
                    ?.let { options["headers"] = it }
            }
            videoStream = -2 // Disable video
            sampleFormat = AV_SAMPLE_FMT_S16
            start()
        }

    val audioFormat =
        AudioFormat(
            frameGrabber.sampleRate.toFloat(),
            16,
            frameGrabber.audioChannels,
            true,
            false,
        )

    val duration = frameGrabber.getLengthInTime() / 1000

    fun readPCMFrame(): ByteArray? {
        val frame = frameGrabber.grabSamples() ?: return null
        val samples = frame.samples ?: return null
        if (samples.isEmpty()) return null

        return when (val buffer = samples[0]) {
            is ShortBuffer -> {
                val byteArray = ByteArray(buffer.remaining() * 2)
                val resultByteBuffer = ByteBuffer.wrap(byteArray).order(ByteOrder.LITTLE_ENDIAN)
                resultByteBuffer.asShortBuffer().put(buffer)
                byteArray
            }

            is ByteBuffer -> {
                val byteArray = ByteArray(buffer.remaining())
                buffer.get(byteArray)
                byteArray
            }

            else -> {
                null
            }
        }
    }

    fun seekTo(position: Long) {
        try {
            frameGrabber.setTimestamp(position * 1000, true)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun getCurrentPosition(): Long = frameGrabber.timestamp / 1000

    fun close() {
        try {
            frameGrabber.stop()
            frameGrabber.release()
        } catch (_: Exception) {
        }
    }
}
