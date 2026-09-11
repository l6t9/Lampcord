package me.lampu.lampcord.shared.utils

import org.slf4j.Logger
import org.slf4j.LoggerFactory

private val logger: Logger = LoggerFactory.getLogger("lampcord")

actual fun platformLog(level: String, tag: String, message: String, throwable: Throwable?) {
    val entry = "[$tag] $message"

    when (level) {
        "D" -> logger.debug(entry, throwable)
        "I" -> logger.info(entry, throwable)
        "W" -> logger.warn(entry, throwable)
        else -> logger.error(entry, throwable)
    }
}
