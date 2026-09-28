package me.lampu.lampcord.shared.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive

/**
 * An entry of `user_profile.collectibles`, which is a list rather than an object.
 *
 * The effect and frame a profile has equipped are often only discoverable here: some responses
 * omit `profile_effect`/`profile_frame` and carry the sku in this list instead, distinguished
 * by [type] (1 = effect, 3 = frame).
 */
@Serializable
data class CollectibleRef(
    val type: Int? = null,
    @SerialName("sku_id") val sku_id: String? = null,
)

@Serializable
data class ProfileEffectLayer(
    val source: String,
    val start: Long = 0,
    val duration: Long = 0,
    val loop: Boolean = false,
    val loopDelay: Long = 0,
    val x: Long = 0,
    val y: Long = 0,
    val width: Long = EFFECT_CANVAS_WIDTH,
    val height: Long = 880,
    val zIndex: Long = 0,
)

@Serializable
data class ProfileEffectProduct(
    val sku: String,
    val source: String? = null,
    val layers: List<ProfileEffectLayer> = emptyList(),
)

@Serializable
data class ProfileFrameLayer(
    val id: String,
    val front: Boolean = false,
    val anchor: Int = ANCHOR_TOP,
    val rail: Boolean = false,
)

@Serializable
data class ProfileFrameMetrics(
    val innerWidth: Long = 1200,
    val overflowTop: Long = 0,
    val overflowBottom: Long = 0,
    val overflowHorizontal: Long = 0,
)

@Serializable
data class ProfileFrameProduct(
    val sku: String,
    val metrics: ProfileFrameMetrics = ProfileFrameMetrics(),
    val layers: List<ProfileFrameLayer> = emptyList(),
)

const val EFFECT_CANVAS_WIDTH = 450L
const val COLLECTIBLE_TYPE_EFFECT = 1
const val COLLECTIBLE_TYPE_FRAME = 3
const val ANCHOR_TOP = 0
const val ANCHOR_BOTTOM = 1
const val ANCHOR_CENTER = 2

private const val CDN = "https://cdn.discordapp.com"

/**
 * Reads profile effects and frames out of Discord's collectibles payloads.
 *
 * A profile's equipped effect and frame are looked up first as `profile_effect`/`profile_frame`
 * on the profile metadata, then by scanning `collectibles` for a matching type, because either
 * can be absent. A product response is a list of items, and bundles can place a different
 * collectible before the one that was asked for, so items are matched on their own type rather
 * than taking the first.
 */
object ProfileCollectibles {

    fun effectSku(global: UserProfileMetadata?, guild: UserProfileMetadata?): String? =
        sku(guild, COLLECTIBLE_TYPE_EFFECT) ?: sku(global, COLLECTIBLE_TYPE_EFFECT)

    fun frameSku(global: UserProfileMetadata?, guild: UserProfileMetadata?): String? =
        sku(guild, COLLECTIBLE_TYPE_FRAME) ?: sku(global, COLLECTIBLE_TYPE_FRAME)

    private fun sku(profile: UserProfileMetadata?, type: Int): String? {
        if (profile == null) return null
        val direct = when (type) {
            COLLECTIBLE_TYPE_EFFECT -> profile.profile_effect?.sku_id
            else -> profile.profile_frame?.sku_id
        }
        if (!direct.isNullOrBlank()) return direct
        return profile.collectibles
            ?.firstOrNull { it.type == type }
            ?.sku_id
            ?.takeIf { it.isNotBlank() }
    }

