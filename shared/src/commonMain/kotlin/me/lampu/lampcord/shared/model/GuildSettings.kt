package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class Ban(
    val reason: String?,
    val user: User
)

@Serializable
data class Invite(
    val code: String,
    val guild: Guild? = null,
    val channel: Channel? = null,
    val inviter: User? = null,
    val target_type: Int? = null,
    val target_user: User? = null,
    val approximate_presence_count: Int? = null,
    val approximate_member_count: Int? = null,
    val expires_at: String? = null,
    val uses: Int? = null,
    val max_uses: Int? = null,
    val max_age: Int? = null,
    val temporary: Boolean? = null,
    val created_at: String? = null
)

@Serializable
data class AuditLog(
    val audit_log_entries: List<AuditLogEntry>,
    val users: List<User>,
    val webhooks: List<Webhook>,
    val integrations: List<Integration>
)

@Serializable
data class AuditLogEntry(
    val target_id: String?,
    val changes: List<AuditLogChange>? = null,
    val user_id: String?,
    val id: String,
    val action_type: Int,
    val options: AuditLogOptions? = null,
    val reason: String? = null
)

@Serializable
data class AuditLogChange(
    val new_value: JsonElement? = null,
    val old_value: JsonElement? = null,
    val key: String
)

@Serializable
data class AuditLogOptions(
    val delete_member_days: String? = null,
    val members_removed: String? = null,
    val channel_id: String? = null,
    val message_id: String? = null,
    val count: String? = null,
    val id: String? = null,
    val type: String? = null,
    val role_name: String? = null
)

@Serializable
data class Webhook(
    val id: String,
    val type: Int,
    val guild_id: String? = null,
    val channel_id: String? = null,
    val user: User? = null,
    val name: String? = null,
    val avatar: String? = null,
    val token: String? = null,
    val application_id: String? = null,
    val source_guild: Guild? = null,
    val source_channel: Channel? = null,
    val url: String? = null
)

@Serializable
data class Integration(
    val id: String,
    val name: String,
    val type: String,
    val enabled: Boolean,
    val syncing: Boolean? = null,
    val role_id: String? = null,
    val enable_emoticons: Boolean? = null,
    val expire_behavior: Int? = null,
    val expire_grace_period: Int? = null,
    val user: User? = null,
    val account: IntegrationAccount,
    val synced_at: String? = null,
    val subscriber_count: Int? = null,
    val revoked: Boolean? = null,
    val application: IntegrationApplication? = null
)

@Serializable
data class IntegrationAccount(
    val id: String,
    val name: String
)

@Serializable
data class IntegrationApplication(
    val id: String,
    val name: String,
    val icon: String?,
    val description: String,
    val bot: User? = null
)
