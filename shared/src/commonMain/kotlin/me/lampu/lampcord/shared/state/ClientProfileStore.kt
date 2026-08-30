package me.lampu.lampcord.shared.state

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
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

    init {
        fetchCustomProfiles()
    }

    fun fetchCustomProfiles() {
        scope.launch {
            try {
                // Fetch custom profile mapping from a remote source
                // For now using a placeholder URL similar to badges
                val response: ClientProfileMapping = httpClient.get("https://raw.githubusercontent.com/lampcord/badges/main/profiles.json").body()
                _customProfiles.value = response
            } catch (e: Exception) {
                // Fallback or ignore
            }
        }
    }

    fun getCustomProfile(userId: String): CustomProfile? {
        return _customProfiles.value.users[userId]
    }
}
