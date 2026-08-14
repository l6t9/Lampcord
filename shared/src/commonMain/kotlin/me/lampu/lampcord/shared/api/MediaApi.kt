package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.isSuccess
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.StickerPack
import me.lampu.lampcord.shared.model.StickerStoreDirectory
import me.lampu.lampcord.shared.model.TrendingGifCategoriesResponse
import me.lampu.lampcord.shared.utils.Logging

/**
 * Sticker packs and GIF endpoints.
 */
class MediaApi(private val rest: RestClient) {

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
                parameter("provider", "klipy")
                parameter("locale", locale)
                parameter("media_format", "mp4")
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Gifs", "Error fetching trending GIF categories: ${e.message}")
            null
        }
    }

    suspend fun getTrendingGifCategory(category: String, locale: String = "en-US", limit: Int = 50): List<Gif> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/gifs/trending-gifs") {
                standardHeaders(rest)
                parameter("q", category)
                parameter("provider", "klipy")
                parameter("locale", locale)
                parameter("media_format", "mp4")
                parameter("limit", limit)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Gifs", "Error fetching trending GIF category: ${e.message}")
            emptyList()
        }
    }

    suspend fun searchGifs(query: String, locale: String = "en-US", limit: Int = 50): List<Gif> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/gifs/search") {
                standardHeaders(rest)
                parameter("q", query)
                parameter("provider", "klipy")
                parameter("locale", locale)
                parameter("media_format", "mp4")
                parameter("limit", limit)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Gifs", "Error searching GIFs: ${e.message}")
            emptyList()
        }
    }
}
