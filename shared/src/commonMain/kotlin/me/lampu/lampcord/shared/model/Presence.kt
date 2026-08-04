package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class PresenceUpdate(
    val user: User,
    val guild_id: String? = null,
    val status: String = "offline",
    val activities: List<Activity> = emptyList(),
    val client_status: ClientStatus = ClientStatus()
)

@Serializable
data class Activity(
    val name: String,
    val type: Int,
    val url: String? = null,
    val created_at: Long = 0L,
    val timestamps: ActivityTimestamps? = null,
    val application_id: String? = null,
    val details: String? = null,
    val state: String? = null,
    val emoji: Emoji? = null,
    val party: ActivityParty? = null,
    val assets: ActivityAssets? = null,
    val secrets: ActivitySecrets? = null,
    val instance: Boolean? = null,
    val flags: Int? = null,
    val buttons: List<JsonElement>? = null
)

@Serializable
data class ActivityTimestamps(val start: Long? = null, val end: Long? = null)

@Serializable
data class ActivityParty(val id: String? = null, val size: List<Int>? = null)

@Serializable
data class ActivityAssets(val large_image: String? = null, val large_text: String? = null, val small_image: String? = null, val small_text: String? = null)

@Serializable
data class ActivitySecrets(val join: String? = null, val spectate: String? = null, val match: String? = null)

@Serializable
data class ActivityButton(val label: String, val url: String)

@Serializable
data class ClientStatus(
    val desktop: String? = null,
    val mobile: String? = null,
    val web: String? = null
)
