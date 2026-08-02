package com.example.materialcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class User(
    val id: String,
    val username: String,
    val discriminator: String,
    val avatar: String? = null,
    val global_name: String? = null
)

@Serializable
data class Attachment(
    val id: String,
    val filename: String,
    val description: String? = null,
    val content_type: String? = null,
    val size: Int,
    val url: String,
    val proxy_url: String,
    val height: Int? = null,
    val width: Int? = null,
    val ephemeral: Boolean? = null
)

@Serializable
data class Message(
    val id: String,
    val channel_id: String,
    val author: User,
    val content: String,
    val timestamp: String,
    val edited_timestamp: String? = null,
    val tts: Boolean = false,
    val mention_everyone: Boolean = false,
    val attachments: List<Attachment> = emptyList()
)

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
    val compress: Boolean = false,
    val large_threshold: Int = 50,
    val intents: Int = 0
)

@Serializable
data class IdentifyProperties(
    val os: String,
    val browser: String,
    val device: String
)

@Serializable
data class Guild(
    val id: String,
    val name: String? = null,
    val icon: String? = null,
    val banner: String? = null,
    val owner: Boolean? = null,
    val permissions: String? = null,
    val features: List<String>? = null
)

@Serializable
data class Channel(
    val id: String,
    val type: Int,
    val guild_id: String? = null,
    val position: Int? = null,
    val name: String? = null,
    val topic: String? = null,
    val nsfw: Boolean? = null,
    val last_message_id: String? = null,
    val parent_id: String? = null,
    val recipients: List<User>? = null,
    val icon: String? = null,
    val thread_metadata: ThreadMetadata? = null,
    val message_count: Int? = null,
    val member_count: Int? = null,
    val total_message_sent: Int? = null
)

@Serializable
data class ThreadMetadata(
    val archived: Boolean,
    val auto_archive_duration: Int,
    val archive_timestamp: String,
    val locked: Boolean,
    val invitable: Boolean? = null,
    val create_timestamp: String? = null
)

@Serializable
data class Member(
    val user: User? = null,
    val nick: String? = null,
    val avatar: String? = null,
    val roles: List<String>,
    val joined_at: String,
    val premium_since: String? = null,
    val deaf: Boolean = false,
    val mute: Boolean = false,
    val flags: Int = 0,
    val pending: Boolean? = null,
    val permissions: String? = null,
    val communication_disabled_until: String? = null
)

@Serializable
data class ThreadListResponse(
    val threads: List<Channel>,
    val members: List<ThreadMember>,
    val has_more: Boolean? = null
)

@Serializable
data class ThreadMember(
    val id: String? = null,
    val user_id: String? = null,
    val join_timestamp: String,
    val flags: Int
)

@Serializable
data class GuildFolder(
    val id: Long? = null,
    val guild_ids: List<String>,
    val name: String? = null,
    val color: Int? = null
)

@Serializable
data class UserSettings(
    val guild_positions: List<String> = emptyList(),
    val guild_folders: List<GuildFolder> = emptyList()
)

@Serializable
data class ReadyPayload(
    val v: Int,
    val user: User,
    val guilds: List<Guild>,
    val private_channels: List<Channel>,
    val session_id: String,
    val resume_gateway_url: String,
    val user_settings: UserSettings? = null
)

@Serializable
data class LoginRequest(
    val login: String,
    val password: String,
    val undelete: Boolean = false,
    val login_source: String? = null,
    val gift_code_sku_id: String? = null
)

@Serializable
data class LoginResponse(
    val user_id: String? = null,
    val token: String? = null,
    val ticket: String? = null,
    val totp: Boolean? = null,
    val sms: Boolean? = null,
    val mfa: Boolean? = null,
    val backup: Boolean? = null,
    val code: Int? = null, // Error code (60003 for MFA required)
    val message: String? = null
)

@Serializable
data class MFALoginRequest(
    val code: String,
    val ticket: String,
    val login_source: String? = null,
    val gift_code_sku_id: String? = null
)

@Serializable
data class FingerprintResponse(
    val fingerprint: String? = null
)
