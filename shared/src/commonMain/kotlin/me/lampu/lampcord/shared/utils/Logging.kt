package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlin.concurrent.Volatile
import kotlin.time.Duration.Companion.milliseconds

object Logging {
    private const val TAG = "lampcord"
    private val _logs = MutableStateFlow<List<LogEntry>>(emptyList())
    val logs: StateFlow<List<LogEntry>> = _logs.asStateFlow()
    private const val MAX_LOGS = 2000
    private val FLUSH_INTERVAL_MS = 2000L.milliseconds

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    // Logging must never block the thread that is logging.
    private val incoming = Channel<LogEntry>(
        capacity = 1024,
        onBufferOverflow = BufferOverflow.DROP_OLDEST
    )

    @Volatile
    private var clearRequested = false

    @Volatile
    var debugEnabled: Boolean = false

    data class LogEntry(
        val timestamp: Long = DateTimeUtils.now(),
        val level: String,
        val tag: String,
        val message: String,
        val throwable: String? = null
    ) {
        override fun toString(): String = "${DateTimeUtils.formatLogTimestamp(timestamp)} $level/$tag: $message"
    }

    @Volatile
    private var pendingChanges: List<LogEntry>? = null

    init {
        scope.launch { dumpLoop() }
        scope.launch { flushLoop() }
        i("System", "Lampcord Logging initialized. Platform: ${getPlatformName()}, Version: 1.0.0")
    }

    private suspend fun dumpLoop() {
        val buffer = ArrayDeque<LogEntry>()

        while (true) {
            // Block receive without timeout and clearing buffer before appending prevents data loss.
            val entry = incoming.receive()

            if (clearRequested) {
                clearRequested = false
                buffer.clear()
            }

            buffer.append(entry)

            // Stack queue and dump it in a single go.
            while (true) {
                val next = incoming.tryReceive().getOrNull() ?: break
                buffer.append(next)
            }

            val snapshot = buffer.toList()
            _logs.value = snapshot
            pendingChanges = snapshot
        }
    }

    private suspend fun flushLoop() {
        while (true) {
            delay(FLUSH_INTERVAL_MS)

            val snapshot = pendingChanges ?: continue
            pendingChanges = null

            try {
                writeInternalFile("lampcord_debug.log", snapshot.joinToString("\n"))
            } catch (e: Exception) {
                e("Logging", "Error writing log file", e)
            }
        }
    }

    private fun addLog(level: String, tag: String, message: String, throwable: Throwable? = null) {
        val entry = LogEntry(
            level = level,
            tag = tag,
            message = message,
            throwable = throwable?.stackTraceToString()
        )

        if (debugEnabled) println(entry.toString())
        incoming.trySend(entry)
        platformLog(level, tag, message, throwable)
    }

    fun getLogs(): List<LogEntry> = _logs.value

    fun clear() {
        clearRequested = true
        _logs.value = emptyList()
    }

    fun d(tag: String = TAG, message: String, throwable: Throwable? = null) {
        if (!debugEnabled) return
        addLog("D", tag, message, throwable)
    }

    fun i(tag: String = TAG, message: String, throwable: Throwable? = null) {
        addLog("I", tag, message, throwable)
    }

    fun w(tag: String = TAG, message: String, throwable: Throwable? = null) {
        addLog("W", tag, message, throwable)
    }

    fun e(tag: String = TAG, message: String, throwable: Throwable? = null) {
        addLog("E", tag, message, throwable)
    }

    fun wtf(tag: String = TAG, message: String, throwable: Throwable? = null) {
        addLog("WTF", tag, message, throwable)
    }

    val ktorLogger = object : io.ktor.client.plugins.logging.Logger {
        override fun log(message: String) {
            // Filter out potentially sensitive data if needed, or just log it
            // Ktor logs can be very verbose, so we log them as Debug by default
            d("Ktor", message)
        }
    }

    private fun ArrayDeque<LogEntry>.append(entry: LogEntry) {
        addLast(entry)
        while (size > MAX_LOGS) removeFirst()
    }
}
