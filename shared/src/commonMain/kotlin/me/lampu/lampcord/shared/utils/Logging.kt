package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Logging {
    private val logger: Logger = LoggerFactory.getLogger("lampcord")
    private val _logs = MutableStateFlow<List<LogEntry>>(emptyList())
    val logs: StateFlow<List<LogEntry>> = _logs.asStateFlow()
    private const val MAX_LOGS = 2000

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    data class LogEntry(
        val timestamp: Long = DateTimeUtils.now(),
        val level: String,
        val tag: String,
        val message: String,
        val throwable: String? = null
    ) {
        override fun toString(): String = "${DateTimeUtils.formatLogTimestamp(timestamp)} $level/$tag: $message"
    }

    init {
        i("System", "Lampcord Logging initialized. Platform: ${getPlatformName()}, Version: 1.0.0")
    }

    private fun addLog(level: String, tag: String, message: String, throwable: Throwable? = null) {
        val entry = LogEntry(
            level = level,
            tag = tag,
            message = message,
            throwable = throwable?.stackTraceToString()
        )
        
        println(entry.toString())
        throwable?.printStackTrace()
        
        _logs.update { current ->
            val newList = current.toMutableList()
            if (newList.size >= MAX_LOGS) {
                newList.removeAt(0)
            }
            newList.add(entry)
            newList
        }

        // Periodically flush to disk
        scope.launch(Dispatchers.Default) {
            try {
                // We'll write the full log buffer to a file every 50 logs
                if (_logs.value.size % 50 == 0) {
                    val logContent = _logs.value.joinToString("\n")
                    writeInternalFile("lampcord_debug.log", logContent)
                }
            } catch (_: Exception) {}
        }
    }

    fun getLogs(): List<LogEntry> = _logs.value

    fun clear() {
        _logs.value = emptyList()
    }

    fun d(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        addLog("D", tag, message, throwable)
        when (throwable) {
            null -> logger.debug("[$tag] $message")
            else -> logger.debug("[$tag] $message", throwable)
        }
    }

    fun i(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        addLog("I", tag, message, throwable)
        when (throwable) {
            null -> logger.info("[$tag] $message")
            else -> logger.info("[$tag] $message", throwable)
        }
    }

    fun w(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        addLog("W", tag, message, throwable)
        when (throwable) {
            null -> logger.warn("[$tag] $message")
            else -> logger.warn("[$tag] $message", throwable)
        }
    }

    fun e(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        addLog("E", tag, message, throwable)
        when (throwable) {
            null -> logger.error("[$tag] $message")
            else -> logger.error("[$tag] $message", throwable)
        }
    }

    fun wtf(tag: String = "lampcord", message: String, throwable: Throwable? = null) {
        addLog("WTF", tag, message, throwable)
        when (throwable) {
            null -> logger.error("[$tag] $message")
            else -> logger.error("[$tag] $message", throwable)
        }
    }

    val ktorLogger = object : io.ktor.client.plugins.logging.Logger {
        override fun log(message: String) {
            // Filter out potentially sensitive data if needed, or just log it
            // Ktor logs can be very verbose, so we log them as Debug by default
            d("Ktor", message)
        }
    }
}
