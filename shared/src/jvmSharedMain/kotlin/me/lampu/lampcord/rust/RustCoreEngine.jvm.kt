package me.lampu.lampcord.rust

import me.lampu.lampcord.shared.utils.Logging
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.RustCoreMode
import uniffi.lampcord_ffi.AuthChallenge as FfiAuthChallenge
import uniffi.lampcord_ffi.AuthFailureKind as FfiAuthFailureKind
import uniffi.lampcord_ffi.AuthStateSnapshot as FfiAuthStateSnapshot
import uniffi.lampcord_ffi.ChannelSnapshot as FfiChannelSnapshot
import uniffi.lampcord_ffi.ClientConfig as FfiClientConfig
import uniffi.lampcord_ffi.ClientEvent as FfiClientEvent
import uniffi.lampcord_ffi.ClientEventEnvelope as FfiClientEventEnvelope
import uniffi.lampcord_ffi.ClientSnapshot as FfiClientSnapshot
import uniffi.lampcord_ffi.ConnectionState as FfiConnectionState
import uniffi.lampcord_ffi.FfiErrorKind as FfiErrorKind
import uniffi.lampcord_ffi.GuildSnapshot as FfiGuildSnapshot
import uniffi.lampcord_ffi.LampcordClient
import uniffi.lampcord_ffi.MemberListRowSnapshot as FfiMemberListRowSnapshot
import uniffi.lampcord_ffi.MemberListSnapshot as FfiMemberListSnapshot
import uniffi.lampcord_ffi.MemberSnapshot as FfiMemberSnapshot
import uniffi.lampcord_ffi.RelationshipSnapshot as FfiRelationshipSnapshot
import uniffi.lampcord_ffi.MessageSnapshot as FfiMessageSnapshot
import uniffi.lampcord_ffi.PresenceSnapshot as FfiPresenceSnapshot
import uniffi.lampcord_ffi.PlatformSecretStore
import uniffi.lampcord_ffi.ReadStateSnapshot as FfiReadStateSnapshot
import uniffi.lampcord_ffi.RemoteAuthState as FfiRemoteAuthState
import uniffi.lampcord_ffi.SnapshotMetaPatch as FfiSnapshotMetaPatch
import uniffi.lampcord_ffi.SnapshotPatch as FfiSnapshotPatch
import uniffi.lampcord_ffi.SnapshotVersions as FfiSnapshotVersions

private class SettingsSecretStore : PlatformSecretStore {
    override fun load(): String? = Settings.shared.rustCoreSecrets.takeIf { it.isNotEmpty() }

    override fun save(value: String) {
        Settings.shared.rustCoreSecrets = value
    }

    override fun clear() {
        Settings.shared.rustCoreSecrets = ""
    }
}

