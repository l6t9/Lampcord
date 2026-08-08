package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Role

class GuildStore {
    val guilds = mutableStateListOf<Guild>()
    val channels = mutableStateListOf<Channel>()
    val privateChannels = mutableStateListOf<Channel>()
    val forumThreads = mutableStateListOf<Channel>()
    
    var selectedGuild by mutableStateOf<Guild?>(null)
    var selectedChannel by mutableStateOf<Channel?>(null)
    var selectedThread by mutableStateOf<Channel?>(null)

    val allGuildChannels = mutableStateMapOf<String, List<Channel>>()

    fun setGuilds(newGuilds: List<me.lampu.lampcord.shared.model.Guild>, order: List<String>) {
        println("GuildStore received ${newGuilds.size} guilds")
        newGuilds.forEach { newGuild ->
            val index = guilds.indexOfFirst { it.id == newGuild.id }
            if (index != -1) {
                val existing = guilds[index]
                // Merge logic: Replicate 126.21 StoreGuilds.handleGuild
                guilds[index] = existing.copy(
                    name = newGuild.name ?: existing.name,
                    icon = newGuild.icon ?: existing.icon,
                    banner = newGuild.banner ?: existing.banner,
                    roles = if (newGuild.roles.isNotEmpty()) newGuild.roles else existing.roles,
                    features = newGuild.features ?: existing.features,
                    owner_id = newGuild.owner_id ?: existing.owner_id,
                    unavailable = newGuild.unavailable ?: existing.unavailable,
                    channels = newGuild.channels ?: existing.channels,
                    members = if (newGuild.members != null) newGuild.members else existing.members
                )
            } else {
                guilds.add(newGuild)
            }
            
            // Sync initial channels to allGuildChannels cache
            newGuild.channels?.let { guildChannels ->
                allGuildChannels[newGuild.id] = guildChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
            }
        }
        
        // Re-sort based on user_settings order (matches StoreGuildsSorted in 126.21)
        if (order.isNotEmpty()) {
            val sorted = guilds.sortedBy { guild -> 
                val pos = order.indexOf(guild.id)
                if (pos == -1) Int.MAX_VALUE else pos 
            }.toList()
            guilds.clear()
            guilds.addAll(sorted)
        }
    }

    fun handleGuildCreate(guild: me.lampu.lampcord.shared.model.Guild, order: List<String>) {
        println("GuildStore received GUILD_CREATE for ${guild.id}")
        val existingIndex = guilds.indexOfFirst { it.id == guild.id }
        if (existingIndex != -1) {
            val existing = guilds[existingIndex]
            guilds[existingIndex] = existing.copy(
                name = guild.name ?: existing.name,
                icon = guild.icon ?: existing.icon,
                banner = guild.banner ?: existing.banner,
                roles = if (guild.roles.isNotEmpty()) guild.roles else existing.roles,
                features = guild.features ?: existing.features,
                owner_id = guild.owner_id ?: existing.owner_id,
                unavailable = guild.unavailable ?: existing.unavailable,
                channels = guild.channels ?: existing.channels,
                members = guild.members ?: existing.members
            )
            if (selectedGuild?.id == guild.id) {
                selectedGuild = guilds[existingIndex]
            }
        } else {
            guilds.add(guild)
        }
        
        // Sync channels to cache
        guild.channels?.let { guildChannels ->
            allGuildChannels[guild.id] = guildChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
            if (selectedGuild?.id == guild.id) {
                channels.clear()
                channels.addAll(allGuildChannels[guild.id] ?: emptyList())
            }
        }
        
        // Re-sort
        if (order.isNotEmpty()) {
            val sorted = guilds.sortedBy { g -> 
                val pos = order.indexOf(g.id)
                if (pos == -1) Int.MAX_VALUE else pos 
            }
            guilds.clear()
            guilds.addAll(sorted)
        }
    }

    fun handleGuildDelete(guildId: String) {
        println("GuildStore received GUILD_DELETE for $guildId")
        guilds.removeAll { it.id == guildId }
        allGuildChannels.remove(guildId)
        if (selectedGuild?.id == guildId) {
            selectedGuild = null
            selectedChannel = null
        }
    }

    fun handleChannelCreateOrUpdate(channel: Channel) {
        val guildId = channel.guild_id ?: return
        val currentChannels = allGuildChannels[guildId]?.toMutableList() ?: mutableListOf()
        val index = currentChannels.indexOfFirst { it.id == channel.id }
        if (index != -1) {
            currentChannels[index] = channel
        } else {
            currentChannels.add(channel)
        }
        val sorted = currentChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
        allGuildChannels[guildId] = sorted
        
        if (selectedGuild?.id == guildId) {
            channels.clear()
            channels.addAll(sorted)
        }
    }

    fun handleChannelDelete(channel: Channel) {
        val guildId = channel.guild_id ?: return
        val currentChannels = allGuildChannels[guildId]?.toMutableList() ?: return
        currentChannels.removeAll { it.id == channel.id }
        allGuildChannels[guildId] = currentChannels
        
        if (selectedGuild?.id == guildId) {
            channels.clear()
            channels.addAll(currentChannels)
            if (selectedChannel?.id == channel.id) {
                selectedChannel = null
            }
        }
    }

    fun handleRoleCreateOrUpdate(guildId: String, role: Role) {
        val guildIndex = guilds.indexOfFirst { it.id == guildId }
        if (guildIndex != -1) {
            val guild = guilds[guildIndex]
            val currentRoles = guild.roles.toMutableList()
            val roleIndex = currentRoles.indexOfFirst { it.id == role.id }
            if (roleIndex != -1) {
                currentRoles[roleIndex] = role
            } else {
                currentRoles.add(role)
            }
            guilds[guildIndex] = guild.copy(roles = currentRoles)
            if (selectedGuild?.id == guildId) {
                selectedGuild = guilds[guildIndex]
            }
        }
    }

    fun handleRoleDelete(guildId: String, roleId: String) {
        val guildIndex = guilds.indexOfFirst { it.id == guildId }
        if (guildIndex != -1) {
            val guild = guilds[guildIndex]
            val currentRoles = guild.roles.toMutableList()
            currentRoles.removeAll { it.id == roleId }
            guilds[guildIndex] = guild.copy(roles = currentRoles)
            if (selectedGuild?.id == guildId) {
                selectedGuild = guilds[guildIndex]
            }
        }
    }

    fun setPrivateChannels(newChannels: List<Channel>) {
        println("GuildStore received ${newChannels.size} private channels")
        privateChannels.clear()
        privateChannels.addAll(newChannels)
    }

    fun clear() {
        guilds.clear()
        channels.clear()
        privateChannels.clear()
        forumThreads.clear()
        allGuildChannels.clear()
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
    }
}
