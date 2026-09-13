package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.ui.geometry.Offset
import kotlinx.coroutines.*
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.UserProfile

class ProfileStore(
    private val userApi: UserApi,
    private val scope: CoroutineScope
) {
    var selectedProfile by mutableStateOf<UserProfile?>(null)
    var sidebarProfile by mutableStateOf<UserProfile?>(null)
    var isSidebarProfileLoading by mutableStateOf(false)
    var isProfileExpanded by mutableStateOf(false)
    var isProfileLoading by mutableStateOf(false)
    var profilePosition by mutableStateOf<Offset?>(null)

    private val collectibleCache = mutableStateMapOf<String, kotlinx.serialization.json.JsonObject>()

    fun getCollectible(skuId: String): kotlinx.serialization.json.JsonObject? {
        val cached = collectibleCache[skuId]
        if (cached != null) return cached

        scope.launch {
            userApi.getCollectibleProduct(skuId)?.let {
                collectibleCache[skuId] = it
            }
        }
        return null
    }

    fun showProfile(userId: String, guildId: String? = null, position: Offset? = null) {
        selectedProfile = null
        isProfileExpanded = false
        profilePosition = position
        isProfileLoading = true
        scope.launch {
            selectedProfile = userApi.getUserProfile(userId, guildId)?.copy(guild_id = guildId)
            isProfileLoading = false
        }
    }

    fun updateMemberRoles(userId: String, roles: List<String>) {
        selectedProfile?.let { p ->
            if (p.user.id == userId) {
                selectedProfile = p.copy(guild_member = p.guild_member?.copy(roles = roles))
            }
        }
        sidebarProfile?.let { p ->
            if (p.user.id == userId) {
                sidebarProfile = p.copy(guild_member = p.guild_member?.copy(roles = roles))
            }
        }
    }

    fun clear() {
        selectedProfile = null
        sidebarProfile = null
        isSidebarProfileLoading = false
        isProfileExpanded = false
        isProfileLoading = false
        profilePosition = null
    }
}
