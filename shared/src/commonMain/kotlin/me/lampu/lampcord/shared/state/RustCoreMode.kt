package me.lampu.lampcord.shared.state

import kotlin.concurrent.Volatile

object RustCoreMode {
    var enabled: Boolean = false

    fun requireEnabled(operation: String) {
        check(enabled) { "Rust core is disabled; refusing $operation" }
    }
}

object RustCoreWriteGuard {
    @Volatile
    private var owner: String? = null

    fun claim(owner: String): AutoCloseable {
        val previous = this.owner
        check(previous == null || previous == owner) {
            "Rust core store injection already owned by $previous"
        }
        this.owner = owner
        return AutoCloseable {
            if (this.owner == owner) this.owner = null
        }
    }

    fun require(owner: String) {
        check(this.owner == owner) { "Rust core store injection is not owned by $owner" }
    }
}
