package me.lampu.lampcord.shared.playback.ffmpeg

import org.bytedeco.ffmpeg.avutil.AVDictionary
import org.bytedeco.ffmpeg.avutil.AVDictionaryEntry
import org.bytedeco.ffmpeg.avutil.LogCallback
import org.bytedeco.ffmpeg.global.avutil.*
import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.tools.Logger

class FFmpegLogCallback : LogCallback() {
    companion object {
        private val logger = Logger.create(FFmpegLogCallback::class.java)
        private val instance = FFmpegLogCallback().retainReference<FFmpegLogCallback>()

        @JvmStatic
        fun getInstance(): FFmpegLogCallback = instance

        @JvmStatic
        fun set() {
            setLogCallback(instance)
        }

        @JvmStatic
        fun getLevel(): Int = av_log_get_level()

        @JvmStatic
        fun setLevel(level: Int) {
            av_log_set_level(level)
        }

        @JvmStatic
        fun logRejectedOptions(
            options: AVDictionary?,
            command: String,
        ) {
            if (getLevel() >= AV_LOG_INFO && av_dict_count(options) > 0) {
                val sb = StringBuilder("$command rejected some options:")
                var e: AVDictionaryEntry? = null
                while (av_dict_iterate(options, e).also { e = it } != null) {
                    sb
                        .append("\tOption: ")
                        .append(e?.key()?.string)
                        .append(", value: ")
                        .append(e?.value()?.string)
                }
                logger.info(sb.toString())
            }
        }
    }

    override fun call(
        level: Int,
        msg: BytePointer?,
    ) {
        val message = msg?.string ?: return
        when (level) {
            AV_LOG_PANIC, AV_LOG_FATAL, AV_LOG_ERROR -> {
                logger.error(message)
            }

            AV_LOG_WARNING -> {
                logger.warn(message)
            }

            AV_LOG_INFO -> {
                logger.info(message)
            }

            AV_LOG_VERBOSE, AV_LOG_DEBUG, AV_LOG_TRACE -> {
                logger.debug(message)
            }

            else -> {}
        }
    }
}
