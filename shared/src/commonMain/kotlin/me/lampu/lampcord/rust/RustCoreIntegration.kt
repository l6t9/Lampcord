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
import me.lampu.lampcord.shared.model.PresenceUpdate
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
    private var lastInjectedVersions = RustSnapshotVersions()
    private var lastInjectedSnapshot: RustSnapshot? = null

    fun start() {
        if (job != null) return
        RustCoreMode.requireEnabled("store injection")
        ownership = RustCoreWriteGuard.claim("RustCoreIntegration")
        lastInjectedVersions = RustSnapshotVersions()
        lastInjectedSnapshot = null

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
                    }
            }
        }
        Logging.i("RustCore", "RustCoreIntegration started (Rust authoritative)")
    }

    private fun inject(snapshot: RustSnapshot) {
        val previous = lastInjectedSnapshot
        val versions = snapshot.versions
        val metaChanged = previous == null || versions.meta != lastInjectedVersions.meta
        val guildsChanged = previous == null || versions.guilds != lastInjectedVersions.guilds
        val channelsChanged = previous == null || versions.channels != lastInjectedVersions.channels
        val messagesChanged = previous == null || versions.messages != lastInjectedVersions.messages
        val membersChanged = previous == null || versions.members != lastInjectedVersions.members
        val presencesChanged = previous == null || versions.presences != lastInjectedVersions.presences
        val readStatesChanged = previous == null || versions.readStates != lastInjectedVersions.readStates
        val selectionChanged = previous == null ||
            previous.selectedGuildId != snapshot.selectedGuildId ||
            previous.selectedChannelId != snapshot.selectedChannelId

        if (metaChanged) {
            injectMeta(snapshot)
        }
        if (guildsChanged) {
            if (previous != null) {
                removedIds(previous.guilds, snapshot.guilds, RustGuild::id).forEach(entityStore::removeGuild)
            }
            val guilds = if (previous == null) {
                RustSnapshotMapper.mapGuilds(snapshot)
            } else {
                RustSnapshotMapper.mapChangedGuilds(previous, snapshot)
            }
            guilds.forEach(entityStore::updateGuild)
        }
        if (channelsChanged) {
            if (previous != null) {
                removedIds(previous.channels, snapshot.channels, RustChannel::id).forEach(entityStore::removeChannel)
            }
            val channels = if (previous == null) {
                RustSnapshotMapper.mapChannels(snapshot)
            } else {
                RustSnapshotMapper.mapChangedChannels(previous, snapshot)
            }
            channels.forEach(entityStore::updateChannel)
        }
        if (membersChanged) {
            val members = if (previous == null) {
                RustSnapshotMapper.mapMembers(snapshot)
            } else {
                RustSnapshotMapper.mapChangedMembers(previous, snapshot)
            }
            members.forEach { (guildId, guildMembers) -> userStore.cacheMembers(guildId, guildMembers) }
        }
        if (presencesChanged) {
            val presences = if (previous == null) {
                RustSnapshotMapper.mapPresences(snapshot)
            } else {
                val removed = removedIds(previous.presences, snapshot.presences, RustPresence::userId)
                    .map { userId ->
                        PresenceUpdate(
                            user = User(id = userId),
                            user_id = userId,
                            status = "offline",
                        )
                    }
                removed + RustSnapshotMapper.mapChangedPresences(previous, snapshot)
            }
            presenceStore.applyPresences(presences)
        }
        if (readStatesChanged) {
            val acknowledgements = if (previous == null) {
                RustSnapshotMapper.mapAcknowledgements(snapshot)
            } else {
                RustSnapshotMapper.mapChangedAcknowledgements(previous, snapshot)
            }
            readStateStore.applyAcknowledgements(acknowledgements)
        }
        val selectedChannelId = snapshot.selectedChannelId?.toString()
        if (selectedChannelId != null && (messagesChanged || selectionChanged)) {
            if (previous != null && messagesChanged) {
                removedIds(previous.messages, snapshot.messages, RustMessage::id).forEach { messageId ->
                    messageStore.handleMessageDelete(messageId, selectedChannelId)
                }
            }
            val messages = if (previous == null || selectionChanged) {
                RustSnapshotMapper.mapMessages(snapshot)
            } else {
                RustSnapshotMapper.mapChangedMessages(previous, snapshot)
            }
            if (messages.isNotEmpty()) {
                messageStore.mergeMessages(selectedChannelId, messages)
            }
        }

        lastInjectedSnapshot = snapshot
        lastInjectedVersions = versions
    }

    private fun injectMeta(snapshot: RustSnapshot) {
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

private fun <T> removedIds(
    previous: List<T>,
    current: List<T>,
    id: (T) -> ULong,
): Set<String> {
    val currentIds = current.map(id).toHashSet()
    return previous.map(id).filterNot(currentIds::contains).map(ULong::toString).toSet()
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
