package me.lampu.lampcord.shared.state

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.model.BadgeMapping
import me.lampu.lampcord.shared.model.CustomBadge
import me.lampu.lampcord.shared.utils.ResourceLoader

class BadgeStore(
    private val httpClient: HttpClient,
    private val json: Json,
    private val scope: CoroutineScope
) {
    private val _lampcordBadges = MutableStateFlow<BadgeMapping>(BadgeMapping())
    val lampcordBadges = _lampcordBadges.asStateFlow()

    init {
        loadLocalBadges()
        fetchBadges()
    }

    private fun loadLocalBadges() {
        try {
            val text = ResourceLoader.readText("badges/badges.json")
            if (text != null) {
                val localBadges = json.decodeFromString<BadgeMapping>(text)
                _lampcordBadges.value = localBadges
            }
        } catch (e: Exception) {
            println("Error loading local badges: ${e.message}")
        }
    }

    fun fetchBadges() {
        scope.launch {
            try {
                val lampcordResponse: BadgeMapping = httpClient.get("https://raw.githubusercontent.com/lampcord/badges/main/badges.json").body()
                
                _lampcordBadges.value = BadgeMapping(
                    badges = _lampcordBadges.value.badges + lampcordResponse.badges,
                    users = _lampcordBadges.value.users + lampcordResponse.users
                )
            } catch (e: Exception) {
                // Ignore if fetch fails, keep local
            }
        }
    }

    fun getUserBadges(userId: String): List<CustomBadge> {
        val badges = mutableListOf<CustomBadge>()
        
        lampcordBadges.value.users[userId]?.forEach { badgeId ->
            lampcordBadges.value.badges[badgeId]?.let { badges.add(it) }
        }

        return badges
    }
}
