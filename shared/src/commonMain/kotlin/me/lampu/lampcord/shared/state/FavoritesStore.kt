package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.settings.Settings

class FavoritesStore(private val json: Json) {

    val favoriteChannelIds = mutableStateListOf<String>()

    init {
        try {
            val stored = Settings.shared.favoriteChannelsJson
            if (stored.isNotBlank()) {
                favoriteChannelIds.addAll(json.decodeFromString<List<String>>(stored).distinct())
            }
        } catch (_: Exception) {
        }
    }

    fun isFavorite(channelId: String): Boolean = channelId in favoriteChannelIds

    fun toggle(channelId: String) {
        if (isFavorite(channelId)) remove(channelId) else add(channelId)
    }

    fun add(channelId: String) {
        if (favoriteChannelIds.contains(channelId)) return
        favoriteChannelIds.add(channelId)
        persist()
    }

    fun remove(channelId: String) {
        if (favoriteChannelIds.remove(channelId)) persist()
    }

    fun clear() {
        favoriteChannelIds.clear()
        persist()
    }

    private fun persist() {
        Settings.shared.favoriteChannelsJson = json.encodeToString(favoriteChannelIds.toList())
    }
}
