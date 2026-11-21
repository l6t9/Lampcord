package me.lampu.lampcord.rust

import me.lampu.lampcord.shared.state.RustCoreMode

fun main() {
    RustCoreMode.enabled = true
    val engine = RustCoreEngine(RustClientConfig())
    try {
        val snap = engine.snapshot()
        check(snap.schemaVersion > 0u) { "expected a schema version" }
        check(snap.seq >= 0uL) { "expected seq >= 0" }
        check(engine.shutdown()) { "shutdown should succeed" }
        println("rust-ffi smoke ok: schema=${snap.schemaVersion} seq=${snap.seq} connection=${snap.connection.state}")
    } finally {
        engine.close()
    }
}
