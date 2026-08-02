package com.example.materialcord.shared.api

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import com.example.materialcord.shared.model.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.websocket.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.json.Json
import kotlinx.serialization.Serializable

@Serializable
data class MessageRequest(val content: String)

class DiscordClient(
    private val httpClient: HttpClient
) {
    private var token: String? = null

    fun setToken(token: String) {
        this.token = token
    }

    suspend fun sendMessage(channelId: String, content: String): Boolean {
        if (token == null) return false
        
        return try {
            val response = httpClient.post("https://discord.com/api/v10/channels/$channelId/messages") {
                header(HttpHeaders.Authorization, token!!)
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
            httpClient.get("https://discord.com/api/v10/guilds/$guildId/channels") {
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
            httpClient.get("https://discord.com/api/v10/channels/$channelId/messages") {
                header(HttpHeaders.Authorization, token!!)
                parameter("limit", limit)
            }.body()
        } catch (e: Exception) {
            println("Error fetching messages: ${e.message}")
            emptyList()
        }
    }
}

fun createHttpClient() = HttpClient {
    install(ContentNegotiation) {
        json(Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
        })
    }
    install(WebSockets)
}
