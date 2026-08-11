package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.ui.geometry.Offset
import kotlinx.coroutines.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.UserProfile

class ProfileStore(
    private val discordClient: DiscordClient,
    private val scope: CoroutineScope
) {
    var selectedProfile by mutableStateOf<UserProfile?>(null)
    var sidebarProfile by mutableStateOf<UserProfile?>(null)
    var isSidebarProfileLoading by mutableStateOf(false)
    var isProfileExpanded by mutableStateOf(false)
    var isProfileLoading by mutableStateOf(false)
    var profilePosition by mutableStateOf<Offset?>(null)

    fun showProfile(userId: String, guildId: String? = null, position: Offset? = null) {
        selectedProfile = null
        isProfileExpanded = false
        profilePosition = position
        isProfileLoading = true
        scope.launch {
            selectedProfile = discordClient.getUserProfile(userId, guildId)?.copy(guild_id = guildId)
            isProfileLoading = false
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
