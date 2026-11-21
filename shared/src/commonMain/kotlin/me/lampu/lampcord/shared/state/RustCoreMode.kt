package me.lampu.lampcord.shared.state

/**
 * Process-wide flag for the Rust core PoC. Set from platform entrypoint before Koin init.
 * When enabled, Kotlin gateway connection is skipped and Rust core injection is authoritative.
 */
object RustCoreMode {
    var enabled: Boolean = false
}
