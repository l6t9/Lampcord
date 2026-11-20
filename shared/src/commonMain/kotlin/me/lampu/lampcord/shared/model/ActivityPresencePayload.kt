package me.lampu.lampcord.shared.model

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put

private val RECEIVE_ONLY_ACTIVITY_FIELDS =
    setOf(
        "id",
        "created_at",
        "session_id",
        "platform",
        "supported_platforms",
        "status_display_type",
        "details_url",
        "state_url",
        "sync_id",
    )

/**
 * Builds the op 3 payload Discord expects. Receive-only fields are stripped and
 * both activity button shapes are normalized to `buttons: string[]` with URLs in
 * `metadata.button_urls`.
 */
fun buildUpdatePresencePayload(
    status: String?,
    activities: List<Activity>?,
    json: Json,
): JsonObject =
    buildJsonObject {
        put("status", status?.let(::JsonPrimitive) ?: JsonNull)
        put("activities", JsonArray(activities.orEmpty().map { it.toOutgoingActivityJson(json) }))
        put("afk", JsonPrimitive(false))
        put("since", JsonPrimitive(0))
    }

private fun Activity.toOutgoingActivityJson(json: Json): JsonObject {
    val outgoing = json.encodeToJsonElement(Activity.serializer(), this)
        .jsonObject
        .toMutableMap()
    RECEIVE_ONLY_ACTIVITY_FIELDS.forEach(outgoing::remove)

    val resolved = resolvedButtons
    if (resolved.isEmpty()) {
        outgoing.remove("buttons")
    } else {
        outgoing["buttons"] = JsonArray(resolved.map { JsonPrimitive(it.label) })
        val metadata = (outgoing["metadata"] as? JsonObject).orEmptyJsonObject().toMutableMap()
        metadata["button_urls"] = JsonArray(resolved.map { JsonPrimitive(it.url) })
        outgoing["metadata"] = JsonObject(metadata)
    }

    return JsonObject(outgoing)
}

private fun JsonObject?.orEmptyJsonObject(): JsonObject = this ?: JsonObject(emptyMap())
