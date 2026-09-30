package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.request
import io.ktor.http.HttpMethod
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.utils.Logging

data class BoardGameEntry(
    val gameId: String,
    val comment: String? = null,
    val tags: List<String> = emptyList()
)

data class BoardWidget(
    val type: String,
    val games: List<BoardGameEntry> = emptyList(),
    val applicationId: String? = null
)

data class BoardGameInfo(
    val name: String,
    val image: String? = null
)

data class BoardStat(val value: String, val label: String)

data class BoardApplication(
    val name: String,
    val icon: String? = null,
    val title: String? = null,
    val image: String? = null,
    val subtitles: List<String> = emptyList(),
    val stats: List<BoardStat> = emptyList()
)

data class ProfileBoard(
    val widgets: List<BoardWidget> = emptyList(),
    val games: Map<String, BoardGameInfo> = emptyMap(),
    val applications: Map<String, BoardApplication> = emptyMap()
) {
    val isEmpty: Boolean get() = widgets.isEmpty()
}class ProfileBoardApi(private val rest: RestClient) {

    private val json = Json { ignoreUnknownKeys = true }

    suspend fun load(userId: String): ProfileBoard? {
        val profile = try {
            val response = rest.httpClient.get("${rest.apiBase}/users/$userId/profile") {
                standardHeaders(rest)
                parameter("with_mutual_guilds", false)
                parameter("with_mutual_friends", false)
            }
            if (!response.status.isSuccess()) return null
            json.parseToJsonElement(response.body<String>()).jsonObject
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Board", "Error fetching profile board: ${e.message}")
            return null
        }

        val widgets = parseWidgets(profile["widgets"])
        if (widgets.isEmpty()) return ProfileBoard()

        val gameIds = widgets.flatMap { it.games }.map { it.gameId }.distinct()
        return ProfileBoard(
            widgets = widgets,
            games = resolveGames(gameIds),
            applications = resolveApplications(userId, widgets)
        )
    }

    private suspend fun resolveGames(ids: List<String>): Map<String, BoardGameInfo> {
        val games = LinkedHashMap<String, BoardGameInfo>()
        ids.chunked(GAME_BATCH).forEach { batch ->
            val response = try {
                rest.httpClient.get("${rest.apiBase}/games") {
                    standardHeaders(rest)
                    batch.forEach { parameter("game_ids", it) }
                }
            } catch (e: Exception) {
                if (e is CancellationException) throw e
                null
            }
            val array = response?.takeIf { it.status.isSuccess() }
                ?.let { json.parseToJsonElement(it.body<String>()) }
                ?.let { element ->
                    when (element) {
                        is JsonArray -> element
                        is JsonObject -> element["games"] as? JsonArray
                        else -> null
                    }
                }
            array?.forEach { element ->
                parseGame(element as? JsonObject ?: return@forEach)?.let { (id, info) -> games[id] = info }
            }
        }
        ids.forEach { id ->
            if (id in games) return@forEach
            val response = try {
                rest.httpClient.get("${rest.apiBase}/games/$id") { standardHeaders(rest) }
            } catch (e: Exception) {
                if (e is CancellationException) throw e
                null
            }
            if (response?.status?.isSuccess() != true) return@forEach
            parseGame(json.parseToJsonElement(response.body<String>()) as? JsonObject)
                ?.let { (gameId, info) -> games[gameId] = info }
        }
        return games
    }

    private fun parseWidgets(element: JsonElement?): List<BoardWidget> {
        val array = element as? JsonArray ?: return emptyList()
        val widgets = mutableListOf<BoardWidget>()
        array.forEach { wrapper ->
            val data = (wrapper as? JsonObject)?.get("data") as? JsonObject ?: return@forEach
            val type = data.text("type") ?: return@forEach
            if (type !in WIDGET_TYPES) return@forEach
            val games = (data["games"] as? JsonArray)?.mapNotNull { game ->
                val obj = game as? JsonObject ?: return@mapNotNull null
                val id = obj.text("game_id") ?: return@mapNotNull null
                val tags = (obj["tags"] as? JsonArray)?.mapNotNull { it.textOrNull() } ?: emptyList()
                BoardGameEntry(id, obj.text("comment"), tags)
            }.orEmpty()
            val applicationId = data.text("application_id")
            if (games.isNotEmpty() || applicationId != null) {
                widgets += BoardWidget(type, games, applicationId)
            }
        }
        return widgets
    }

    private fun parseGame(game: JsonObject?): Pair<String, BoardGameInfo>? {
        val id = game?.text("id") ?: return null
        val supplemental = game["supplemental_game_data"] as? JsonObject
        val name = game.text("name")?.takeIf { it != id }
            ?: supplemental?.text("name")?.takeIf { it != id }
            ?: UNKNOWN_GAME_NAME
        val artwork = (supplemental?.get("artwork_urls") as? JsonArray)?.firstOrNull()?.textOrNull()
        val cover = supplemental?.text("cover_image_url")
        val icon = supplemental?.text("icon_hash") ?: game.text("icon_hash")
        val mediaCover = (game["media"] as? JsonObject)?.get("cover") as? JsonObject
        val mediaCoverUrl = when (mediaCover?.text("type")) {
            "url" -> mediaCover.text("value")
            "hash" -> mediaCover.text("value")?.let { "$APP_ICONS/$id/$it.png?size=256" }
            else -> null
        }
        val image = when {
            mediaCoverUrl?.startsWith("https://") == true -> mediaCoverUrl
            artwork?.startsWith("https://") == true -> artwork
            cover?.startsWith("https://") == true -> cover
            icon != null -> "$APP_ICONS/$id/$icon.png?size=256"
            else -> null
        }
        return id to BoardGameInfo(name, image)
    }

    private suspend fun resolveApplications(
        userId: String,
        widgets: List<BoardWidget>
    ): Map<String, BoardApplication> {
        val ids = widgets.mapNotNull { it.applicationId }.distinct()
        if (ids.isEmpty()) return emptyMap()

        val identities = requestJson("${rest.apiBase}/users/$userId/application-identities") {
            standardHeaders(rest)
            parameter("with_profiles", true)
        }.list("identities").associateBy { it.text("application_id") }

        val publicQuery = ids.joinToString("&") { "application_ids=$it" }
        val publicApps = requestJson("${rest.apiBase}/applications/public?$publicQuery") {
            standardHeaders(rest)
        }.list("applications").associateBy { it.text("id") }

        val result = LinkedHashMap<String, BoardApplication>()
        ids.forEach { id ->
            val configs = requestJson("${rest.apiBase}/applications/$id/widget-configs") {
                standardHeaders(rest)
            }.list("configs")
            result[id] = parseApplication(id, publicApps[id], identities[id], selectConfig(configs))
        }
        return result
    }

    private suspend fun requestJson(url: String, block: io.ktor.client.request.HttpRequestBuilder.() -> Unit): JsonObject? {
        return try {
            val response = rest.httpClient.request(url) {
                method = HttpMethod.Get
                block()
            }
            if (response.status.isSuccess()) {
                json.parseToJsonElement(response.body<String>()) as? JsonObject
            } else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            null
        }
    }

    private fun JsonObject?.list(key: String): List<JsonObject> =
        (this?.get(key) as? JsonArray)?.filterIsInstance<JsonObject>() ?: emptyList()

    private fun parseArray(element: JsonElement?): List<JsonObject> {
        return when (element) {
            is JsonArray -> element.filterIsInstance<JsonObject>()
            is JsonObject -> (element["games"] as? JsonArray)?.filterIsInstance<JsonObject>()
                ?: (element["applications"] as? JsonArray)?.filterIsInstance<JsonObject>()
                ?: (element["configs"] as? JsonArray)?.filterIsInstance<JsonObject>()
                ?: (element["identities"] as? JsonArray)?.filterIsInstance<JsonObject>()
                ?: emptyList()

            else -> emptyList()
        }
    }    private fun selectConfig(configs: List<JsonObject>): JsonObject? {
        var first: JsonObject? = null
        var withTop: JsonObject? = null
        configs.forEach { config ->
            if (first == null) first = config
            val hasTop = (config["surfaces"] as? JsonObject)?.get("widget_top") != null
            if (hasTop) {
                if (withTop == null) withTop = config
                if (config.text("status") == "published") return config
            }
        }
        return withTop ?: first
    }

    private fun parseApplication(
        id: String,
        application: JsonObject?,
        identity: JsonObject?,
        config: JsonObject?
    ): BoardApplication {
        val app = application ?: (config?.get("application") as? JsonObject)
        val name = config?.text("display_name") ?: app?.text("name") ?: "Game Stats"
        val iconHash = app?.text("icon") ?: app?.text("icon_hash")
        val icon = iconHash?.let { "$APP_ICONS/$id/$it.png?size=64" }
        val profile = identity?.get("profile") as? JsonObject
            ?: (identity?.get("profiles") as? JsonArray)?.firstOrNull() as? JsonObject
        val values = profileValues(profile)
        val assets = config?.get("resolved_assets") as? JsonArray
        val top = (config?.get("surfaces") as? JsonObject)?.get("widget_top") as? JsonObject
        val bottom = (config?.get("surfaces") as? JsonObject)?.get("widget_bottom") as? JsonObject
        val topComponents = top?.get("components") as? JsonObject

        val title = componentText(topComponents, "title", "text", values, assets, id)
            ?: profile?.text("username")
        val image = componentImage(topComponents, "hero_image", values, assets, id)
            ?: componentImage(topComponents, "contained_image", values, assets, id)
            ?: mediaUrl(values["featured_played_character_image"])
        val subtitles = (1..3).mapNotNull { componentText(topComponents, "subtitle_$it", "text", values, assets, id) }

        val stats = mutableListOf<BoardStat>()
        val bottomComponents = bottom?.get("components") as? JsonObject
        for (index in 1..20) {
            val component = bottomComponents?.get("stat_$index") as? JsonObject ?: continue
            val fields = component["fields"] as? JsonObject
            val value = resolve(fields?.get("value") as? JsonObject, values, assets, id)?.text
            val label = resolve(fields?.get("label") as? JsonObject, values, assets, id)?.text
            if (value != null && label != null) stats += BoardStat(value, label)
        }
        return BoardApplication(name, icon, title, image, subtitles, stats)
    }

    private fun componentText(
        components: JsonObject?,
        name: String,
        field: String,
        values: Map<String, JsonElement>,
        assets: JsonArray?,
        applicationId: String
    ): String? = resolve(
        (components?.get(name) as? JsonObject)?.let { (it["fields"] as? JsonObject)?.get(field) } as? JsonObject,
        values, assets, applicationId
    )?.text

    private fun componentImage(
        components: JsonObject?,
        name: String,
        values: Map<String, JsonElement>,
        assets: JsonArray?,
        applicationId: String
    ): String? = resolve(
        (components?.get(name) as? JsonObject)?.let { (it["fields"] as? JsonObject)?.get("image") } as? JsonObject,
        values, assets, applicationId
    )?.image

    private data class Resolved(val text: String? = null, val image: String? = null)

    private fun resolve(
        field: JsonObject?,
        values: Map<String, JsonElement>,
        assets: JsonArray?,
        applicationId: String,
        depth: Int = 0
    ): Resolved? {
        if (field == null || depth > 3) return null
        val key = field.text("value")
        val resolved = when (field.text("value_type")) {
            "custom_string" -> key?.let { Resolved(text = it) }
            "data" -> when (val data = key?.let { values[it] }) {
                is JsonObject -> mediaUrl(data)?.let { Resolved(image = it) }
                is JsonPrimitive -> if (data.isString) Resolved(text = data.content)
                else Resolved(text = data.content)

                else -> null
            }
            "application_asset" -> findAsset(assets, key)?.text("asset_id")?.let { assetId ->
                Resolved(image = "https://cdn.discordapp.com/app-assets/$applicationId/$assetId.webp?size=512")
            }
            else -> null
        }
        return resolved ?: resolve(field["fallback"] as? JsonObject, values, assets, applicationId, depth + 1)
    }

    private fun findAsset(assets: JsonArray?, key: String?): JsonObject? {
        if (assets == null || key == null) return null
        return assets.filterIsInstance<JsonObject>().firstOrNull { it.text("key") == key }
    }

    private fun profileValues(profile: JsonObject?): Map<String, JsonElement> {
        val values = mutableMapOf<String, JsonElement>()
        if (profile == null) return values
        profile.text("username")?.let { values["username"] = JsonPrimitive(it) }
        val data = profile["data"] as? JsonObject
        (data?.get("primary") as? JsonObject)?.forEach { (key, value) ->
            if (value is JsonPrimitive || value is JsonObject) values[key] = value
        }
        (data?.get("dynamic") as? JsonArray)?.forEach { field ->
            val obj = field as? JsonObject ?: return@forEach
            val key = obj.text("name") ?: return@forEach
            val value = obj["value"] ?: return@forEach
            if (value is JsonPrimitive || value is JsonObject) values[key] = value
        }
        return values
    }

    private fun mediaUrl(value: JsonElement?): String? {
        val media = value as? JsonObject ?: return null
        media.text("proxy_url")?.let { if (it.startsWith("https://")) return it }
        return media.text("url")?.takeIf { it.startsWith("https://") }
    }

    private companion object {
        const val UNKNOWN_GAME_NAME = "Unknown game"
        const val GAME_BATCH = 25
        const val APP_ICONS = "https://cdn.discordapp.com/app-icons"
        val WIDGET_TYPES = setOf(
            "favorite_games",
            "played_games",
            "current_games",
            "want_to_play_games",
            "application"
        )
    }
}

private fun JsonObject.text(key: String): String? {
    val element = this[key] ?: return null
    if (element is JsonNull) return null
    val primitive = element as? JsonPrimitive ?: return null
    if (!primitive.isString) return null
    return primitive.content.takeIf { it.isNotBlank() && it != "null" }
}

private fun JsonElement.textOrNull(): String? =
    (this as? JsonPrimitive)?.takeIf { it.isString }?.content?.takeIf { it.isNotBlank() && it != "null" }
