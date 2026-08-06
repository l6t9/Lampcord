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
    val properties: IdentifyProperties,
    val compress: Boolean? = null,
    val large_threshold: Int? = null,
    val intents: Int? = null,
    val capabilities: Int? = null
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
    val design_id: Int? = null
)

@Serializable
data class ReadyPayload(
    val v: Int,
    val user: User,
    val guilds: List<Guild>,
    val private_channels: List<Channel>,
    val session_id: String,
    val resume_gateway_url: String,
    val read_state: List<ReadState>? = null,
    val user_guild_settings: List<UserGuildSettings>? = null,
    val user_settings: JsonElement? = null
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
