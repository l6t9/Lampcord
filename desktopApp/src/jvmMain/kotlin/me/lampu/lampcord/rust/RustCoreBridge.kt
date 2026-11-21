package me.lampu.lampcord.rust

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import uniffi.lampcord_ffi.ClientCommand
import uniffi.lampcord_ffi.ClientConfig
import uniffi.lampcord_ffi.ClientEvent
import uniffi.lampcord_ffi.ClientSnapshot
import uniffi.lampcord_ffi.LampcordClient

/**
 * JVM bridge to the Rust lampcord-ffi core.
 *
 * Proof-of-concept: keep behind a feature flag before replacing Kotlin core paths.
 * Polls the Rust runtime and exposes snapshots/events as Kotlin flows.
 */
class RustCoreBridge(
    config: ClientConfig = ClientConfig(
        apiBase = "https://discord.com/api/v9",
        gatewayUrl = null,
        userAgent = null,
        storagePath = null,
    ),
    pollIntervalMs: Long = 100L,
) : AutoCloseable {
    private val client = LampcordClient.start(config)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _snapshots = MutableStateFlow(client.snapshot())
    val snapshots: StateFlow<ClientSnapshot> = _snapshots.asStateFlow()

    private val _events = MutableSharedFlow<ClientEvent>(extraBufferCapacity = 64)
    val events: SharedFlow<ClientEvent> = _events.asSharedFlow()

    private var pollJob: Job? = scope.launch {
        var lastSeq = _snapshots.value.seq
        while (isActive) {
            val events = runCatching { client.drainEvents() }.getOrDefault(emptyList())
            for (event in events) {
                _events.tryEmit(event)
            }
            val snap = runCatching { client.snapshot() }.getOrNull()
            if (snap != null && snap.seq != lastSeq) {
                lastSeq = snap.seq
                _snapshots.value = snap
            }
            delay(pollIntervalMs)
        }
    }

    fun snapshot(): ClientSnapshot = client.snapshot()
    fun drainEvents(): List<ClientEvent> = client.drainEvents()
    fun send(command: ClientCommand): Boolean = runCatching { client.send(command) }.isSuccess
    fun connectToken(token: String): Boolean = runCatching { client.connectToken(token) }.isSuccess
    fun selectGuild(guildId: ULong?): Boolean = runCatching { client.selectGuild(guildId) }.isSuccess
    fun selectChannel(channelId: ULong?): Boolean = runCatching { client.selectChannel(channelId) }.isSuccess
    fun fetchMessages(channelId: ULong, before: ULong? = null, limit: UByte? = null): Boolean =
        runCatching { client.fetchMessages(channelId, before, limit) }.isSuccess
    fun sendMessage(channelId: ULong, content: String, replyToMessageId: ULong? = null, replyToGuildId: ULong? = null): Boolean =
        runCatching { client.sendMessage(channelId, content, replyToMessageId, replyToGuildId) }.isSuccess
    fun logout(): Boolean = runCatching { client.logout() }.isSuccess

    fun shutdown(): Boolean {
        pollJob?.cancel()
        pollJob = null
        scope.cancel()
        return runCatching { client.shutdown() }.isSuccess
    }

    override fun close() {
        shutdown()
    }
}
