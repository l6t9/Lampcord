package me.lampu.lampcord.shared.state

import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.model.ClientProfileMapping
import me.lampu.lampcord.shared.model.CustomProfile

class ClientProfileStore(
    private val httpClient: HttpClient,
    private val json: Json,
    private val scope: CoroutineScope
) {
    private val _customProfiles = MutableStateFlow<ClientProfileMapping>(ClientProfileMapping())
    val customProfiles = _customProfiles.asStateFlow()

    private val _localOverrides = MutableStateFlow<Map<String, CustomProfile>>(emptyMap())
    val localOverrides = _localOverrides.asStateFlow()
    
    init {
        loadLocalOverrides()
        fetchCustomProfiles()
    }

    private fun loadLocalOverrides() {
        val saved = me.lampu.lampcord.shared.settings.Settings.shared.localProfileOverrides
        if (saved.isNotBlank()) {
            try {
                _localOverrides.value = json.decodeFromString(saved)
            } catch (e: Exception) { }
        }
    }

    fun setLocalOverride(userId: String, profile: CustomProfile?) {
        val newOverrides = _localOverrides.value.toMutableMap()
        if (profile == null) {
            newOverrides.remove(userId)
        } else {
            newOverrides[userId] = profile
        }
        _localOverrides.value = newOverrides
        me.lampu.lampcord.shared.settings.Settings.shared.localProfileOverrides = json.encodeToString(newOverrides)
    }

    fun fetchCustomProfiles() {
        scope.launch {
            try {
                // Fetch UserBG
                val userBgRaw = httpClient.get("https://usrbg.is-hardly.online/users").bodyAsText()
                val userBgResponse = try {
                    json.decodeFromString<UserBgResponse>(userBgRaw)
                } catch (e: Exception) {
                    println("ClientProfileStore: Failed to decode UserBG: ${e.message}")
                    null
                }

                // Fetch UserPFP
                val userPfpRaw = httpClient.get("https://raw.githubusercontent.com/UserPFP/UserPFP/main/source/data.json").bodyAsText()
                val userPfpResponse = try {
                    json.decodeFromString<UserPfpResponse>(userPfpRaw)
                } catch (e: Exception) {
                    println("ClientProfileStore: Failed to decode UserPFP: ${e.message}")
                    null
                }
                
                val combined = mutableMapOf<String, CustomProfile>()
                
                userBgResponse?.users?.keys?.forEach { userId ->
                    combined[userId] = CustomProfile(
                        user_id = userId,
                        banner = "https://usrbg.is-hardly.online/usrbg/v2/$userId"
                    )
                }
                
                userPfpResponse?.avatars?.forEach { (userId, avatarUrl) ->
                    val existing = combined[userId]
                    combined[userId] = if (existing != null) {
                        existing.copy(avatar = avatarUrl)
                    } else {
                        CustomProfile(user_id = userId, avatar = avatarUrl)
                    }
                }
                
                if (combined.isNotEmpty()) {
                    _customProfiles.value = ClientProfileMapping(combined)
                }
            } catch (e: Exception) {
                // me.lampu.lampcord.shared.utils.Logging.e("ClientProfile", "Error fetching databases: ${e.message}")
            }
        }
    }

    @kotlinx.serialization.Serializable
    private data class UserBgResponse(
        val users: Map<String, String>
    )

    @kotlinx.serialization.Serializable
    private data class UserPfpResponse(
        val avatars: Map<String, String>
    )

    fun getCustomProfile(userId: String): CustomProfile? {
        val db = _customProfiles.value.users[userId]
        val local = _localOverrides.value[userId]
        
        return if (local != null) {
            local.copy(
                banner = if (local.banner != null) local.banner.takeIf { it.isNotEmpty() } else db?.banner,
                avatar = if (local.avatar != null) local.avatar.takeIf { it.isNotEmpty() } else db?.avatar,
                theme_colors = local.theme_colors ?: db?.theme_colors,
                accent_color = local.accent_color ?: db?.accent_color
            )
        } else {
            db
        }
    }
}

