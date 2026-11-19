package me.lampu.lampcord.shared.model

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/**
 * Discord sends timestamps as either JSON numbers or numeric strings. Keeping this
 * local avoids depending on a globally lenient Json configuration.
 */
@OptIn(ExperimentalSerializationApi::class)
object NullableLongSerializer : KSerializer<Long?> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("NullableLong", PrimitiveKind.LONG)

    override fun deserialize(decoder: Decoder): Long? {
        val jsonDecoder = decoder as? JsonDecoder ?: return decoder.decodeLong()
        return when (val element = jsonDecoder.decodeJsonElement()) {
            JsonNull -> null
            is JsonPrimitive -> element.content.toLongOrNull() ?: element.contentOrNull?.toLongOrNull()
            else -> null
        }
    }

    override fun serialize(encoder: Encoder, value: Long?) {
        if (value == null) encoder.encodeNull() else encoder.encodeLong(value)
    }
}

@Serializable
data class PresenceUpdate(
    val user: User? = null,
    val user_id: String? = null,
    val guild_id: String? = null,
    val status: String = "offline",
    val activities: List<Activity> = emptyList(),
    val hidden_activities: List<Activity> = emptyList(),
    val has_played_game: Boolean? = null,
    val client_status: ClientStatus = ClientStatus()
)

@Serializable
data class Activity(
    val id: String? = null,
    val name: String,
    val type: Int,
    val url: String? = null,
    @Serializable(with = NullableLongSerializer::class)
    val created_at: Long? = null,
    val timestamps: ActivityTimestamps? = null,
    val application_id: String? = null,
    val parent_application_id: String? = null,
    val details: String? = null,
    val state: String? = null,
    val emoji: Emoji? = null,
    val party: ActivityParty? = null,
    val assets: ActivityAssets? = null,
    val secrets: ActivitySecrets? = null,
    val instance: Boolean? = null,
    val flags: Int? = null,
    val buttons: List<JsonElement>? = null,
    val session_id: String? = null,
    val platform: String? = null,
    val supported_platforms: List<String>? = null,
    val status_display_type: Int? = null,
    val details_url: String? = null,
    val state_url: String? = null,
    val sync_id: String? = null,
    val metadata: ActivityMetadata? = null
) {
    companion object {
        const val TYPE_PLAYING: Int = 0
        const val TYPE_STREAMING: Int = 1
        const val TYPE_LISTENING: Int = 2
        const val TYPE_WATCHING: Int = 3
        const val TYPE_CUSTOM_STATUS: Int = 4
        const val TYPE_COMPETING: Int = 5
        const val TYPE_HANG: Int = 6
    }
}

@Serializable
data class ActivityTimestamps(
    @Serializable(with = NullableLongSerializer::class)
    val start: Long? = null,
    @Serializable(with = NullableLongSerializer::class)
    val end: Long? = null
)

@Serializable
data class ActivityParty(
    val id: String? = null,
    val size: List<Int>? = null,
    val privacy: Int? = null
)

@Serializable
data class ActivityAssets(
    val large_image: String? = null,
    val large_text: String? = null,
    val small_image: String? = null,
    val small_text: String? = null,
    val large_url: String? = null,
    val small_url: String? = null,
    val invite_cover_image: String? = null
)

@Serializable
data class ActivitySecrets(
    val join: String? = null,
    val spectate: String? = null,
    val match: String? = null
)

@Serializable
data class ActivityButton(val label: String, val url: String)

@Serializable
data class ActivityMetadata(
    val button_urls: List<String>? = null,
    val artist_ids: List<String>? = null,
    val album_id: String? = null,
    val context_uri: String? = null,
    val type: String? = null
)

@Serializable
data class ClientStatus(
    val desktop: String? = null,
    val mobile: String? = null,
    val web: String? = null,
    val embedded: String? = null,
    val vr: String? = null
)

private fun String.isSafeActivityUrl(): Boolean =
    startsWith("http://", ignoreCase = true) || startsWith("https://", ignoreCase = true)

/**
 * Gateway activities send labels in `buttons` and URLs in `metadata.button_urls`,
 * while RPC SET_ACTIVITY sends `{ label, url }` objects. Discord allows at most two.
 */
val Activity.resolvedButtons: List<ActivityButton>
    get() {
        val urls = metadata?.button_urls.orEmpty()
        return buttons.orEmpty().mapIndexedNotNull { index, element ->
            val label = when (element) {
                is JsonPrimitive -> element.contentOrNull
                is JsonObject -> (element["label"] as? JsonPrimitive)?.contentOrNull
                else -> null
            }?.takeIf { it.isNotBlank() } ?: return@mapIndexedNotNull null

            val objectUrl = (element as? JsonObject)?.let { (it["url"] as? JsonPrimitive)?.contentOrNull }
            val url = listOfNotNull(objectUrl, urls.getOrNull(index))
                .firstOrNull { it.isSafeActivityUrl() }
                ?: return@mapIndexedNotNull null

            ActivityButton(label, url)
        }.take(2)
    }
