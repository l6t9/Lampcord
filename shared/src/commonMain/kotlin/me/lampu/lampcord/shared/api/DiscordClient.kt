package me.lampu.lampcord.shared.api

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.client.request.forms.*
import io.ktor.http.content.*
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.plugins.cookies.*
import io.ktor.client.engine.cio.*
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
    val nonce: String? = null,
    val attachments: List<AttachmentRequest>? = null,
    val sticker_ids: List<String>? = null,
    val allowed_mentions: AllowedMentions? = null
)

@Serializable
data class AllowedMentions(
    val parse: List<String>? = null,
    val roles: List<String>? = null,
    val users: List<String>? = null,
    val replied_user: Boolean? = null
)

@Serializable
data class AttachmentRequest(
    val id: String,
    val filename: String,
    val description: String? = null
)

class DiscordClient(
    private val httpClient: HttpClient,
    private val json: Json
) {
    private var token: String? = null
    private val apiVersion = 9
    private val apiBase = "https://discord.com/api/v$apiVersion"
    val vendorId = randomUUID()
    val launchId = randomUUID()
    val heartbeatSessionId = randomUUID()
    val launchSignature = (getCurrentTimeMillis() * 1_000_000L).toString()

    fun setToken(token: String?) {
        this.token = token
    }

    @OptIn(ExperimentalEncodingApi::class)
    private fun getSuperProperties(): String {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        
        val properties = buildJsonObject {
            put("os", if (platform == "macos") "Mac OS X" else platform.replaceFirstChar { it.uppercase() })
            put("browser", if (isMobile) "Discord Android" else "Discord Client")
            put("device", getDeviceName())
            put("system_locale", "en-US")
            put("has_client_mods", false)
            put("client_version", if (isMobile) "341.0 - rn" else "0.0.398")
            put("release_channel", if (isMobile) "canaryRelease" else "stable")
            put("device_vendor_id", vendorId)
            put("design_id", 2)
            put("browser_user_agent", if (isMobile) {
                if (platform == "android") "Discord-Android/341200;RNA" else "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
            } else "")
            put("browser_version", if (isMobile) "" else "37.6.0")
            put("os_version", getOsVersion())
            put("client_build_number", if (isMobile) 6081 else 575562)
            put("client_event_source", JsonNull)
            if (isMobile) {
                put("client_launch_id", launchId)
                put("launch_signature", launchSignature)
                put("client_app_state", "active")
                put("client_heartbeat_session_id", heartbeatSessionId)
            } else {
                put("client_launch_id", JsonNull)
                put("launch_signature", "")
                put("client_app_state", JsonNull)
                put("client_heartbeat_session_id", JsonNull)
            }
        }
        val jsonString = json.encodeToString(properties)
        return Base64.encode(jsonString.encodeToByteArray())
    }

    private fun HttpRequestBuilder.standardHeaders() {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        
        val userAgent = if (isMobile) {
            if (platform == "android") {
                "Discord-Android/341200;RNA"
            } else {
                "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
            }
        } else {
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
        }

        header("User-Agent", userAgent)
        header("Accept-Language", "en-US,en;q=0.9")
        header("X-Super-Properties", getSuperProperties())
        header("X-Discord-Locale", "en-US")
        token?.let { header(HttpHeaders.Authorization, it) }
    }

    private fun HttpRequestBuilder.loginHeaders(fingerprint: String? = null) {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        val userAgent = if (isMobile) {
            if (platform == "android") "Discord-Android/341200;RNA" else "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
        } else {
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
        }

        header("User-Agent", userAgent)
        header("Accept-Language", "en-US,en;q=0.9")
        header("X-Super-Properties", getSuperProperties())
        fingerprint?.let { header("X-Fingerprint", it) }
        header("Origin", "https://discord.com")
        header("Referer", "https://discord.com/login")
    }

    suspend fun getFingerprint(): String? {
        return try {
            val response: FingerprintResponse = httpClient.get("$apiBase/experiments") {
                loginHeaders()
            }.body()
            response.fingerprint
        } catch (e: Exception) {
            println("Error fetching fingerprint: ${e.message}")
            null
        }
    }

    suspend fun login(request: LoginRequest, fingerprint: String): LoginResponse? {
        return try {
            val response = httpClient.post("$apiBase/auth/login") {
                loginHeaders(fingerprint)
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
                loginHeaders(fingerprint)
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
        forwardFrom: Message? = null,
        files: List<Pair<String, ByteArray>> = emptyList(),
        nonce: String? = null,
        stickerIds: List<String>? = null,
        allowedMentions: AllowedMentions? = null
    ): Boolean {
        val messageReference = when {
            replyTo != null -> MessageReference(message_id = replyTo)
            forwardFrom != null -> MessageReference(
                type = 1,
                message_id = forwardFrom.id,
                channel_id = forwardFrom.channel_id,
                guild_id = forwardFrom.guild_id
            )
            else -> null
        }

        return try {
            if (files.isEmpty()) {
                val response = httpClient.post("$apiBase/channels/$channelId/messages") {
                    standardHeaders()
                    contentType(ContentType.Application.Json)
                    
                    val request = MessageRequest(
                        content = content,
                        message_reference = messageReference,
                        nonce = nonce,
                        sticker_ids = stickerIds,
                        allowed_mentions = allowedMentions
                    )
                    setBody(request)
                }
                response.status.isSuccess()
            } else {
                val response = httpClient.post("$apiBase/channels/$channelId/messages") {
                    standardHeaders()
                    
                    val attachmentMetadata = files.mapIndexed { index, (name, _) ->
                        AttachmentRequest(id = index.toString(), filename = name)
                    }
                    
                    setBody(MultiPartFormDataContent(
                        formData {
                            append("payload_json", json.encodeToString(MessageRequest(
                                content = content,
                                message_reference = messageReference,
                                nonce = nonce,
                                attachments = attachmentMetadata,
                                sticker_ids = stickerIds,
                                allowed_mentions = allowedMentions
                            )), Headers.build {
                                append(HttpHeaders.ContentType, ContentType.Application.Json.toString())
                            })
                            files.forEachIndexed { index, (name, bytes) ->
                                val contentType = when {
                                    name.endsWith(".png", true) -> ContentType.Image.PNG
                                    name.endsWith(".jpg", true) || name.endsWith(".jpeg", true) -> ContentType.Image.JPEG
                                    name.endsWith(".gif", true) -> ContentType.Image.GIF
                                    name.endsWith(".webp", true) -> ContentType.parse("image/webp")
                                    name.endsWith(".mp4", true) -> ContentType.Video.MP4
                                    name.endsWith(".mov", true) -> ContentType.Video.QuickTime
                                    name.endsWith(".webm", true) -> ContentType.Video.Any
                                    else -> ContentType.Application.OctetStream
                                }
                                append("files[$index]", bytes, Headers.build {
                                    append(HttpHeaders.ContentType, contentType.toString())
                                    append(HttpHeaders.ContentDisposition, "form-data; name=\"files[$index]\"; filename=\"$name\"")
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

    suspend fun getGuild(guildId: String): Guild? {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            println("Error fetching guild: ${e.message}")
            null
        }
    }

    suspend fun updateGuild(guildId: String, partial: Guild.Partial): Boolean {
        return try {
            val response = httpClient.patch("$apiBase/guilds/$guildId") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error updating guild: ${e.message}")
            false
        }
    }

    suspend fun getGuildChannels(guildId: String): List<Channel> {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/channels") {
                standardHeaders()
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                if (response.status.value == 429) {
                    println("Rate limited while fetching channels for guild $guildId")
                }
                emptyList()
            }
        } catch (e: Exception) {
            println("Error fetching channels: ${e.message}")
            emptyList()
        }
    }

    suspend fun getPrivateChannels(): List<Channel> {
        return try {
            val response = httpClient.get("$apiBase/users/@me/channels") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            println("Error fetching DMs: ${e.message}")
            emptyList()
        }
    }

    suspend fun getChannelMessages(channelId: String, limit: Int = 50, before: String? = null): List<Message> {
        return try {
            val response = httpClient.get("$apiBase/channels/$channelId/messages") {
                standardHeaders()
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            println("Error fetching messages: ${e.message}")
            emptyList()
        }
    }

    suspend fun getActiveThreads(channelId: String): ThreadListResponse? {
        return try {
            httpClient.get("$apiBase/channels/$channelId/threads/active") {
                standardHeaders()
            }.body()
        } catch (e: Exception) {
            println("Error fetching active threads: ${e.message}")
            null
        }
    }

    suspend fun getArchivedPublicThreads(channelId: String, limit: Int = 100, before: String? = null): ThreadListResponse? {
        return try {
            httpClient.get("$apiBase/channels/$channelId/threads/archived/public") {
                standardHeaders()
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

    suspend fun getGuildOnboarding(guildId: String): Onboarding? {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/onboarding") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            println("Error fetching guild onboarding: ${e.message}")
            null
        }
    }

    suspend fun getGuildMembers(guildId: String, limit: Int = 100, after: String? = null): List<Member> {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/members") {
                standardHeaders()
                parameter("limit", limit)
                if (after != null) {
                    parameter("after", after)
                }
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

    suspend fun getGuildMember(guildId: String, userId: String): Member? {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/members/$userId") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            println("Error fetching member: ${e.message}")
            null
        }
    }

    suspend fun getUserProfile(userId: String, guildId: String? = null): UserProfile? {
        return try {
            val url = if (guildId != null) {
                "$apiBase/users/$userId/profile?guild_id=$guildId"
            } else {
                "$apiBase/users/$userId/profile"
            }
            val response = httpClient.get(url) {
                standardHeaders()
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                val errorBody = response.bodyAsText()
                println("Error fetching user profile: ${response.status}, body: $errorBody")
                null
            }
        } catch (e: Exception) {
            println("Error fetching user profile: ${e.message}")
            null
        }
    }

    suspend fun editMessage(channelId: String, messageId: String, content: String): Boolean {
        return try {
            val response = httpClient.patch("$apiBase/channels/$channelId/messages/$messageId") {
                standardHeaders()
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
        return try {
            val response = httpClient.delete("$apiBase/channels/$channelId/messages/$messageId") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error deleting message: ${e.message}")
            false
        }
    }

    suspend fun getRelationships(): List<Relationship> {
        return try {
            val response = httpClient.get("$apiBase/users/@me/relationships") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            println("Error fetching relationships: ${e.message}")
            emptyList()
        }
    }

    suspend fun addRelationship(userId: String, type: Int): Boolean {
        return try {
            val response = httpClient.put("$apiBase/users/@me/relationships/$userId") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("type", type) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error adding relationship: ${e.message}")
            false
        }
    }

    suspend fun deleteRelationship(userId: String): Boolean {
        return try {
            val response = httpClient.delete("$apiBase/users/@me/relationships/$userId") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error deleting relationship: ${e.message}")
            false
        }
    }

    suspend fun addReaction(channelId: String, messageId: String, emoji: String): Boolean {
        return try {
            val response = httpClient.put("$apiBase/channels/$channelId/messages/$messageId/reactions/$emoji/@me") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error adding reaction: ${e.message}")
            false
        }
    }

    suspend fun removeReaction(channelId: String, messageId: String, emoji: String): Boolean {
        return try {
            val response = httpClient.delete("$apiBase/channels/$channelId/messages/$messageId/reactions/$emoji/@me") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error removing reaction: ${e.message}")
            false
        }
    }

    suspend fun pinMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = httpClient.put("$apiBase/channels/$channelId/pins/$messageId") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error pinning message: ${e.message}")
            false
        }
    }

    suspend fun unpinMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = httpClient.delete("$apiBase/channels/$channelId/pins/$messageId") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error unpinning message: ${e.message}")
            false
        }
    }

    suspend fun getPinnedMessages(channelId: String): List<Message> {
        return try {
            val response = httpClient.get("$apiBase/channels/$channelId/pins") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            println("Error fetching pinned messages: ${e.message}")
            emptyList()
        }
    }

    suspend fun sendTyping(channelId: String): Boolean {
        return try {
            val response = httpClient.post("$apiBase/channels/$channelId/typing") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun updateStatus(status: String): Boolean {
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                standardHeaders()
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
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                standardHeaders()
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
        return try {
            val response = httpClient.post("$apiBase/channels/$channelId/messages/$messageId/ack") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("token", JsonNull) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error acking message: ${e.message}")
            false
        }
    }

    suspend fun ackBulk(channelIds: List<String>): Boolean {
        return try {
            val response = httpClient.post("$apiBase/read-states/ack-bulk") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("read_states", buildJsonArray {
                        channelIds.forEach { id ->
                            add(buildJsonObject { put("channel_id", id); put("message_id", "99999999999999999999") }) // Hack to mark all as read
                        }
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) { false }
    }

    suspend fun leaveGuild(guildId: String): Boolean {
        return try {
            val response = httpClient.delete("$apiBase/users/@me/guilds/$guildId") {
                standardHeaders()
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error leaving guild: ${e.message}")
            false
        }
    }

    suspend fun getCommandIndex(guildId: String): ApplicationCommandIndex? {
        return try {
            val response = httpClient.get("$apiBase/guilds/$guildId/application-command-index") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            println("Error getting command index: ${e.message}")
            null
        }
    }

    suspend fun sendInteraction(request: InteractionRequest): Boolean {
        return try {
            val response = httpClient.post("$apiBase/interactions") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error sending interaction: ${e.message}")
            false
        }
    }

    suspend fun getUserSettings(): UserSettings? {
        return try {
            val response = httpClient.get("$apiBase/users/@me/settings") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            println("Error fetching user settings: ${e.message}")
            null
        }
    }

    suspend fun getGuildFolders(): List<GuildFolder> {
        return getUserSettings()?.guild_folders ?: emptyList()
    }

    suspend fun updateUserSettings(partial: UserSettings.Partial): Boolean {
        return try {
            val response = httpClient.patch("$apiBase/users/@me/settings") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error updating user settings: ${e.message}")
            false
        }
    }

    suspend fun updateUserGuildSettings(guildId: String, settings: UserGuildSettings.Partial): Boolean {
        return try {
            val response = httpClient.patch("$apiBase/users/@me/guilds/$guildId/settings") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(settings)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error updating guild settings: ${e.message}")
            false
        }
    }

    suspend fun onboardSelectedChannels(guildId: String, channelIds: List<String>): Boolean {
        return try {
            val response = httpClient.put("$apiBase/guilds/$guildId/members/@me/channels") {
                standardHeaders()
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("channel_ids", buildJsonArray {
                        channelIds.forEach { add(it) }
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error onboarding channels: ${e.message}")
            false
        }
    }

    suspend fun getConnections(): List<ConnectedAccount> {
        return try {
            httpClient.get("$apiBase/users/@me/connections") {
                standardHeaders()
            }.body()
        } catch (e: Exception) {
            println("Error fetching connections: ${e.message}")
            emptyList()
        }
    }

    suspend fun getDevices(): List<DiscordDevice> {
        return try {
            val response = httpClient.get("$apiBase/users/@me/devices") {
                standardHeaders()
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            println("Error fetching devices: ${e.message}")
            emptyList()
        }
    }

    suspend fun exchangeRemoteAuthTicket(ticket: String): String? {
        return try {
            val response = httpClient.post("$apiBase/users/@me/remote-auth/login") {
                loginHeaders()
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("ticket", ticket) })
            }
            if (response.status.isSuccess()) {
                val body = response.body<JsonObject>()
                body["encrypted_token"]?.jsonPrimitive?.content
            } else {
                null
            }
        } catch (e: Exception) {
            println("Error exchanging remote auth ticket: ${e.message}")
            null
        }
    }

    suspend fun getGatewayUrl(): String? {
        return try {
            val response = httpClient.get("$apiBase/gateway") {
                standardHeaders()
            }
            if (response.status.isSuccess()) {
                val body = response.body<JsonObject>()
                body["url"]?.jsonPrimitive?.content
            } else null
        } catch (e: Exception) { null }
    }

    fun getGuildIconUrl(guildId: String, iconHash: String?, size: Int = 1024): String? {
        if (iconHash == null) return null
        return "https://cdn.discordapp.com/icons/$guildId/$iconHash.webp?size=$size"
    }

    fun getUserAvatarUrl(userId: String, avatarHash: String?, size: Int = 1024): String? {
        if (avatarHash == null) return null
        val extension = if (avatarHash.startsWith("a_")) "gif" else "webp"
        return "https://cdn.discordapp.com/avatars/$userId/$avatarHash.$extension?size=$size"
    }
}

@Serializable
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

fun createHttpClient() = HttpClient(CIO) {
    install(HttpCookies)
    install(ContentNegotiation) {
        json(Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
        })
    }
    install(WebSockets)
}
