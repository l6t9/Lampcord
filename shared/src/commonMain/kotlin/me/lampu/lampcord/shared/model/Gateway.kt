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
data class IdentifyProperties(
    val os: String,
    val browser: String,
    val release_channel: String? = null,
    val client_version: String? = null,
    val os_version: String? = null,
    val os_arch: String? = null,
    val app_arch: String? = null,
    val system_locale: String? = null,
    val has_client_mods: Boolean? = null,
    val client_launch_id: String? = null,
    val browser_user_agent: String? = null,
    val browser_version: String? = null,
    val os_sdk_version: String? = null,
    val client_build_number: Int? = null,
    val native_build_number: Int? = null,
    val client_event_source: String? = null,
    val launch_signature: String? = null,
    val client_heartbeat_session_id: String? = null,
    val client_app_state: String? = null,
    val device: String? = null,
    val device_vendor_id: String? = null,
    val design_id: Int? = null,
    val accessibility_features: Long? = null,
    val accessibility_support_enabled: Boolean? = null,
    val client_performance_cpu: Int? = null,
    val client_performance_memory: Long? = null,
    val cpu_core_count: Int? = null
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
    val guild_id: String? = null
)

@Serializable
data class Resume(
    val token: String,
    val session_id: String,
    val seq: Int
)

@Serializable
data class ReadySupplementalPayload(
    val guilds: List<Guild> = emptyList(),
    val merged_members: List<List<Member>>? = null,
    val merged_presences: MergedPresences? = null
)
