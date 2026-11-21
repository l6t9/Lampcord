package me.lampu.lampcord.rust

import uniffi.lampcord_ffi.ClientCommand
import uniffi.lampcord_ffi.ClientConfig
import uniffi.lampcord_ffi.ClientEvent
import uniffi.lampcord_ffi.ClientSnapshot
import uniffi.lampcord_ffi.LampcordClient

/**
 * Minimal JVM bridge to the Rust lampcord-ffi core.
 * Proof-of-concept: use behind a feature flag before replacing Kotlin core paths.
 */
class RustCoreBridge(config: ClientConfig = ClientConfig(
    apiBase = "https://discord.com/api/v9",
    gatewayUrl = null,
    userAgent = null,
    storagePath = null,
)) : AutoCloseable {
    private val client = LampcordClient.start(config)

    fun snapshot(): ClientSnapshot = client.snapshot()
    fun drainEvents(): List<ClientEvent> = client.drainEvents()
    fun send(command: ClientCommand): Boolean = client.send(command)
    fun connectToken(token: String): Boolean = client.connectToken(token)
    fun shutdown(): Boolean = client.shutdown()

    override fun close() {
        shutdown()
    }
}
