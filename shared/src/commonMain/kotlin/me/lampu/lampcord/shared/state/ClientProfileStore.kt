package me.lampu.lampcord.shared.state

import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.utils.Logging
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.model.ClientProfileMapping
import me.lampu.lampcord.shared.model.CustomProfile
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import org.koin.compose.koinInject

@OptIn(ExperimentalCoroutinesApi::class)
class ClientProfileStore(
    private val httpClient: HttpClient,
    private val json: Json,
    private val scope: CoroutineScope,
    private val settingsStore: SettingsStore
) {
    private val _customProfiles = MutableStateFlow<ClientProfileMapping>(ClientProfileMapping())
    val customProfiles = _customProfiles.asStateFlow()

    private val _localOverrides = MutableStateFlow<Map<String, CustomProfile>>(emptyMap())
    val localOverrides = _localOverrides.asStateFlow()

    init {
        loadLocalOverrides()
        // Each third-party database is only contacted while its toggle is on, and dropped when it is turned off:
        // the UserBG list alone is hundreds of thousands of ids. Fetches are independent, so one failing source
        // doesn't take the other down with it.
        val bannerUserIds = snapshotFlow { settingsStore.userBg }.distinctUntilChanged()
            .mapLatest { enabled -> if (enabled) fetchUserBg() else LongArray(0) }
            .onStart { emit(LongArray(0)) }
        val avatars = snapshotFlow { settingsStore.userPfp }.distinctUntilChanged()
            .mapLatest { enabled -> if (enabled) fetchUserPfp() else emptyMap() }
            .onStart { emit(emptyMap()) }
        scope.launch {
            combine(bannerUserIds, avatars, ::ClientProfileMapping).collect { _customProfiles.value = it }
        }
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

    private suspend fun fetchUserBg(): LongArray = try {
        val raw = httpClient.get("https://usrbg.is-hardly.online/users").bodyAsText()
        json.decodeFromString<UserBgResponse>(raw).users.keys
            .mapNotNull { it.toLongOrNull() }
            .toLongArray()
            .apply { sort() }
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Logging.e("ClientProfileStore", "Failed to load UserBG", e)
        LongArray(0)
    }

    private suspend fun fetchUserPfp(): Map<String, String> = try {
        val raw = httpClient.get("https://raw.githubusercontent.com/UserPFP/UserPFP/main/source/data.json").bodyAsText()
        json.decodeFromString<UserPfpResponse>(raw).avatars
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Logging.e("ClientProfileStore", "Failed to load UserPFP", e)
        emptyMap()
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
        val db = _customProfiles.value[userId]
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

    fun getLocalProfile(userId: String): CustomProfile? = _localOverrides.value[userId]

    fun getRemoteProfile(userId: String): CustomProfile? = _customProfiles.value[userId]

    fun setLocalBanner(userId: String, banner: String?) {
        val existing = _localOverrides.value[userId]
        val updated = (existing ?: CustomProfile(user_id = userId)).copy(
            banner = banner?.takeIf { it.isNotEmpty() }
        )
        setLocalOverride(userId, updated)
    }

    fun setLocalAvatar(userId: String, avatar: String?) {
        val existing = _localOverrides.value[userId]
        val updated = (existing ?: CustomProfile(user_id = userId)).copy(
            avatar = avatar?.takeIf { it.isNotEmpty() }
        )
        setLocalOverride(userId, updated)
    }
}

data class GuildMediaUrls(val icon: String?, val banner: String?)

@Composable
fun rememberGuildMediaUrls(
    guildId: String?,
    guildIconUrl: String?,
    guildBannerUrl: String?,
    clientProfileStore: ClientProfileStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
): GuildMediaUrls {
    val localOverrides by clientProfileStore.localOverrides.collectAsState()
    val customProfiles by clientProfileStore.customProfiles.collectAsState()
    return remember(guildId, guildIconUrl, guildBannerUrl, localOverrides, customProfiles, settingsStore.userBg, settingsStore.userPfp) {
        val local = guildId?.let { localOverrides[it] }
        val remote = guildId?.let { customProfiles[it] }
        GuildMediaUrls(
            icon = local?.avatar?.takeIf { it.isNotEmpty() }
                ?: (if (settingsStore.userPfp) remote?.avatar?.takeIf { it.isNotEmpty() } else null)
                ?: guildIconUrl,
            banner = local?.banner?.takeIf { it.isNotEmpty() }
                ?: (if (settingsStore.userBg) remote?.banner?.takeIf { it.isNotEmpty() } else null)
                ?: guildBannerUrl
        )
    }
}

