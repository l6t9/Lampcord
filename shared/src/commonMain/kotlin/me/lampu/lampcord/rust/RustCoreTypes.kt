package me.lampu.lampcord.rust

data class RustClientConfig(
    val apiBase: String = "https://discord.com/api/v9",
    val gatewayUrl: String? = null,
    val userAgent: String? = null,
    val storagePath: String? = null,
)

enum class RustConnectionState {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    READY,
    RESUMING,
    ERROR,
}

data class RustConnection(
    val state: RustConnectionState,
    val message: String? = null,
)

enum class RustAuthFailureKind {
    INVALID_CREDENTIALS,
    SUSPENDED,
    RATE_LIMITED,
    SERVER_ERROR,
    VERIFICATION_REQUIRED,
    ACCOUNT_DISABLED,
    SCHEDULED_FOR_DELETION,
    CAPTCHA_REJECTED,
    CAPTCHA_CANCELLED,
    PASSWORD_UPDATE_REQUIRED,
    NETWORK,
    UNKNOWN,
}

sealed class RustAuthChallenge {
    data class Success(val token: String) : RustAuthChallenge()
    data class MfaRequired(val ticket: String, val methods: List<String>) : RustAuthChallenge()
    data class CaptchaRequired(
        val sitekey: String,
        val service: String,
        val rqdata: String?,
        val sessionId: String?,
        val invisible: Boolean,
    ) : RustAuthChallenge()

    data object PasswordUpdateRequired : RustAuthChallenge()
    data class SmsSent(val phone: String) : RustAuthChallenge()
    data class ResetSent(val phoneVerificationRequired: Boolean) : RustAuthChallenge()
    data class Failed(val kind: RustAuthFailureKind, val message: String) : RustAuthChallenge()
}

sealed class RustRemoteAuthState {
    data object Idle : RustRemoteAuthState()
    data object Connecting : RustRemoteAuthState()
    data class QrReady(val url: String) : RustRemoteAuthState()
    data class UserScanned(val userId: String, val username: String) : RustRemoteAuthState()
    data class Finished(val token: String) : RustRemoteAuthState()
    data class Error(val message: String) : RustRemoteAuthState()
}

data class RustAuthState(
    val challenge: RustAuthChallenge? = null,
    val remote: RustRemoteAuthState = RustRemoteAuthState.Idle,
)

data class RustGuild(
    val id: ULong,
    val name: String,
    val iconUrl: String?,
)

data class RustChannel(
    val id: ULong,
    val guildId: ULong?,
    val name: String,
    val kind: UByte,
    val parentId: ULong?,
    val position: Int?,
    val canView: Boolean = true,
    val icon: String? = null,
    val recipients: List<RustRecipient> = emptyList(),
    val canSend: Boolean = true,
)

data class RustRecipient(
    val id: ULong,
    val username: String,
    val globalName: String?,
    val avatar: String?,
)

data class RustMessage(
    val id: ULong,
    val channelId: ULong,
    val authorId: ULong?,
    val authorName: String,
    val content: String,
    val timestamp: String,
    val editedTimestamp: String?,
    val authorAvatar: String? = null,
)

data class RustMember(
    val guildId: ULong,
    val userId: ULong,
    val username: String,
    val avatar: String?,
    val nickname: String?,
    val roles: List<ULong>,
    val status: String?,
)

data class RustPresence(
    val userId: ULong,
    val status: String,
    val activities: List<String>,
)

data class RustReadState(
    val channelId: ULong,
    val lastMessageId: ULong?,
    val lastReadMessageId: ULong?,
    val mentionCount: UInt,
    val unread: Boolean,
)

data class RustSnapshotVersions(
    val seq: ULong = 0u,
    val meta: ULong = 0u,
    val guilds: ULong = 0u,
    val channels: ULong = 0u,
    val messages: ULong = 0u,
    val members: ULong = 0u,
    val presences: ULong = 0u,
    val readStates: ULong = 0u,
)

data class RustSnapshotMetaPatch(
    val connection: RustConnection,
    val hasToken: Boolean,
    val currentUserId: ULong?,
    val currentUser: String?,
    val currentUserAvatar: String? = null,
    val selectedGuildId: ULong?,
    val selectedChannelId: ULong?,
    val auth: RustAuthState,
    val error: String?,
)

data class RustSnapshot(
    val schemaVersion: UInt,
    val seq: ULong,
    val connection: RustConnection,
    val hasToken: Boolean,
    val currentUserId: ULong?,
    val currentUser: String?,
    val currentUserAvatar: String? = null,
    val selectedGuildId: ULong?,
    val selectedChannelId: ULong?,
    val guilds: List<RustGuild>,
    val channels: List<RustChannel>,
    val messages: List<RustMessage>,
    val members: List<RustMember>,
    val presences: List<RustPresence>,
    val readStates: List<RustReadState>,
    val auth: RustAuthState,
    val error: String?,
    val memberList: RustMemberList = RustMemberList(),
    val relationships: List<RustRelationship> = emptyList(),
    val versions: RustSnapshotVersions = RustSnapshotVersions(),
)

data class RustMemberListRow(
    val kind: Int,
    val groupId: String? = null,
    val groupCount: Int? = null,
    val member: RustMember? = null,
)

data class RustMemberList(
    val rows: List<RustMemberListRow> = emptyList(),
    val onlineCount: Int? = null,
    val memberCount: Int? = null,
)

enum class RustErrorKind {
    INVALID_INPUT,
    NOT_CONNECTED,
    ALREADY_CONNECTED,
    UNAUTHORIZED,
    FORBIDDEN,
    NOT_FOUND,
    RATE_LIMITED,
    TIMEOUT,
    CAPTCHA_REQUIRED,
    MFA_REQUIRED,
    CANCELLED,
    TRANSPORT,
    PROTOCOL,
    STORAGE,
    INTERNAL,
    LOCK_POISONED,
}

sealed class RustEvent {
    data object SnapshotUpdated : RustEvent()
    data class ConnectionChanged(val connection: RustConnection) : RustEvent()
    data class AuthChanged(val challenge: RustAuthChallenge) : RustEvent()
    data class RemoteAuthChanged(val state: RustRemoteAuthState) : RustEvent()
    data class MessageUpdated(val message: RustMessage) : RustEvent()
    data class MessageDeleted(val channelId: ULong, val messageId: ULong) : RustEvent()
    data class ReactionChanged(
        val channelId: ULong,
        val messageId: ULong,
        val emoji: String,
        val added: Boolean,
    ) : RustEvent()

    data class TypingStarted(val channelId: ULong, val userId: ULong) : RustEvent()
    data class PresenceUpdated(val presence: RustPresence) : RustEvent()
    data class NotificationReceived(val message: RustMessage) : RustEvent()
    data class MentionsLoaded(val count: UInt) : RustEvent()
    data class CommandFailed(val kind: RustErrorKind, val message: String) : RustEvent()
    data class Error(val message: String) : RustEvent()
    data object LoggedOut : RustEvent()
}

data class RustEventEnvelope(
    val sequence: ULong,
    val emittedAtMs: ULong,
    val event: RustEvent,
)

data class RustRelationship(
    val userId: ULong,
    val kind: Int,
    val username: String,
    val globalName: String?,
    val avatar: String?,
    val avatarDecoration: String?,
)
