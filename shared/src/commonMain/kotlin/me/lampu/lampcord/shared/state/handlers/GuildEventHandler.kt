package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.*

class GuildEventHandler(
    private val json: Json,
    private val entityStore: EntityStore,
    private val guildStore: GuildStore,
    private val navigationStore: NavigationStore,
    private val settingsStore: SettingsStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("GUILD_CREATE", "GUILD_UPDATE", "GUILD_DELETE", "CHANNEL_UPDATE", "CHANNEL_DELETE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_CREATE" -> handleGuildCreate(data)
            "GUILD_UPDATE" -> handleGuildUpdate(data)
            "GUILD_DELETE" -> handleGuildDelete(data)
            "CHANNEL_UPDATE" -> handleChannelUpdate(data)
            "CHANNEL_DELETE" -> handleChannelDelete(data)
        }
    }

    private fun handleGuildCreate(data: JsonElement) {
        val guild = json.decodeFromJsonElement<Guild>(data)
        entityStore.updateGuild(guild)
        
        val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.toString() } ?: emptyList()
        guildStore.handleGuildCreate(guild, guildOrder)
        
        guild.members?.forEach { member ->
            entityStore.updateMember(guild.id, member)
        }
    }

    private fun handleGuildUpdate(data: JsonElement) {
        // ...
    }

    private fun handleGuildDelete(data: JsonElement) {
        // ...
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
