package me.lampu.lampcord.rust

import uniffi.lampcord_ffi.ClientConfig

/** Simple JVM smoke test: load UniFFI, start client, snapshot, shutdown. */
fun main() {
    val bridge = RustCoreBridge(
        ClientConfig(
            apiBase = "https://discord.com/api/v9",
            gatewayUrl = null,
            userAgent = null,
            storagePath = null,
        )
    )
    try {
        val snap = bridge.snapshot()
        check(snap.seq >= 0uL) { "expected seq >= 0" }
        check(bridge.shutdown()) { "shutdown should succeed" }
        println("rust-ffi smoke ok: seq=${snap.seq} connection=${snap.connection}")
    } finally {
        bridge.close()
    }
}
