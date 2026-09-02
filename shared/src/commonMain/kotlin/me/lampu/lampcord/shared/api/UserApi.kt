package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.patch
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.model.ConnectedAccount
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GuildFolder
import me.lampu.lampcord.shared.model.Relationship
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.model.decodeFavoriteGifs
import me.lampu.lampcord.shared.utils.Logging

/**
 * Profiles, relationships, status, user settings, connections and devices.
 */
class UserApi(private val rest: RestClient) {

    suspend fun getUserSettingsProto(type: Int = 2): String? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/settings-proto/$type") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) {
                response.body<SettingsProtoResponse>().settings
            } else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Settings", "Error fetching user settings proto: ${e.message}")
            null
        }
    }

    suspend fun getFavoriteGifs(): List<Gif> {
        return getUserSettingsProto(type = 2)?.let(::decodeFavoriteGifs).orEmpty()
    }

    suspend fun getUserProfile(userId: String, guildId: String? = null): UserProfile? {
        return try {
            val url = if (guildId != null) {
                "${rest.apiBase}/users/$userId/profile?guild_id=$guildId&with_mutual_guilds=true&with_mutual_friends_count=true"
            } else {
                "${rest.apiBase}/users/$userId/profile?with_mutual_guilds=true&with_mutual_friends_count=true"
            }
            val response = rest.httpClient.get(url) {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                val errorBody = response.bodyAsText()
                Logging.e("Profile", "Error fetching user profile: ${response.status}, body: $errorBody")
                null
            }
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Profile", "Error fetching user profile: ${e.message}")
            null
        }
    }

    suspend fun patchUser(partial: User.Partial): User? {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("User", "Error patching user: ${e.message}")
            null
        }
    }

    suspend fun patchUserProfile(partial: me.lampu.lampcord.shared.model.UserProfileMetadata.Partial): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/profile") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("User", "Error patching user profile: ${e.message}")
            false
        }
    }

    suspend fun getRelationships(): List<Relationship> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/relationships") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Relationship", "Error fetching relationships: ${e.message}")
            emptyList()
        }
    }

    suspend fun getMutualFriends(userId: String): List<User> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/$userId/relationships") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) {
                response.body<List<me.lampu.lampcord.shared.model.MutualFriendResponse>>().map { it.user }
            } else emptyList()
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Relationship", "Error fetching mutual friends for $userId: ${e.message}")
            emptyList()
        }
    }

    suspend fun addRelationship(userId: String, type: Int): Boolean {
        return try {
            val response = rest.httpClient.put("${rest.apiBase}/users/@me/relationships/$userId") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("type", type) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Relationship", "Error adding relationship: ${e.message}")
            false
        }
    }

    suspend fun addRelationshipByUsername(username: String, discriminator: String? = null): Boolean {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/users/@me/relationships") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("username", username)
                    if (discriminator != null) put("discriminator", discriminator)
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Relationship", "Error sending friend request by username: ${e.message}")
            false
        }
    }

    suspend fun deleteRelationship(userId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/users/@me/relationships/$userId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Relationship", "Error deleting relationship: ${e.message}")
            false
        }
    }

    suspend fun updateRelationship(userId: String, nickname: String?): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/relationships/$userId") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    if (nickname != null) put("nickname", nickname)
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Relationship", "Error updating relationship: ${e.message}")
            false
        }
    }

    suspend fun updateStatus(status: String): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/settings") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("status", status) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Status", "Error updating status: ${e.message}")
            false
        }
    }

    suspend fun updateCustomStatus(text: String?, emojiName: String? = null, emojiId: String? = null): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/settings") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("custom_status", buildJsonObject {
                        put("text", text)
                        put("emoji_name", emojiName)
                        put("emoji_id", emojiId)
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Status", "Error updating custom status: ${e.message}")
            false
        }
    }

    suspend fun getUserSettings(): UserSettings? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/settings") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Settings", "Error fetching user settings: ${e.message}")
            null
        }
    }

    suspend fun getGuildFolders(): List<GuildFolder> {
        return getUserSettings()?.guild_folders ?: emptyList()
    }

    suspend fun updateUserSettings(partial: UserSettings.Partial): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/settings") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Settings", "Error updating user settings: ${e.message}")
            false
        }
    }

    suspend fun updateUserSettingsProto(base64Payload: String, type: Int = 1): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/settings-proto/$type") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(mapOf("settings" to base64Payload))
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Settings", "Error updating user settings proto: ${e.message}")
            false
        }
    }

    suspend fun getConnections(): List<ConnectedAccount> {
        return try {
            rest.httpClient.get("${rest.apiBase}/users/@me/connections") {
                standardHeaders(rest)
            }.body()
        } catch (e: Exception) {
            Logging.e("Auth", "Error fetching connections: ${e.message}")
            emptyList()
        }
    }

    suspend fun getDevices(): List<DiscordDevice> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/devices") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Devices", "Error fetching devices: ${e.message}")
            emptyList()
        }
    }
}

@kotlinx.serialization.Serializable
private data class SettingsProtoResponse(
    val settings: String = ""
)

@kotlinx.serialization.Serializable
data class DiscordDevice(
    val id: String,
    val model: String? = null,
    val os: String? = null,
    val browser: String? = null,
    val client_version: String? = null,
    val last_used: String? = null,
    val ip_address: String? = null,
    val location: String? = null
)