actual class RustCoreEngine actual constructor(config: RustClientConfig) : AutoCloseable {
    private val ffiConfig = FfiClientConfig(
        apiBase = config.apiBase,
        gatewayUrl = config.gatewayUrl,
        userAgent = config.userAgent,
        storagePath = config.storagePath,
    )
    private val client = run {
        RustCoreMode.requireEnabled("constructor")
        LampcordClient.startWithStore(ffiConfig, SettingsSecretStore())
    }
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _snapshots = MutableStateFlow(
        mapSnapshot(client.snapshot(), client.snapshotVersions()),
    )
    actual val snapshots: StateFlow<RustSnapshot> = _snapshots.asStateFlow()

    private val _events = MutableSharedFlow<RustEventEnvelope>(extraBufferCapacity = 64)
    actual val events: SharedFlow<RustEventEnvelope> = _events.asSharedFlow()

    private var pollJob: Job? = scope.launch {
        var lastVersions = client.snapshotVersions()
        while (isActive) {
            for (event in runCatching { client.drainEvents() }.getOrDefault(emptyList())) {
                _events.tryEmit(mapEvent(event))
            }
            val versions = runCatching { client.snapshotVersions() }.getOrNull()
            if (versions != null && versions.seq != lastVersions.seq) {
                val patch = runCatching { client.snapshotPatch(lastVersions) }.getOrNull()
                if (patch != null) {
                    lastVersions = patch.versions
                    _snapshots.value = mapSnapshotPatch(patch, _snapshots.value)
                }
            }
            delay(100L)
        }
    }

    actual fun snapshot(): RustSnapshot = mapSnapshot(client.snapshot(), client.snapshotVersions())

    actual fun drainEvents(): List<RustEventEnvelope> =
        runCatching { client.drainEvents() }
            .getOrDefault(emptyList())
            .map(::mapEvent)

    actual fun connectToken(token: String): Boolean =
        runCatching { client.connectToken(token) }
            .onFailure { Logging.w("RustCore", "connectToken failed: ${it.message}", it) }
            .isSuccess
    actual fun connectStoredToken(): Boolean =
        runCatching { client.connectStoredToken() }
            .onFailure { Logging.w("RustCore", "connectStoredToken failed: ${it.message}", it) }
            .isSuccess

    actual fun loginPassword(login: String, password: String, captchaKey: String?): Boolean =
        runCatching { client.loginPassword(login, password, captchaKey) }.isSuccess

    actual fun submitMfa(code: String, ticket: String, method: String): Boolean =
        runCatching { client.submitMfa(code, ticket, method) }.isSuccess

    actual fun sendMfaSms(ticket: String): Boolean = runCatching { client.sendMfaSms(ticket) }.isSuccess
    actual fun forgotPassword(login: String): Boolean = runCatching { client.forgotPassword(login) }.isSuccess
    actual fun startRemoteAuth(): Boolean = runCatching { client.startRemoteAuth() }.isSuccess
    actual fun cancelRemoteAuth(): Boolean = runCatching { client.cancelRemoteAuth() }.isSuccess

    actual fun selectGuild(guildId: ULong?): Boolean = runCatching { client.selectGuild(guildId) }.isSuccess
    actual fun setMemberListRanges(ranges: List<List<Int>>): Boolean =
        runCatching { client.setMemberListRanges(ranges.map { range -> range.map(Int::toUInt) }) }.isSuccess
    actual fun selectChannel(channelId: ULong?): Boolean = runCatching { client.selectChannel(channelId) }.isSuccess

    actual fun fetchMessages(channelId: ULong, before: ULong?, limit: UByte?): Boolean =
        runCatching { client.fetchMessages(channelId, before, limit) }.isSuccess

    actual fun fetchMentions(limit: UByte?, before: ULong?): Boolean =
        runCatching { client.fetchMentions(limit, before) }.isSuccess

    actual fun sendMessage(
        channelId: ULong,
        content: String,
        replyToMessageId: ULong?,
        replyToGuildId: ULong?,
    ): Boolean = runCatching {
        client.sendMessage(channelId, content, replyToMessageId, replyToGuildId)
    }.isSuccess

    actual fun editMessage(channelId: ULong, messageId: ULong, content: String): Boolean =
        runCatching { client.editMessage(channelId, messageId, content) }.isSuccess

    actual fun deleteMessage(channelId: ULong, messageId: ULong): Boolean =
        runCatching { client.deleteMessage(channelId, messageId) }.isSuccess

    actual fun addReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean =
        runCatching { client.addReaction(channelId, messageId, emoji) }.isSuccess

    actual fun removeReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean =
        runCatching { client.removeReaction(channelId, messageId, emoji) }.isSuccess

    actual fun sendTyping(channelId: ULong): Boolean = runCatching { client.sendTyping(channelId) }.isSuccess
    actual fun setStatus(status: String): Boolean = runCatching { client.setStatus(status) }.isSuccess
    actual fun ackMessage(channelId: ULong, messageId: ULong): Boolean =
        runCatching { client.ackMessage(channelId, messageId) }.isSuccess

    actual fun logout(): Boolean = runCatching { client.logout() }.isSuccess

    actual fun shutdown(): Boolean {
        pollJob?.cancel()
        pollJob = null
        scope.cancel()
        return runCatching { client.shutdown() }.isSuccess
    }

    override fun close() {
        shutdown()
    }
}

private fun mapConnection(value: FfiConnectionState): RustConnection = when (value) {
    FfiConnectionState.Disconnected -> RustConnection(RustConnectionState.DISCONNECTED)
    FfiConnectionState.Connecting -> RustConnection(RustConnectionState.CONNECTING)
    FfiConnectionState.Connected -> RustConnection(RustConnectionState.CONNECTED)
    FfiConnectionState.Ready -> RustConnection(RustConnectionState.READY)
    FfiConnectionState.Resuming -> RustConnection(RustConnectionState.RESUMING)
    is FfiConnectionState.Error -> RustConnection(RustConnectionState.ERROR, value.message)
}

private fun mapFailureKind(value: FfiAuthFailureKind): RustAuthFailureKind =
    runCatching { RustAuthFailureKind.valueOf(value.name) }.getOrDefault(RustAuthFailureKind.UNKNOWN)

private fun mapErrorKind(value: FfiErrorKind): RustErrorKind =
    runCatching { RustErrorKind.valueOf(value.name) }.getOrDefault(RustErrorKind.INTERNAL)

