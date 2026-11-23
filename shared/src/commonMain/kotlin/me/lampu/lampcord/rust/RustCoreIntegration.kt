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
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.RustChannelVisibility
import me.lampu.lampcord.shared.state.RustCoreMode
import me.lampu.lampcord.shared.state.RustCoreWriteGuard
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.ReadStateStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging

class RustCoreIntegration(
    private val engine: RustCoreEngine,
    private val entityStore: EntityStore,
    private val guildStore: GuildStore,
    private val messageStore: MessageStore,
    private val navigationStore: NavigationStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore,
    private val readStateStore: ReadStateStore,
    private val memberListStore: MemberListStore,
    private val relationshipStore: RelationshipStore,
) : AutoCloseable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private var job: Job? = null
    private var ownership: AutoCloseable? = null
    private var lastInjectedVersions = RustSnapshotVersions()
    private var lastInjectedSnapshot: RustSnapshot? = null
    private var initialSelectionRestored = false

    private var sentGuildId: String? = null
    private var sentChannelId: String? = null

    private fun syncSelection(guildId: String?, channelId: String?) {
        if (guildId != sentGuildId) {
            sentGuildId = guildId
            engine.selectGuild(guildId?.toULongOrNull())
        }
        if (channelId != sentChannelId) {
            sentChannelId = channelId
            engine.selectChannel(channelId?.toULongOrNull())
        }
    }

    fun start() {
        if (job != null) return
        RustCoreMode.requireEnabled("store injection")
        ownership = RustCoreWriteGuard.claim("RustCoreIntegration")
        memberListStore.rustRangeRequester = { ranges -> engine.setMemberListRanges(ranges) }
        lastInjectedVersions = RustSnapshotVersions()
        lastInjectedSnapshot = null

        job = scope.launch {
            launch {
                snapshotFlow { navigationStore.selectedGuild?.id to navigationStore.selectedChannel?.id }
                    .distinctUntilChanged()
                    .collect { (guildId, channelId) -> syncSelection(guildId, channelId) }
            }

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
            guildStore.setGuilds(
                RustSnapshotMapper.mapGuilds(snapshot),
                snapshot.guilds.map { it.id.toString() },
            )
            if (
                !initialSelectionRestored &&
                snapshot.guilds.isNotEmpty() &&
                navigationStore.selectedGuild == null &&
                navigationStore.selectedChannel == null &&
                !navigationStore.isFriendsSelected
            ) {
                initialSelectionRestored = true
                navigationStore.restoreLastState { }
            }
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
            RustChannelVisibility.replaceHidden(snapshot.channels.filter { !it.canView }.map { it.id.toString() })
            RustChannelVisibility.replaceReadOnly(snapshot.channels.filter { !it.canSend }.map { it.id.toString() })
            guildStore.setPrivateChannels(RustSnapshotMapper.mapPrivateChannels(snapshot))
        }
        if (membersChanged) {
            val members = if (previous == null) {
                RustSnapshotMapper.mapMembers(snapshot)
            } else {
                RustSnapshotMapper.mapChangedMembers(previous, snapshot)
            }
            members.forEach { (guildId, guildMembers) -> userStore.cacheMembers(guildId, guildMembers) }
            memberListStore.applyRustMemberList(
                guildId = snapshot.selectedGuildId?.toString(),
                items = RustSnapshotMapper.mapMemberListItems(snapshot),
                online = snapshot.memberList.onlineCount,
                members = snapshot.memberList.memberCount,
            )
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
            if (previous != null && messagesChanged && !selectionChanged) {
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
                navigationStore.isConnected = snapshot.guilds.isNotEmpty()
            }
            RustConnectionState.DISCONNECTED, RustConnectionState.ERROR -> {
                navigationStore.isConnected = snapshot.guilds.isNotEmpty()
                navigationStore.isConnecting = false
                if (snapshot.connection.state == RustConnectionState.ERROR &&
                    snapshot.connection.message == "Authentication failed"
                ) {
                    Logging.w("RustCore", "Token rejected; returning to login")
                    Settings.shared.discordToken = ""
                    engine.logout()
                }
            }
        }

        val lastUser = lastInjectedSnapshot
        if (lastUser?.relationships != snapshot.relationships) {
            relationshipStore.handleReady(RustSnapshotMapper.mapRelationships(snapshot))
        }
        if (lastUser?.currentUserId != snapshot.currentUserId || lastUser?.currentUser != snapshot.currentUser ||
            lastUser?.currentUserAvatar != snapshot.currentUserAvatar
        ) {
            snapshot.currentUserId?.let { id ->
                val name = snapshot.currentUser ?: id.toString()
                userStore.setCurrentUser(
                    User(id = id.toString(), username = name, global_name = name, avatar = snapshot.currentUserAvatar)
                )
            }
        }

    }

    private fun handle(event: RustEvent) {
        when (event) {
            RustEvent.SnapshotUpdated -> Unit
            is RustEvent.ConnectionChanged -> Logging.i("RustCore", "connection changed: $event")
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
        memberListStore.rustRangeRequester = null
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
