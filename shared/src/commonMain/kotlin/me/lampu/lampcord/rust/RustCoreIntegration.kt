package me.lampu.lampcord.rust

import androidx.compose.runtime.snapshotFlow
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.RustCoreMode
import me.lampu.lampcord.shared.state.RustCoreWriteGuard
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.ReadStateStore
import me.lampu.lampcord.shared.utils.Logging

class RustCoreIntegration(
    private val engine: RustCoreEngine,
    private val entityStore: EntityStore,
    private val messageStore: MessageStore,
    private val navigationStore: NavigationStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore,
    private val readStateStore: ReadStateStore,
) : AutoCloseable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var job: Job? = null
    private var ownership: AutoCloseable? = null

    fun start() {
        if (job != null) return
        RustCoreMode.requireEnabled("store injection")
        ownership = RustCoreWriteGuard.claim("RustCoreIntegration")

        job = scope.launch {
            launch {
                var lastSeq: ULong? = null
                engine.snapshots.collectLatest { snapshot ->
                    if (snapshot.seq == lastSeq) return@collectLatest
                    lastSeq = snapshot.seq
                    runCatching { inject(snapshot) }
                        .onFailure { Logging.w("RustCore", "snapshot injection failed: ${it.message}", it) }
                }
            }

            launch {
                var lastSequence: ULong? = null
                engine.events.collectLatest { envelope ->
                    if (envelope.sequence == lastSequence) return@collectLatest
                    lastSequence = envelope.sequence
                    runCatching { handle(envelope.event) }
                        .onFailure { Logging.w("RustCore", "event injection failed: ${it.message}", it) }
                }
            }

            launch {
                snapshotFlow { navigationStore.selectedGuild?.id }
                    .distinctUntilChanged()
                    .collectLatest { guildId ->
                        engine.selectGuild(guildId?.toULongOrNull())
                    }
            }

            launch {
                snapshotFlow { navigationStore.selectedChannel?.id }
                    .distinctUntilChanged()
                    .collectLatest { channelId ->
                        val id = channelId?.toULongOrNull()
                        engine.selectChannel(id)
                        if (id != null) engine.fetchMessages(id, null, 50u)
                    }
            }
        }
        Logging.i("RustCore", "RustCoreIntegration started (Rust authoritative)")
    }

    private fun inject(snapshot: RustSnapshot) {
        when (snapshot.connection.state) {
            RustConnectionState.READY, RustConnectionState.CONNECTED -> {
                navigationStore.isConnected = true
                navigationStore.isConnecting = false
            }
            RustConnectionState.CONNECTING, RustConnectionState.RESUMING -> {
                navigationStore.isConnecting = true
            }
            RustConnectionState.DISCONNECTED, RustConnectionState.ERROR -> {
                navigationStore.isConnected = false
                navigationStore.isConnecting = false
            }
        }

        snapshot.currentUserId?.let { id ->
            val name = snapshot.currentUser ?: id.toString()
            userStore.setCurrentUser(User(id = id.toString(), username = name, global_name = name))
        }

        RustSnapshotMapper.mapGuilds(snapshot).forEach(entityStore::updateGuild)
        RustSnapshotMapper.mapChannels(snapshot).forEach(entityStore::updateChannel)
        RustSnapshotMapper.mapMembers(snapshot).forEach { (guildId, members) ->
            userStore.cacheMembers(guildId, members)
        }
        presenceStore.applyPresences(RustSnapshotMapper.mapPresences(snapshot))
        readStateStore.applyAcknowledgements(RustSnapshotMapper.mapAcknowledgements(snapshot))

        val selectedChannelId = snapshot.selectedChannelId?.toString()
        val messages = RustSnapshotMapper.mapMessages(snapshot)
        if (selectedChannelId != null && messages.isNotEmpty()) {
            messageStore.mergeMessages(selectedChannelId, messages)
        }

        val rustGuildId = snapshot.selectedGuildId?.toString()
        val rustChannelId = snapshot.selectedChannelId?.toString()
        if (rustGuildId != null && navigationStore.selectedGuild?.id != rustGuildId) {
            entityStore.guilds.value[rustGuildId]?.let { navigationStore.selectedGuild = it }
        }
        if (rustChannelId != null && navigationStore.selectedChannel?.id != rustChannelId) {
            entityStore.channels.value[rustChannelId]?.let { navigationStore.selectedChannel = it }
        }
    }

    private fun handle(event: RustEvent) {
        when (event) {
            RustEvent.SnapshotUpdated -> Unit
            is RustEvent.ConnectionChanged -> Unit
            is RustEvent.AuthChanged -> Logging.i("RustCore", "auth=${event.challenge}")
            is RustEvent.RemoteAuthChanged -> Logging.i("RustCore", "remoteAuth=${event.state}")
            is RustEvent.MessageUpdated -> {
                val message = RustSnapshotMapper.mapMessage(event.message)
                messageStore.handleMessageUpdate(message, messageUpdatePayload(event.message))
            }
            is RustEvent.MessageDeleted -> messageStore.handleMessageDelete(
                event.messageId.toString(),
                event.channelId.toString(),
            )
            is RustEvent.ReactionChanged -> Logging.d(
                "RustCore",
                "reaction channel=${event.channelId} message=${event.messageId} emoji=${event.emoji} added=${event.added}",
            )
            is RustEvent.TypingStarted -> Logging.d(
                "RustCore",
                "typing channel=${event.channelId} user=${event.userId}",
            )
            is RustEvent.PresenceUpdated -> presenceStore.applyPresences(
                listOf(
                    me.lampu.lampcord.shared.model.PresenceUpdate(
                        user = User(id = event.presence.userId.toString()),
                        user_id = event.presence.userId.toString(),
                        status = event.presence.status,
                        activities = event.presence.activities.map { name ->
                            me.lampu.lampcord.shared.model.Activity(name = name, type = 0)
                        },
                    ),
                ),
            )
            is RustEvent.NotificationReceived -> messageStore.mergeMessages(
                event.message.channelId.toString(),
                listOf(RustSnapshotMapper.mapMessage(event.message)),
            )
            is RustEvent.MentionsLoaded -> Logging.i("RustCore", "mentions loaded: ${event.count}")
            is RustEvent.CommandFailed -> Logging.w(
                "RustCore",
                "command failed (${event.kind}): ${event.message}",
            )
            is RustEvent.Error -> Logging.w("RustCore", "FFI error: ${event.message}")
            RustEvent.LoggedOut -> Logging.i("RustCore", "Rust core logged out")
        }
    }

    override fun close() {
        job?.cancel()
        job = null
        ownership?.close()
        ownership = null
        scope.cancel()
        Logging.i("RustCore", "RustCoreIntegration stopped")
    }
}

private fun messageUpdatePayload(message: RustMessage) = buildJsonObject {
    put("id", JsonPrimitive(message.id.toString()))
    put("channel_id", JsonPrimitive(message.channelId.toString()))
    put("content", JsonPrimitive(message.content))
    put(
        "edited_timestamp",
        message.editedTimestamp?.let(::JsonPrimitive) ?: JsonNull,
    )
}