private fun mapAuthChallenge(value: FfiAuthChallenge): RustAuthChallenge = when (value) {
    is FfiAuthChallenge.Success -> RustAuthChallenge.Success(value.token)
    is FfiAuthChallenge.MfaRequired -> RustAuthChallenge.MfaRequired(value.ticket, value.methods)
    is FfiAuthChallenge.CaptchaRequired -> RustAuthChallenge.CaptchaRequired(
        sitekey = value.sitekey,
        service = value.service,
        rqdata = value.rqdata,
        sessionId = value.sessionId,
        invisible = value.invisible,
    )
    FfiAuthChallenge.PasswordUpdateRequired -> RustAuthChallenge.PasswordUpdateRequired
    is FfiAuthChallenge.SmsSent -> RustAuthChallenge.SmsSent(value.phone)
    is FfiAuthChallenge.ResetSent -> RustAuthChallenge.ResetSent(value.phoneVerificationRequired)
    is FfiAuthChallenge.Failed -> RustAuthChallenge.Failed(mapFailureKind(value.kind), value.message)
}

private fun mapRemoteAuth(value: FfiRemoteAuthState): RustRemoteAuthState = when (value) {
    FfiRemoteAuthState.Idle -> RustRemoteAuthState.Idle
    FfiRemoteAuthState.Connecting -> RustRemoteAuthState.Connecting
    is FfiRemoteAuthState.QrReady -> RustRemoteAuthState.QrReady(value.url)
    is FfiRemoteAuthState.UserScanned -> RustRemoteAuthState.UserScanned(value.userId, value.username)
    is FfiRemoteAuthState.Finished -> RustRemoteAuthState.Finished(value.token)
    is FfiRemoteAuthState.Error -> RustRemoteAuthState.Error(value.message)
}

private fun mapAuthState(value: FfiAuthStateSnapshot): RustAuthState = RustAuthState(
    challenge = value.challenge?.let(::mapAuthChallenge),
    remote = mapRemoteAuth(value.remote),
)

private fun mapGuild(value: FfiGuildSnapshot): RustGuild =
    RustGuild(value.id, value.name, value.iconUrl)

private fun mapChannel(value: FfiChannelSnapshot): RustChannel = RustChannel(
    id = value.id,
    guildId = value.guildId,
    name = value.name,
    kind = value.kind,
    parentId = value.parentId,
    position = value.position,
    canView = value.canView,
    canSend = value.canSend,
    icon = value.icon,
    recipients = value.recipients.map { recipient ->
        RustRecipient(
            id = recipient.id,
            username = recipient.username,
            globalName = recipient.globalName,
            avatar = recipient.avatar,
        )
    },
)

private fun mapMessage(value: FfiMessageSnapshot): RustMessage = RustMessage(
    id = value.id,
    channelId = value.channelId,
    authorId = value.authorId,
    authorName = value.authorName,
    content = value.content,
    timestamp = value.timestamp,
    editedTimestamp = value.editedTimestamp,
    authorAvatar = value.authorAvatar,
)

private fun mapMember(value: FfiMemberSnapshot): RustMember = RustMember(
    guildId = value.guildId,
    userId = value.userId,
    username = value.username,
    avatar = value.avatar,
    nickname = value.nickname,
    roles = value.roles,
    status = value.status,
)

private fun mapMemberListRow(value: FfiMemberListRowSnapshot): RustMemberListRow = RustMemberListRow(
    kind = value.kind.toInt(),
    groupId = value.groupId,
    groupCount = value.groupCount?.toInt(),
    member = value.member?.let(::mapMember),
)

private fun mapRelationship(value: FfiRelationshipSnapshot): RustRelationship = RustRelationship(
    userId = value.userId,
    kind = value.kind.toInt(),
    username = value.username,
    globalName = value.globalName,
    avatar = value.avatar,
    avatarDecoration = value.avatarDecoration,
)

private fun mapMemberList(value: FfiMemberListSnapshot): RustMemberList = RustMemberList(
    rows = value.rows.map(::mapMemberListRow),
    onlineCount = value.onlineCount?.toInt(),
    memberCount = value.memberCount?.toInt(),
)

private fun mapPresence(value: FfiPresenceSnapshot): RustPresence =
    RustPresence(value.userId, value.status, value.activities)

private fun mapReadState(value: FfiReadStateSnapshot): RustReadState = RustReadState(
    channelId = value.channelId,
    lastMessageId = value.lastMessageId,
    lastReadMessageId = value.lastReadMessageId,
    mentionCount = value.mentionCount,
    unread = value.unread,
)

private fun mapVersions(value: FfiSnapshotVersions): RustSnapshotVersions = RustSnapshotVersions(
    seq = value.seq,
    meta = value.meta,
    guilds = value.guilds,
    channels = value.channels,
    messages = value.messages,
    members = value.members,
    presences = value.presences,
    readStates = value.readStates,
)

