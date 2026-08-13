package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.patch
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlin.time.Duration.Companion.seconds
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.Onboarding
import me.lampu.lampcord.shared.model.ThreadListResponse
import me.lampu.lampcord.shared.utils.Logging

/**
 * Channels, threads, read-state and onboarding endpoints.
 */
class ChannelApi(private val rest: RestClient) {

    suspend fun getGuildChannels(guildId: String): List<Channel> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/channels") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                if (response.status.value == 429) {
                    Logging.w("RateLimit", "Rate limited while fetching channels for guild $guildId")
                }
                emptyList()
            }
        } catch (e: Exception) {
            Logging.e("Guild", "Error fetching channels: ${e.message}")
            emptyList()
        }
    }

    suspend fun getChannel(channelId: String): Channel? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/channels/$channelId") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Channel", "Error fetching channel: ${e.message}")
            null
        }
    }

    suspend fun createThread(
        channelId: String,
        name: String,
        content: String,
        appliedTags: List<String> = emptyList()
    ): Channel? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/threads") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("name", name)
                    put("auto_archive_duration", 4320)
                    put("message", buildJsonObject {
                        put("content", content)
                    })
                    put("applied_tags", buildJsonArray {
                        appliedTags.forEach { add(it) }
                    })
                })
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Thread", "Error creating thread: ${e.message}")
            null
        }
    }

    suspend fun createThreadFromMessage(
        channelId: String,
        messageId: String,
        name: String
    ): Channel? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/messages/$messageId/threads") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("name", name)
                    put("auto_archive_duration", 4320)
                })
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Thread", "Error creating thread from message: ${e.message}")
            null
        }
    }

    suspend fun updateChannel(channelId: String, name: String?, topic: String? = null, nsfw: Boolean? = null): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/channels/$channelId") {
                standardHeaders(rest)
                val body = mutableMapOf<String, Any?>()
                if (name != null) body["name"] = name
                if (topic != null) body["topic"] = topic
                if (nsfw != null) body["nsfw"] = nsfw
                setBody(body)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun deleteChannel(channelId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/channels/$channelId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun getPrivateChannels(): List<Channel> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/channels") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("DM", "Error fetching DMs: ${e.message}")
            emptyList()
        }
    }

    suspend fun getActiveThreads(channelId: String): ThreadListResponse? {
        return try {
            rest.httpClient.get("${rest.apiBase}/channels/$channelId/threads/active") {
                standardHeaders(rest)
            }.body()
        } catch (e: Exception) {
            Logging.e("Threads", "Error fetching active threads: ${e.message}")
            null
        }
    }

    suspend fun searchThreads(channelId: String, limit: Int = 25): ThreadListResponse? {
        return try {
            rest.httpClient.get("${rest.apiBase}/channels/$channelId/threads/search") {
                standardHeaders(rest)
                parameter("limit", limit)
                parameter("sort_by", "last_message_at")
                parameter("sort_order", "desc")
            }.body()
        } catch (e: Exception) {
            Logging.e("Threads", "Error searching threads: ${e.message}")
            null
        }
    }

    suspend fun getArchivedPublicThreads(channelId: String, limit: Int = 100, before: String? = null): ThreadListResponse? {
        return try {
            rest.httpClient.get("${rest.apiBase}/channels/$channelId/threads/archived/public") {
                standardHeaders(rest)
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }.body()
        } catch (e: Exception) {
            Logging.e("Threads", "Error fetching archived threads: ${e.message}")
            null
        }
    }

    suspend fun getGuildOnboarding(guildId: String): Onboarding? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/onboarding") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Guild", "Error fetching guild onboarding: ${e.message}")
            null
        }
    }

    suspend fun getPinnedMessages(channelId: String): List<Message> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/channels/$channelId/pins") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Pinned", "Error fetching pinned messages: ${e.message}")
            emptyList()
        }
    }

    suspend fun pinMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = rest.httpClient.put("${rest.apiBase}/channels/$channelId/pins/$messageId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Message", "Error pinning message: ${e.message}")
            false
        }
    }

    suspend fun unpinMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/channels/$channelId/pins/$messageId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Message", "Error unpinning message: ${e.message}")
            false
        }
    }

    suspend fun triggerTyping(channelId: String): Boolean {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/typing") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun ackMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/messages/$messageId/ack") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("token", JsonNull) })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Message", "Error acking message: ${e.message}")
            false
        }
    }

    suspend fun ackBulk(channelIds: List<String>): Boolean {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/read-states/ack-bulk") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("read_states", buildJsonArray {
                        channelIds.forEach { id ->
                            add(buildJsonObject { put("channel_id", id); put("message_id", "99999999999999999999") })
                        }
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) { false }
    }

    suspend fun muteChannelForDuration(channelId: String, guildId: String?, durationSeconds: Long): Boolean {
        return try {
            val settingsGuildId = guildId ?: "@me"
            val endTime = (kotlin.time.Clock.System.now() + durationSeconds.seconds).toString()
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/guilds/$settingsGuildId/settings") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("channel_overrides", buildJsonArray {
                        add(buildJsonObject {
                            put("channel_id", channelId)
                            put("muted", true)
                            put("mute_config", buildJsonObject {
                                put("selected_time_window", durationSeconds)
                                put("duration", durationSeconds)
                                put("end_time", endTime)
                            })
                        })
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            println("Error muting channel: ${e.message}")
            false
        }
    }
}
