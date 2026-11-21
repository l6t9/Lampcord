package me.lampu.lampcord.rust

import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.utils.Logging
import uniffi.lampcord_ffi.ClientSnapshot
import uniffi.lampcord_ffi.ConnectionState

/**
 * Read-only parity slice: injects Rust core snapshots into existing Kotlin stores
 * so current Compose screens can render Rust-sourced data. Feature-flagged only.
 */
class RustCoreIntegration(
    private val bridge: RustCoreBridge,
    private val entityStore: EntityStore,
    private val messageStore: MessageStore,
    private val navigationStore: NavigationStore,
) : AutoCloseable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var job: Job? = null

    fun start() {
        if (job != null) return
        job = scope.launch {
            launch {
                var lastSeq: ULong? = null
                bridge.snapshots.collectLatest { snap ->
                    if (snap.seq == lastSeq) return@collectLatest
                    lastSeq = snap.seq
                    runCatching { inject(snap) }
                        .onFailure { Logging.w("RustCore", "snapshot injection failed: ${it.message}", it) }
                }
            }

            launch {
                snapshotFlow { navigationStore.selectedGuild?.id }
                    .distinctUntilChanged()
                    .collectLatest { guildId ->
                        val id = guildId?.toULongOrNull()
                        bridge.selectGuild(id)
                    }
            }

            launch {
                snapshotFlow { navigationStore.selectedChannel?.id }
                    .distinctUntilChanged()
                    .collectLatest { channelId ->
                        val id = channelId?.toULongOrNull()
                        bridge.selectChannel(id)
                        if (id != null) {
                            bridge.fetchMessages(id, null, 50.toUByte())
                        }
                    }
            }
        }
        Logging.i("RustCore", "RustCoreIntegration started (read-only injection)")
    }

    private fun inject(snap: ClientSnapshot) {
        when (snap.connection) {
            is ConnectionState.Ready, is ConnectionState.Connected -> {
                navigationStore.isConnected = true
                navigationStore.isConnecting = false
            }
            is ConnectionState.Connecting, is ConnectionState.Resuming -> {
                navigationStore.isConnecting = true
            }
            is ConnectionState.Disconnected, is ConnectionState.Error -> {
                navigationStore.isConnected = false
                navigationStore.isConnecting = false
            }
        }

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