private fun mapMeta(value: FfiSnapshotMetaPatch): RustSnapshotMetaPatch = RustSnapshotMetaPatch(
    connection = mapConnection(value.connection),
    hasToken = value.hasToken,
    currentUserId = value.currentUserId,
    currentUser = value.currentUser,
    currentUserAvatar = value.currentUserAvatar,
    selectedGuildId = value.selectedGuildId,
    selectedChannelId = value.selectedChannelId,
    auth = mapAuthState(value.auth),
    error = value.error,
)

private fun mapSnapshot(
    value: FfiClientSnapshot,
    versions: FfiSnapshotVersions,
): RustSnapshot = RustSnapshot(
    schemaVersion = value.schemaVersion,
    seq = value.seq,
    connection = mapConnection(value.connection),
    hasToken = value.hasToken,
    currentUserId = value.currentUserId,
    currentUser = value.currentUser,
    currentUserAvatar = value.currentUserAvatar,
    selectedGuildId = value.selectedGuildId,
    selectedChannelId = value.selectedChannelId,
    guilds = value.guilds.map(::mapGuild),
    channels = value.channels.map(::mapChannel),
    messages = value.messages.map(::mapMessage),
    members = value.members.map(::mapMember),
    presences = value.presences.map(::mapPresence),
    readStates = value.readStates.map(::mapReadState),
    memberList = mapMemberList(value.memberList),
    relationships = value.relationships.map(::mapRelationship),
    auth = mapAuthState(value.auth),
    error = value.error,
    versions = mapVersions(versions),
)

private fun mapSnapshotPatch(value: FfiSnapshotPatch, previous: RustSnapshot): RustSnapshot {
    val meta = value.meta?.let(::mapMeta)
    return previous.copy(
        schemaVersion = value.schemaVersion,
        seq = value.seq,
        connection = meta?.connection ?: previous.connection,
        hasToken = meta?.hasToken ?: previous.hasToken,
        currentUserId = meta?.currentUserId ?: previous.currentUserId,
        currentUser = meta?.currentUser ?: previous.currentUser,
        currentUserAvatar = meta?.currentUserAvatar ?: previous.currentUserAvatar,
        selectedGuildId = meta?.selectedGuildId ?: previous.selectedGuildId,
        selectedChannelId = meta?.selectedChannelId ?: previous.selectedChannelId,
        guilds = value.guilds?.map(::mapGuild) ?: previous.guilds,
        channels = value.channels?.map(::mapChannel) ?: previous.channels,
        messages = value.messages?.map(::mapMessage) ?: previous.messages,
        members = value.members?.map(::mapMember) ?: previous.members,
        presences = value.presences?.map(::mapPresence) ?: previous.presences,
        readStates = value.readStates?.map(::mapReadState) ?: previous.readStates,
        memberList = value.memberList?.let(::mapMemberList) ?: previous.memberList,
        relationships = value.relationships.map(::mapRelationship),
        auth = meta?.auth ?: previous.auth,
        error = meta?.error ?: previous.error,
        versions = mapVersions(value.versions),
    )
}

private fun mapEvent(value: FfiClientEventEnvelope): RustEventEnvelope = RustEventEnvelope(
    sequence = value.sequence,
    emittedAtMs = value.emittedAtMs,
    event = when (val event = value.event) {
        FfiClientEvent.SnapshotUpdated -> RustEvent.SnapshotUpdated
        is FfiClientEvent.ConnectionChanged -> RustEvent.ConnectionChanged(mapConnection(event.v1))
        is FfiClientEvent.AuthChanged -> RustEvent.AuthChanged(mapAuthChallenge(event.v1))
        is FfiClientEvent.RemoteAuthChanged -> RustEvent.RemoteAuthChanged(mapRemoteAuth(event.v1))
        is FfiClientEvent.MessageUpdated -> RustEvent.MessageUpdated(mapMessage(event.v1))
        is FfiClientEvent.MessageDeleted -> RustEvent.MessageDeleted(event.channelId, event.messageId)
        is FfiClientEvent.ReactionChanged -> RustEvent.ReactionChanged(
            event.channelId,
            event.messageId,
            event.emoji,
            event.added,
        )
        is FfiClientEvent.TypingStarted -> RustEvent.TypingStarted(event.channelId, event.userId)
        is FfiClientEvent.PresenceUpdated -> RustEvent.PresenceUpdated(mapPresence(event.v1))
        is FfiClientEvent.NotificationReceived -> RustEvent.NotificationReceived(mapMessage(event.v1))
        is FfiClientEvent.MentionsLoaded -> RustEvent.MentionsLoaded(event.count)
        is FfiClientEvent.CommandFailed -> RustEvent.CommandFailed(
            mapErrorKind(event.kind),
            event.message,
        )
        is FfiClientEvent.Error -> RustEvent.Error(event.message)
        FfiClientEvent.LoggedOut -> RustEvent.LoggedOut
    },
)
