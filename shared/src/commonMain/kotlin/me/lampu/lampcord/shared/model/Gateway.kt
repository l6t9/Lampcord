package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*

@Serializable
data class GatewayPayload(
    val op: Int,
    val d: JsonElement? = null,
    val s: Int? = null,
    val t: String? = null
)

@Serializable
data class Identify(
    val token: String,
    val properties: Map<String, JsonElement>,
    val compress: Boolean? = null,
    val large_threshold: Int? = null,
    val capabilities: Long? = null,
    val client_state: IdentifyClientState? = null
)

@Serializable
data class IdentifyClientState(
    val guild_hashes: Map<String, JsonElement> = emptyMap(),
    val highest_last_message_id: Long = 0,
    val read_state_version: Int = 0,
    val user_guild_settings_version: Int = -1
)

@Serializable
data class VersionedModel<T>(
    val entries: List<T> = emptyList(),
    val version: Int = -1,
    val partial: Boolean = false
)

@Serializable
data class ReadyPayload(
    val v: Int,
    val user: User? = null,
    val guilds: List<Guild> = emptyList(),
    val private_channels: List<Channel> = emptyList(),
    val session_id: String = "",
    val resume_gateway_url: String? = null,
    val users: List<User>? = null,
    val read_state: VersionedModel<ReadState>? = null,
    val user_guild_settings: VersionedModel<UserGuildSettings>? = null,
    val user_settings: JsonElement? = null,
    val relationships: List<Relationship>? = null,
    val merged_members: List<List<Member>>? = null,
    val merged_presences: MergedPresences? = null,
    val sessions: List<JsonElement>? = null,
    val auth_token: String? = null,
    val analytics_token: String? = null,
    val country_code: String? = null,
    val friend_suggestion_count: Int? = null,
    val experiments: List<JsonElement>? = null
)

@Serializable
data class MergedPresences(
    val friends: List<PresenceUpdate>? = null,
    val guilds: List<List<PresenceUpdate>>? = null
)

@Serializable
data class ReadState(
    val id: String,
    var last_message_id: JsonElement? = null,
    var mention_count: Int = 0,
    val last_pin_timestamp: String? = null
)

@Serializable
data class TypingStart(
    val channel_id: String,
    val user_id: String,
    val timestamp: Long,
    val guild_id: String? = null,
    val member: Member? = null
)

@Serializable
data class Resume(
    val token: String,
    val session_id: String,
    val seq: Int
)

