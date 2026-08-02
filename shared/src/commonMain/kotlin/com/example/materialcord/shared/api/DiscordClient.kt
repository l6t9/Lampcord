package com.example.materialcord.shared.api

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import com.example.materialcord.shared.model.*
import com.example.materialcord.shared.utils.getPlatformName
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
data class MessageRequest(val content: String)

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

    suspend fun sendMessage(channelId: String, content: String): Boolean {
        if (token == null) return false
        
        return try {
            val response = httpClient.post("$apiBase/channels/$channelId/messages") {
                header(HttpHeaders.Authorization, token!!)
                header("X-Super-Properties", getSuperProperties())
                contentType(ContentType.Application.Json)
                setBody(MessageRequest(content))
            }
            response.status.isSuccess()
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

    suspend fun getChannelMessages(channelId: String, limit: Int = 50): List<Message> {
        if (token == null) return emptyList()
        return try {
            httpClient.get("$apiBase/channels/$channelId/messages") {
                header(HttpHeaders.Authorization, token!!)
                parameter("limit", limit)
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
