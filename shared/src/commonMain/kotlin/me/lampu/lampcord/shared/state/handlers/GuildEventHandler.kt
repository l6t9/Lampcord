package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.*

/**
 * GuildEventHandler handles server and channel lifecycle events,
 * distributing state updates to EntityStore and UserStore.
 */
class GuildEventHandler(
    private val json: Json,
    private val entityStore: EntityStore,
    private val guildStore: GuildStore,
    private val navigationStore: NavigationStore,
    private val settingsStore: SettingsStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore
) : GatewayEventHandler {
    override val supportedEvents = setOf(
        "GUILD_CREATE", "GUILD_UPDATE", "GUILD_DELETE",
        "CHANNEL_CREATE", "CHANNEL_UPDATE", "CHANNEL_DELETE",
        "THREAD_CREATE", "THREAD_UPDATE", "THREAD_DELETE", "THREAD_LIST_SYNC"
    )

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_CREATE" -> handleGuildCreate(data)
            "GUILD_UPDATE" -> handleGuildUpdate(data)
            "GUILD_DELETE" -> handleGuildDelete(data)
            "CHANNEL_CREATE", "THREAD_CREATE" -> handleChannelUpdate(data)
            "CHANNEL_UPDATE", "THREAD_UPDATE" -> handleChannelUpdate(data)
            "CHANNEL_DELETE", "THREAD_DELETE" -> handleChannelDelete(data)
            "THREAD_LIST_SYNC" -> handleThreadListSync(data)
        }
    }

    private fun handleGuildCreate(data: JsonElement) {
        val guild = json.decodeFromJsonElement<Guild>(data)
        // EntityStore handles guilds and nested channels
        entityStore.updateGuild(guild)
        
        val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() } ?: emptyList()
        guildStore.handleGuildCreate(guild, guildOrder)
        
        // Members go to UserStore
        guild.members?.forEach { member ->
            val userId = member.userId() ?: return@forEach
            userStore.cacheMember(guild.id, userId, member)
        }

        // Presences go to PresenceStore
        guild.presences?.forEach { presence ->
            presenceStore.handlePresenceUpdate(presence.copy(guild_id = guild.id))
        }
    }

    private fun handleGuildUpdate(data: JsonElement) {
        val guild = json.decodeFromJsonElement<Guild>(data)
        entityStore.updateGuild(guild)
    }

    private fun handleGuildDelete(data: JsonElement) {
        val jsonObject = data as? JsonObject ?: return
        val id = jsonObject["id"]?.jsonPrimitive?.content ?: return
        val unavailable = jsonObject["unavailable"]?.jsonPrimitive?.booleanOrNull ?: false
        
        if (!unavailable) {
            guildStore.handleGuildDelete(id)
            entityStore.removeGuild(id)
        }
    }

    private fun handleChannelUpdate(data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.updateChannel(channel)
        guildStore.handleChannelCreateOrUpdate(channel)
    }

    private fun handleChannelDelete(data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.removeChannel(channel.id)
        guildStore.handleChannelDelete(channel)
    }

    private fun handleThreadListSync(data: JsonElement) {
        val obj = data.jsonObject
        val threads = obj["threads"]?.jsonArray?.map { json.decodeFromJsonElement<Channel>(it) } ?: emptyList()
        val guildId = obj["guild_id"]?.jsonPrimitive?.content ?: return
        
        threads.forEach { 
            val thread = it.copy(guild_id = guildId)
            entityStore.updateChannel(thread)
            guildStore.handleChannelCreateOrUpdate(thread)
        }
    }
}
