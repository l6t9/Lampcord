package me.lampu.lampcord.rust

import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import me.lampu.lampcord.shared.state.RustCoreMode

private fun unavailable(operation: String): Nothing {
    RustCoreMode.requireEnabled(operation)
    throw UnsupportedOperationException("Rust core native library is not bundled for Android: $operation")
}

actual class RustCoreEngine actual constructor(config: RustClientConfig) : AutoCloseable {
    actual val snapshots: StateFlow<RustSnapshot>
        get() = unavailable("snapshots")

    actual val events: SharedFlow<RustEventEnvelope>
        get() = unavailable("events")

    actual fun snapshot(): RustSnapshot = unavailable("snapshot")
    actual fun drainEvents(): List<RustEventEnvelope> = unavailable("drainEvents")

    actual fun connectToken(token: String): Boolean = unavailable("connectToken")
    actual fun connectStoredToken(): Boolean = unavailable("connectStoredToken")
    actual fun loginPassword(login: String, password: String, captchaKey: String?): Boolean =
        unavailable("loginPassword")

    actual fun submitMfa(code: String, ticket: String, method: String): Boolean = unavailable("submitMfa")
    actual fun sendMfaSms(ticket: String): Boolean = unavailable("sendMfaSms")
    actual fun forgotPassword(login: String): Boolean = unavailable("forgotPassword")
    actual fun startRemoteAuth(): Boolean = unavailable("startRemoteAuth")
    actual fun cancelRemoteAuth(): Boolean = unavailable("cancelRemoteAuth")

    actual fun selectGuild(guildId: ULong?): Boolean = unavailable("selectGuild")
    actual fun selectChannel(channelId: ULong?): Boolean = unavailable("selectChannel")
    actual fun fetchMessages(channelId: ULong, before: ULong?, limit: UByte?): Boolean =
        unavailable("fetchMessages")

    actual fun fetchMentions(limit: UByte?, before: ULong?): Boolean = unavailable("fetchMentions")

    actual fun sendMessage(
        channelId: ULong,
        content: String,
        replyToMessageId: ULong?,
        replyToGuildId: ULong?,
    ): Boolean = unavailable("sendMessage")

    actual fun editMessage(channelId: ULong, messageId: ULong, content: String): Boolean =
        unavailable("editMessage")

    actual fun deleteMessage(channelId: ULong, messageId: ULong): Boolean = unavailable("deleteMessage")
    actual fun addReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean =
        unavailable("addReaction")

    actual fun removeReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean =
        unavailable("removeReaction")

    actual fun sendTyping(channelId: ULong): Boolean = unavailable("sendTyping")
    actual fun setStatus(status: String): Boolean = unavailable("setStatus")
    actual fun ackMessage(channelId: ULong, messageId: ULong): Boolean = unavailable("ackMessage")
    actual fun logout(): Boolean = unavailable("logout")
    actual fun shutdown(): Boolean = unavailable("shutdown")

    override fun close() {
    }
}
