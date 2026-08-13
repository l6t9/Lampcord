package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.patch
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.model.ApplicationCommandIndex
import me.lampu.lampcord.shared.model.AuditLog
import me.lampu.lampcord.shared.model.Ban
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.InteractionRequest
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.Role
import me.lampu.lampcord.shared.model.UserGuildSettings
import me.lampu.lampcord.shared.utils.Logging

/**
 * Guilds, roles, bans, emojis, invites, members, commands and guild settings.
 */
class GuildApi(private val rest: RestClient) {

    suspend fun getGuild(guildId: String): Guild? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Guild", "Error fetching guild: ${e.message}")
            null
        }
    }

    suspend fun updateGuild(guildId: String, partial: Guild.Partial): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/guilds/$guildId") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(partial)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Guild", "Error updating guild: ${e.message}")
            false
        }
    }

    suspend fun getGuildRoles(guildId: String): List<Role> {
        return try {
            rest.httpClient.get("${rest.apiBase}/guilds/$guildId/roles") {
                standardHeaders(rest)
            }.body<List<Role>>()
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun createRole(guildId: String): Role? {
        return try {
            rest.httpClient.post("${rest.apiBase}/guilds/$guildId/roles") {
                standardHeaders(rest)
                setBody(mapOf("name" to "new role"))
            }.body<Role>()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun updateRole(guildId: String, roleId: String, partial: Role.Partial): Role? {
        return try {
            rest.httpClient.patch("${rest.apiBase}/guilds/$guildId/roles/$roleId") {
                standardHeaders(rest)
                setBody(partial)
            }.body<Role>()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun deleteRole(guildId: String, roleId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/guilds/$guildId/roles/$roleId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun getGuildBans(guildId: String): List<Ban> {
        return try {
            rest.httpClient.get("${rest.apiBase}/guilds/$guildId/bans") {
                standardHeaders(rest)
            }.body<List<Ban>>()
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun unbanUser(guildId: String, userId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/guilds/$guildId/bans/$userId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun getGuildEmojis(guildId: String): List<Emoji> {
        return try {
            rest.httpClient.get("${rest.apiBase}/guilds/$guildId/emojis") {
                standardHeaders(rest)
            }.body<List<Emoji>>()
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun createEmoji(guildId: String, name: String, image: String): Emoji? {
        return try {
            rest.httpClient.post("${rest.apiBase}/guilds/$guildId/emojis") {
                standardHeaders(rest)
                setBody(mapOf("name" to name, "image" to image))
            }.body<Emoji>()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun updateEmoji(guildId: String, emojiId: String, name: String): Emoji? {
        return try {
            rest.httpClient.patch("${rest.apiBase}/guilds/$guildId/emojis/$emojiId") {
                standardHeaders(rest)
                setBody(mapOf("name" to name))
            }.body<Emoji>()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun deleteEmoji(guildId: String, emojiId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/guilds/$guildId/emojis/$emojiId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun getGuildInvites(guildId: String): List<Invite> {
        return try {
            rest.httpClient.get("${rest.apiBase}/guilds/$guildId/invites") {
                standardHeaders(rest)
            }.body<List<Invite>>()
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun deleteInvite(inviteCode: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/invites/$inviteCode") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun getGuildAuditLog(guildId: String): AuditLog? {
        return try {
            rest.httpClient.get("${rest.apiBase}/guilds/$guildId/audit-logs") {
                standardHeaders(rest)
            }.body<AuditLog>()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun getGuildMembers(guildId: String, limit: Int = 100, after: String? = null): List<Member> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/members") {
                standardHeaders(rest)
                parameter("limit", limit)
                if (after != null) {
                    parameter("after", after)
                }
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                val errorBody = response.bodyAsText()
                Logging.e("Guild", "Error fetching members: $errorBody")
                emptyList()
            }
        } catch (e: Exception) {
            Logging.e("Guild", "Error fetching members: ${e.message}")
            emptyList()
        }
    }

    suspend fun searchGuildMembers(guildId: String, query: String = "", limit: Int = 100): List<Member> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/members/search") {
                standardHeaders(rest)
                parameter("query", query)
                parameter("limit", limit)
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                val errorBody = response.bodyAsText()
                Logging.e("Members", "Error searching members: $errorBody")
                emptyList()
            }
        } catch (e: Exception) {
            Logging.e("Guild", "Error searching members: ${e.message}")
            emptyList()
        }
    }

    suspend fun getGuildMember(guildId: String, userId: String): Member? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/members/$userId") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Guild", "Error fetching member: ${e.message}")
            null
        }
    }

    suspend fun leaveGuild(guildId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/users/@me/guilds/$guildId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Auth", "Error leaving guild: ${e.message}")
            false
        }
    }

    suspend fun getCommandIndex(guildId: String): ApplicationCommandIndex? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/application-command-index") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            Logging.e("Command", "Error getting command index: ${e.message}")
            null
        }
    }

    suspend fun sendInteraction(request: InteractionRequest): Boolean {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/interactions") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Interaction", "Error sending interaction: ${e.message}")
            false
        }
    }

    suspend fun onboardSelectedChannels(guildId: String, channelIds: List<String>): Boolean {
        return try {
            val response = rest.httpClient.put("${rest.apiBase}/guilds/$guildId/members/@me/channels") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("channel_ids", buildJsonArray {
                        channelIds.forEach { add(it) }
                    })
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Guild", "Error onboarding channels: ${e.message}")
            false
        }
    }

    suspend fun updateUserGuildSettings(guildId: String, settings: UserGuildSettings.Partial): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/users/@me/guilds/$guildId/settings") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(settings)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Guild", "Error updating guild settings: ${e.message}")
            false
        }
    }

    suspend fun modifyGuildMemberRoles(guildId: String, userId: String, roles: List<String>): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/guilds/$guildId/members/$userId") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(mapOf("roles" to roles))
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Guild", "Error updating member roles: ${e.message}")
            false
        }
    }
}
