package com.example.lampcord.shared.api

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.client.request.forms.*
import io.ktor.http.content.*
import com.example.lampcord.shared.model.*
import com.example.lampcord.shared.utils.getPlatformName
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.plugins.cookies.*
import io.ktor.serialization.kotlinx.json.*
import io.ktor.util.*
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlinx.serialization.json.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString

@Serializable
data class MessageRequest(
    val content: String,
    val message_reference: MessageReference? = null,
    val tts: Boolean = false,
    val nonce: String? = null
)

@Serializable
data class SuperProperties(
    val os: String,
    val browser: String,
    val release_channel: String = "stable",
    val client_version: String = "0.0.398",
    val os_version: String = "",
    val os_arch: String = "x64",
    val app_arch: String = "x64",
    val system_locale: String = "en-US",
    val browser_user_agent: String,
    val browser_version: String = "",
    val client_build_number: Int = 363,
    val native_build_number: Int? = null,
    val client_event_source: String? = null
)

class DiscordClient(
    private val httpClient: HttpClient,
    private val json: Json
) {
    private var token: String? = null
    private val apiVersion = 9
    private val apiBase = "https://discord.com/api/v$apiVersion"

    fun setToken(token: String) {
        this.token = token
    }

    @OptIn(ExperimentalEncodingApi::class)
    private fun getSuperProperties(): String {
        val platform = getPlatformName()
        val osName = when(platform) {
            "macos" -> "Mac OS X"
            "windows" -> "Windows"
            "linux" -> "Linux"
            "android" -> "Android"
            "ios" -> "iOS"
            else -> platform
        }
        
        val properties = SuperProperties(
            os = osName,
            browser = "Discord Client",
            browser_user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.1 Electron/28.2.1 Safari/537.36",
            browser_version = "28.2.1",
            client_build_number = 363
        )
        val jsonString = json.encodeToString(properties)
        return Base64.encode(jsonString.encodeToByteArray())
    }

    suspend fun getFingerprint(): String? {
        return try {
            val response: FingerprintResponse = httpClient.get("$apiBase/experiments").body()
            response.fingerprint
        } catch (e: Exception) {
            println("Error fetching fingerprint: ${e.message}")
            null
        }
    }

    suspend fun login(request: LoginRequest, fingerprint: String): LoginResponse? {
        return try {
            val response = httpClient.post("$apiBase/auth/login") {
                header("X-Fingerprint", fingerprint)
                header("X-Super-Properties", getSuperProperties())
                header("Origin", "https://discord.com")
                header("Referer", "https://discord.com/login")
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            
            val responseBody = response.bodyAsText()
            if (response.status.isSuccess() || response.status.value == 400 || response.status.value == 401) {
                json.decodeFromString<LoginResponse>(responseBody)
            } else {
                println("Login failed with status: ${response.status}, body: $responseBody")
                null
            }
        } catch (e: Exception) {
            println("Error logging in: ${e.message}")
            null
        }
    }

    suspend fun loginMFA(request: MFALoginRequest, fingerprint: String, type: String): LoginResponse? {
        return try {
            val response = httpClient.post("$apiBase/auth/mfa/$type") {
                header("X-Fingerprint", fingerprint)
                header("X-Super-Properties", getSuperProperties())
                header("Origin", "https://discord.com")
                header("Referer", "https://discord.com/login")
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                response.body<LoginResponse>()
            } else {
                val errorBody = response.bodyAsText()
                println("MFA failed with status: ${response.status}, body: $errorBody")
                null
            }
        } catch (e: Exception) {
            println("Error verifying MFA: ${e.message}")
            null
        }
    }

    suspend fun sendMessage(
        channelId: String, 
        content: String, 
        replyTo: String? = null,
        files: List<Pair<String, ByteArray>> = emptyList(),
        nonce: String? = null
    ): Boolean {
        if (token == null) return false
        
        return try {
            if (files.isEmpty()) {
                val response = httpClient.post("$apiBase/channels/$channelId/messages") {
                    header(HttpHeaders.Authorization, token!!)
                    header("X-Super-Properties", getSuperProperties())
                    contentType(ContentType.Application.Json)
                    
                    val request = MessageRequest(
                        content = content,
                        message_reference = replyTo?.let { MessageReference(message_id = it) },
                        nonce = nonce
                    )
                    setBody(request)
                }
                response.status.isSuccess()
            } else {
                val response = httpClient.post("$apiBase/channels/$channelId/messages") {
                    header(HttpHeaders.Authorization, token!!)
                    header("X-Super-Properties", getSuperProperties())
                    setBody(MultiPartFormDataContent(
                        formData {
                            append("payload_json", json.encodeToString(MessageRequest(
                                content = content,
                                message_reference = replyTo?.let { MessageReference(message_id = it) },
                                nonce = nonce
                            )))
                            files.forEachIndexed { index, (name, bytes) ->
                                append("file$index", bytes, Headers.build {
                                    append(HttpHeaders.ContentDisposition, "form-data; name=\"file$index\"; filename=\"$name\"")
                                })
                            }
                        }
                    ))
                }
                response.status.isSuccess()
            }
        } catch (e: Exception) {
            println("Error sending message: ${e.message}")
            false
        }
    }

    suspend fun getGuildChannels(guildId: String): List<Channel> {
        if (token == null) return emptyList()
        return try {
            httpClient.get("$apiBase/guilds/$guildId/channels") {
                header(HttpHeaders.Authorization, token!!)
            }.body()
        } catch (e: Exception) {
            println("Error fetching channels: ${e.message}")
            emptyList()
        }
    }

    suspend fun getChannelMessages(channelId: String, limit: Int = 50, before: String? = null): List<Message> {
        if (token == null) return emptyList()
        return try {
            httpClient.get("$apiBase/channels/$channelId/messages") {
                header(HttpHeaders.Authorization, token!!)
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }.body()
        } catch (e: Exception) {
            println("Error fetching messages: ${e.message}")
            emptyList()
        }
    }

    suspend fun getActiveThreads(channelId: String): ThreadListResponse? {
        if (token == null) return null
        return try {
            httpClient.get("$apiBase/channels/$channelId/threads/active") {
                header(HttpHeaders.Authorization, token!!)
            }.body()
        } catch (e: Exception) {
            println("Error fetching active threads: ${e.message}")
            null
        }
    }

    suspend fun getArchivedPublicThreads(channelId: String, limit: Int = 100, before: String? = null): ThreadListResponse? {
        if (token == null) return null
        return try {
            httpClient.get("$apiBase/channels/$channelId/threads/archived/public") {
                header(HttpHeaders.Authorization, token!!)
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }.body()
        } catch (e: Exception) {
            println("Error fetching archived threads: ${e.message}")
            null
        }
    }

    suspend fun getGuildMembers(guildId: String, limit: Int = 100): List<Member> {
        if (token == null) return emptyList()
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/members") {
                header(HttpHeaders.Authorization, token!!)
                parameter("limit", limit)
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                val errorBody = response.bodyAsText()
                println("Error fetching members: $errorBody")
                emptyList()
            }
        } catch (e: Exception) {
            println("Error fetching members: ${e.message}")
            emptyList()
        }
    }

    suspend fun getUserProfile(userId: String, guildId: String? = null): UserProfile? {
        if (token == null) return null
        return try {
            val url = if (guildId != null) {
                "$apiBase/users/$userId/profile?guild_id=$guildId"
            } else {
                "$apiBase/users/$userId/profile"
            }
            httpClient.get(url) {
                header(HttpHeaders.Authorization, token!!)
            }.body()
        } catch (e: Exception) {
            println("Error fetching user profile: ${e.message}")
            null
        }
    }

    suspend fun editMessage(channelId: String, messageId: String, content: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.patch("$apiBase/channels/$channelId/messages/$messageId") {
                header(HttpHeaders.Authorization, token!!)
                contentType(ContentType.Application.Json)
                setBody(MessageRequest(content))
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error editing message: ${e.message}")
            false
        }
    }

    suspend fun deleteMessage(channelId: String, messageId: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.delete("$apiBase/channels/$channelId/messages/$messageId") {
                header(HttpHeaders.Authorization, token!!)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error deleting message: ${e.message}")
            false
        }
    }

    suspend fun getRelationships(): List<Relationship> {
        if (token == null) return emptyList()
        return try {
            httpClient.get("$apiBase/users/@me/relationships") {
                header(HttpHeaders.Authorization, token!!)
            }.body()
        } catch (e: Exception) {
            println("Error fetching relationships: ${e.message}")
            emptyList()
        }
    }

    suspend fun addReaction(channelId: String, messageId: String, emoji: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.put("$apiBase/channels/$channelId/messages/$messageId/reactions/$emoji/@me") {
                header(HttpHeaders.Authorization, token!!)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error adding reaction: ${e.message}")
            false
        }
    }

    suspend fun removeReaction(channelId: String, messageId: String, emoji: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.delete("$apiBase/channels/$channelId/messages/$messageId/reactions/$emoji/@me") {
                header(HttpHeaders.Authorization, token!!)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error removing reaction: ${e.message}")
            false
        }
    }

    suspend fun updateStatus(status: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                header(HttpHeaders.Authorization, token!!)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("status", status) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error updating status: ${e.message}")
            false
        }
    }

    suspend fun updateCustomStatus(text: String?, emojiName: String? = null, emojiId: String? = null): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                header(HttpHeaders.Authorization, token!!)
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
            println("Error updating custom status: ${e.message}")
            false
        }
    }

    suspend fun ackMessage(channelId: String, messageId: String): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.post("$apiBase/channels/$channelId/messages/$messageId/ack") {
                header(HttpHeaders.Authorization, token!!)
                header("X-Super-Properties", getSuperProperties())
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("token", JsonNull) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error acking message: ${e.message}")
            false
        }
    }

    suspend fun updateUserSettings(settings: UserSettings): Boolean {
        if (token == null) return false
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                header(HttpHeaders.Authorization, token!!)
                contentType(ContentType.Application.Json)
                setBody(settings)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error updating user settings: ${e.message}")
            false
        }
    }
}

fun createHttpClient() = HttpClient {
    install(HttpCookies)
    install(ContentNegotiation) {
        json(Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
        })
    }
    install(WebSockets)
}
