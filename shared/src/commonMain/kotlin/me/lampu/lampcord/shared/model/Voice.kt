package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class VoiceState(
    val guild_id: String? = null,
    val channel_id: String? = null,
    val user_id: String,
    val member: Member? = null,
    val session_id: String,
    val deaf: Boolean,
    val mute: Boolean,
    val self_deaf: Boolean,
    val self_mute: Boolean,
    val self_stream: Boolean? = null,
    val self_video: Boolean,
    val suppress: Boolean,
    val request_to_speak_timestamp: String? = null
)

@Serializable
data class VoiceStateUpdate(
    val guild_id: String?,
    val channel_id: String?,
    val self_mute: Boolean,
    val self_deaf: Boolean,
    val self_video: Boolean? = null,
    val self_stream: Boolean? = null,
    val preferred_region: String? = null,
    val preferred_regions: List<String>? = null,
    val flags: Int? = null
)

@Serializable
data class VoiceServerUpdate(
    val token: String,
    val guild_id: String,
    val endpoint: String? = null
)
