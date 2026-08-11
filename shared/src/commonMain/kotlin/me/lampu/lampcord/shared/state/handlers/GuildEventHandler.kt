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
    private val userStore: UserStore
) : GatewayEventHandler {
    override val supportedEvents = setOf(
        "GUILD_CREATE", "GUILD_UPDATE", "GUILD_DELETE",
        "CHANNEL_CREATE", "CHANNEL_UPDATE", "CHANNEL_DELETE"
    )

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_CREATE" -> handleGuildCreate(data)
            "GUILD_UPDATE" -> handleGuildUpdate(data)
            "GUILD_DELETE" -> handleGuildDelete(data)
            "CHANNEL_CREATE" -> handleChannelUpdate(data)
            "CHANNEL_UPDATE" -> handleChannelUpdate(data)
            "CHANNEL_DELETE" -> handleChannelDelete(data)
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
}