    fun parseEffect(sku: String, body: JsonObject): ProfileEffectProduct? {
        for (item in items(body, COLLECTIBLE_TYPE_EFFECT)) {
            val layers = buildList {
                for (raw in item["effects"].asArray()) {
                    val effect = raw as? JsonObject ?: continue
                    val source = imageUrl(effect.str("src")) ?: continue
                    val position = effect["position"] as? JsonObject
                    add(
                        ProfileEffectLayer(
                            source = source,
                            start = effect.long("start") ?: 0,
                            duration = effect.long("duration") ?: 0,
                            loop = effect.bool("loop"),
                            loopDelay = effect.long("loopDelay") ?: effect.long("loop_delay") ?: 0,
                            x = position?.long("x") ?: 0,
                            y = position?.long("y") ?: 0,
                            width = effect.long("width") ?: EFFECT_CANVAS_WIDTH,
                            height = effect.long("height") ?: 880,
                            zIndex = effect.long("zIndex") ?: effect.long("z_index") ?: 0,
                        )
                    )
                }
            }.sortedBy { it.zIndex }

            val source = imageUrl(item.str("reducedMotionSrc") ?: item.str("reduced_motion_src"))
                ?: imageUrl(item.str("staticFrameSrc") ?: item.str("static_frame_src"))
            if (source != null || layers.isNotEmpty()) {
                return ProfileEffectProduct(sku, source ?: layers.first().source, layers)
            }
        }
        return null
    }

    fun parseFrame(sku: String, body: JsonObject): ProfileFrameProduct? {
        for (item in items(body, COLLECTIBLE_TYPE_FRAME)) {
            val layers = buildList {
                val seen = mutableSetOf<String>()
                for (raw in item["layers"].asArray()) {
                    val layer = raw as? JsonObject ?: continue
                    val id = layer.str("id") ?: continue
                    if (!seen.add(id)) continue
                    val anchor = when (layer.enum("anchor")) {
                        "1", "bottom" -> ANCHOR_BOTTOM
                        "2", "center", "middle" -> ANCHOR_CENTER
                        else -> ANCHOR_TOP
                    }
                    val order = layer.enum("order")
                    val type = layer.enum("type") ?: layer.enum("layer_type")
                    add(
                        ProfileFrameLayer(
                            id = id,
                            front = order == "front" || order == "0",
                            anchor = anchor,
                            rail = layer.bool("responsive") || type == "rail" || anchor == ANCHOR_CENTER,
                        )
                    )
                }
            }
            if (layers.isEmpty()) continue
            return ProfileFrameProduct(
                sku = sku,
                metrics = ProfileFrameMetrics(
                    innerWidth = item.long("inner_width") ?: item.long("innerWidth") ?: 1200,
                    overflowTop = item.long("overflow_top") ?: item.long("overflowTop") ?: 0,
                    overflowBottom = item.long("overflow_bottom") ?: item.long("overflowBottom") ?: 0,
                    overflowHorizontal = item.long("overflow_horizontal") ?: item.long("overflowHorizontal") ?: 0,
                ),
                layers = layers,
            )
        }
        return null
    }

    fun frameAssetUrl(sku: String, layerId: String): String =
        "$CDN/media/v1/collectibles-shop/$sku/$layerId/static"

    private fun items(body: JsonObject, type: Int): List<JsonObject> =
        body["items"].asArray().filterIsInstance<JsonObject>().filter { it.int("type") == type }

    private fun JsonElement?.asArray(): List<JsonElement> = (this as? JsonArray)?.toList() ?: emptyList()

    private fun JsonObject.str(vararg keys: String): String? = keys.firstNotNullOfOrNull { key ->
        (this[key] as? JsonPrimitive)?.content?.takeIf { it.isNotBlank() && it != "null" }
    }

    private fun JsonObject?.long(key: String): Long? =
        (this?.get(key) as? JsonPrimitive)?.content?.toDoubleOrNull()?.toLong()

    private fun JsonObject.int(key: String): Int? = long(key)?.toInt()

    private fun JsonObject.bool(key: String): Boolean =
        (this[key] as? JsonPrimitive)?.content.equals("true", ignoreCase = true)

    private fun JsonObject.enum(key: String): String? = str(key)?.lowercase()

    /** Only Discord's own hosts are loaded, so a hostile payload cannot point elsewhere. */
    private fun imageUrl(raw: String?): String? {
        val text = raw?.trim() ?: return null
        val url = when {
            text.startsWith("//") -> "https:$text"
            text.startsWith("/") -> CDN + text
            else -> text
        }
        if (!url.startsWith("https://")) return null
        val host = url.removePrefix("https://").substringBefore('/').substringBefore('?')
        return if (host == "cdn.discordapp.com" || host == "media.discordapp.net") url else null
    }
}
