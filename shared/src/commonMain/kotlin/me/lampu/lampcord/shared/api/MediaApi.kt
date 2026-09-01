package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.isSuccess
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonArray
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GifCategory
import me.lampu.lampcord.shared.model.StickerPack
import me.lampu.lampcord.shared.model.StickerStoreDirectory
import me.lampu.lampcord.shared.model.TrendingGifCategoriesResponse
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getPlatformName

/**
 * Sticker packs and GIF endpoints.
 */
class MediaApi(private val rest: RestClient) {

    // The picker is recreated whenever its popup closes. Keep the last good
    // category response on the API instance so a transient refresh failure
    // cannot make the category grid disappear on the next open.
    private var trendingGifCategoriesCache: TrendingGifCategoriesResponse? = null

    private fun requestedGifFormat(): String {
        // Reduced Motion controls playback locally. `png` is not a valid
        // Discord GIF media format and is treated as an MP4 fallback by the
        // endpoint, which leaves category previews blank in image loaders.
        // Desktop uses MP4 because the native FFmpeg player handles it more
        // consistently than animated GIFs; mobile uses the documented GIF
        // representation and selects still previews when needed.
        return when (getPlatformName()) {
            "windows", "macos", "linux" -> "mp4"
            else -> "gif"
        }
    }

    suspend fun getStickerPacks(): StickerStoreDirectory? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/sticker-packs") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Stickers", "Error fetching sticker packs: ${e.message}")
            null
        }
    }

    suspend fun getStickerPack(packId: String): StickerPack? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/sticker-packs/$packId") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Stickers", "Error fetching sticker pack: ${e.message}")
            null
        }
    }

    suspend fun getTrendingGifCategories(locale: String = "en-US"): TrendingGifCategoriesResponse? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/gifs/trending") {
                standardHeaders(rest)
                parameter("locale", locale)
                parameter("media_format", requestedGifFormat())
            }
            if (!response.status.isSuccess()) return trendingGifCategoriesCache

            // Decode each array entry independently. Discord has returned
            // incomplete placeholder GIF objects in the past; one malformed
            // placeholder should not hide the category list itself.
            val body = response.body<JsonObject>()
            val categories = body["categories"]?.jsonArray.orEmpty().mapNotNull { element ->
                runCatching { rest.json.decodeFromJsonElement<GifCategory>(element) }.getOrNull()
            }
            val gifs = body["gifs"]?.jsonArray.orEmpty().mapNotNull { element ->
                runCatching { rest.json.decodeFromJsonElement<Gif>(element) }.getOrNull()
            }
            val parsed = TrendingGifCategoriesResponse(categories = categories, gifs = gifs)
            if (parsed.categories.isNotEmpty()) {
                trendingGifCategoriesCache = parsed
            }
            parsed.categories
                .takeIf { it.isNotEmpty() }
                ?.let { parsed }
                ?: trendingGifCategoriesCache
        } catch (e: Exception) {
            Logging.e("Gifs", "Error fetching trending GIF categories: ${e.message}")
            trendingGifCategoriesCache
        }
    }

    suspend fun getTrendingGifCategory(category: String, locale: String = "en-US", limit: Int = 50): List<Gif> {
        // Category names are search terms. The trending-gifs route does not
        // document a category query parameter and can return an empty list;
        // use the stable search route instead.
        return searchGifs(category, locale, limit)
    }

    suspend fun searchGifs(query: String, locale: String = "en-US", limit: Int = 50): List<Gif> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/gifs/search") {
                standardHeaders(rest)
                parameter("q", query)
                parameter("provider", "klipy")
                parameter("locale", locale)
                parameter("media_format", requestedGifFormat())
                parameter("limit", limit)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Gifs", "Error searching GIFs: ${e.message}")
            emptyList()
        }
    }
}
