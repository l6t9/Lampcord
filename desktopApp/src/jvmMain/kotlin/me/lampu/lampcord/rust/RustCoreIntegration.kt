package me.lampu.lampcord.rust

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.utils.Logging
import uniffi.lampcord_ffi.ClientSnapshot

/**
 * Read-only parity slice: injects Rust core snapshots into existing Kotlin stores
 * so current Compose screens can render Rust-sourced data. Feature-flagged only.
 */
class RustCoreIntegration(
    private val bridge: RustCoreBridge,
    private val entityStore: EntityStore,
    private val messageStore: MessageStore,
) : AutoCloseable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var job: Job? = null

    fun start() {
        if (job != null) return
        job = scope.launch {
            var lastSeq: ULong? = null
            bridge.snapshots.collectLatest { snap ->
                if (snap.seq == lastSeq) return@collectLatest
                lastSeq = snap.seq
                runCatching { inject(snap) }
                    .onFailure { Logging.w("RustCore", "snapshot injection failed: ${it.message}", it) }
            }
        }
        Logging.i("RustCore", "RustCoreIntegration started (read-only injection)")
    }

    private fun inject(snap: ClientSnapshot) {
        val guilds = RustSnapshotMapper.mapGuilds(snap)
        val channels = RustSnapshotMapper.mapChannels(snap)
        val messages = RustSnapshotMapper.mapMessages(snap)

        guilds.forEach { entityStore.updateGuild(it) }
        channels.forEach { entityStore.updateChannel(it) }

        val selectedChannelId = snap.selectedChannelId?.toString()
        if (selectedChannelId != null && messages.isNotEmpty()) {
            messageStore.addMessages(selectedChannelId, messages)
        }
    }

    override fun close() {
        job?.cancel()
        job = null
        scope.cancel()
        Logging.i("RustCore", "RustCoreIntegration stopped")
    }
}
