package me.lampu.lampcord.rust

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
import uniffi.lampcord_ffi.MemberSnapshot as FfiMemberSnapshot
import uniffi.lampcord_ffi.MessageSnapshot as FfiMessageSnapshot
import uniffi.lampcord_ffi.PresenceSnapshot as FfiPresenceSnapshot
import uniffi.lampcord_ffi.PlatformSecretStore
import uniffi.lampcord_ffi.ReadStateSnapshot as FfiReadStateSnapshot
import uniffi.lampcord_ffi.RemoteAuthState as FfiRemoteAuthState

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

    private val _snapshots = MutableStateFlow(mapSnapshot(client.snapshot()))
    actual val snapshots: StateFlow<RustSnapshot> = _snapshots.asStateFlow()

    private val _events = MutableSharedFlow<RustEventEnvelope>(extraBufferCapacity = 64)
    actual val events: SharedFlow<RustEventEnvelope> = _events.asSharedFlow()

    private var pollJob: Job? = scope.launch {
        var lastSeq = _snapshots.value.seq
        while (isActive) {
            for (event in runCatching { client.drainEvents() }.getOrDefault(emptyList())) {
                _events.tryEmit(mapEvent(event))
            }
            val snapshot = runCatching { client.snapshot() }.getOrNull()
            if (snapshot != null && snapshot.seq != lastSeq) {
                lastSeq = snapshot.seq
                _snapshots.value = mapSnapshot(snapshot)
            }
            delay(100L)
        }
    }

    actual fun snapshot(): RustSnapshot = mapSnapshot(client.snapshot())

    actual fun drainEvents(): List<RustEventEnvelope> =
        runCatching { client.drainEvents() }
            .getOrDefault(emptyList())
            .map(::mapEvent)

    actual fun connectToken(token: String): Boolean = runCatching { client.connectToken(token) }.isSuccess
    actual fun connectStoredToken(): Boolean = runCatching { client.connectStoredToken() }.isSuccess

    actual fun loginPassword(login: String, password: String, captchaKey: String?): Boolean =
        runCatching { client.loginPassword(login, password, captchaKey) }.isSuccess

    actual fun submitMfa(code: String, ticket: String, method: String): Boolean =
        runCatching { client.submitMfa(code, ticket, method) }.isSuccess

    actual fun sendMfaSms(ticket: String): Boolean = runCatching { client.sendMfaSms(ticket) }.isSuccess
    actual fun forgotPassword(login: String): Boolean = runCatching { client.forgotPassword(login) }.isSuccess
    actual fun startRemoteAuth(): Boolean = runCatching { client.startRemoteAuth() }.isSuccess
    actual fun cancelRemoteAuth(): Boolean = runCatching { client.cancelRemoteAuth() }.isSuccess

    actual fun selectGuild(guildId: ULong?): Boolean = runCatching { client.selectGuild(guildId) }.isSuccess
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
)

private fun mapMessage(value: FfiMessageSnapshot): RustMessage = RustMessage(
    id = value.id,
    channelId = value.channelId,
    authorId = value.authorId,
    authorName = value.authorName,
    content = value.content,
    timestamp = value.timestamp,
    editedTimestamp = value.editedTimestamp,
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

private fun mapPresence(value: FfiPresenceSnapshot): RustPresence =
    RustPresence(value.userId, value.status, value.activities)

private fun mapReadState(value: FfiReadStateSnapshot): RustReadState = RustReadState(
    channelId = value.channelId,
    lastMessageId = value.lastMessageId,
    lastReadMessageId = value.lastReadMessageId,
    mentionCount = value.mentionCount,
    unread = value.unread,
)

private fun mapSnapshot(value: FfiClientSnapshot): RustSnapshot = RustSnapshot(
    schemaVersion = value.schemaVersion,
    seq = value.seq,
    connection = mapConnection(value.connection),
    hasToken = value.hasToken,
    currentUserId = value.currentUserId,
    currentUser = value.currentUser,
    selectedGuildId = value.selectedGuildId,
    selectedChannelId = value.selectedChannelId,
    guilds = value.guilds.map(::mapGuild),
    channels = value.channels.map(::mapChannel),
    messages = value.messages.map(::mapMessage),
    members = value.members.map(::mapMember),
    presences = value.presences.map(::mapPresence),
    readStates = value.readStates.map(::mapReadState),
    auth = mapAuthState(value.auth),
    error = value.error,
)

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
