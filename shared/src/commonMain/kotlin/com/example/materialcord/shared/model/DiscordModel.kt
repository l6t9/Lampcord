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
data class Message(
    val id: String,
    val channel_id: String,
    val author: User,
    val content: String,
    val timestamp: String,
    val edited_timestamp: String? = null,
    val tts: Boolean = false,
    val mention_everyone: Boolean = false
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
    val last_message_id: String? = null
)

@Serializable
data class ReadyPayload(
    val v: Int,
    val user: User,
    val guilds: List<Guild>,
    val session_id: String,
    val resume_gateway_url: String
)
