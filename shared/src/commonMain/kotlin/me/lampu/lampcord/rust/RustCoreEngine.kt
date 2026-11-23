package me.lampu.lampcord.rust

import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow

expect class RustCoreEngine(config: RustClientConfig) : AutoCloseable {
    val snapshots: StateFlow<RustSnapshot>
    val events: SharedFlow<RustEventEnvelope>

    fun snapshot(): RustSnapshot
    fun drainEvents(): List<RustEventEnvelope>

    fun connectToken(token: String): Boolean
    fun connectStoredToken(): Boolean
    fun loginPassword(login: String, password: String, captchaKey: String? = null): Boolean
    fun submitMfa(code: String, ticket: String, method: String): Boolean
    fun sendMfaSms(ticket: String): Boolean
    fun forgotPassword(login: String): Boolean
    fun startRemoteAuth(): Boolean
    fun cancelRemoteAuth(): Boolean

    fun selectGuild(guildId: ULong?): Boolean
    fun selectChannel(channelId: ULong?): Boolean
    fun setMemberListRanges(ranges: List<List<Int>>): Boolean
    fun fetchMessages(channelId: ULong, before: ULong? = null, limit: UByte? = null): Boolean
    fun fetchMentions(limit: UByte? = null, before: ULong? = null): Boolean
    fun sendMessage(
        channelId: ULong,
        content: String,
        replyToMessageId: ULong? = null,
        replyToGuildId: ULong? = null,
    ): Boolean

    fun editMessage(channelId: ULong, messageId: ULong, content: String): Boolean
    fun deleteMessage(channelId: ULong, messageId: ULong): Boolean
    fun addReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean
    fun removeReaction(channelId: ULong, messageId: ULong, emoji: String): Boolean
    fun sendTyping(channelId: ULong): Boolean
    fun setStatus(status: String): Boolean
    fun ackMessage(channelId: ULong, messageId: ULong): Boolean
    fun logout(): Boolean
    fun shutdown(): Boolean
}
