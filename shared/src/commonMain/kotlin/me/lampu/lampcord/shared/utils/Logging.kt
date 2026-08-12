package me.lampu.lampcord.shared.utils

import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Logging {
    private val logger: Logger = LoggerFactory.getLogger("lampcord")

    fun d(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        when (throwable) {
            null -> logger.debug("[$tag] $message")
            else -> logger.debug("[$tag] $message", throwable)
        }
    }

    fun i(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        when (throwable) {
            null -> logger.info("[$tag] $message")
            else -> logger.info("[$tag] $message", throwable)
        }
    }

    fun w(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        when (throwable) {
            null -> logger.warn("[$tag] $message")
            else -> logger.warn("[$tag] $message", throwable)
        }
    }

    fun e(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        when (throwable) {
            null -> logger.error("[$tag] $message")
            else -> logger.error("[$tag] $message", throwable)
        }
    }

    fun wtf(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        when (throwable) {
            null -> logger.error("[$tag] $message")
            else -> logger.error("[$tag] $message", throwable)
        }
    }
}