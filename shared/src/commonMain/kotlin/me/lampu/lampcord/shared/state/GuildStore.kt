package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild

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
        newGuilds.forEach { newGuild ->
            val index = guilds.indexOfFirst { it.id == newGuild.id }
            if (index != -1) {
                val existing = guilds[index]
                // Merge: prefer full data from existing if new one is partial
                guilds[index] = existing.copy(
                    name = newGuild.name ?: existing.name,
                    icon = newGuild.icon ?: existing.icon,
                    banner = newGuild.banner ?: existing.banner,
                    roles = if (newGuild.roles.isNotEmpty()) newGuild.roles else existing.roles,
                    features = newGuild.features ?: existing.features,
                    owner_id = newGuild.owner_id ?: existing.owner_id
                )
            } else {
                guilds.add(newGuild)
            }
        }
        
        // Re-sort
        val sorted = guilds.sortedBy { guild -> 
            val pos = order.indexOf(guild.id)
            if (pos == -1) Int.MAX_VALUE else pos 
        }.toList()
        guilds.clear()
        guilds.addAll(sorted)
    }

    fun handleGuildCreate(guild: me.lampu.lampcord.shared.model.Guild, order: List<String>) {
        val existingIndex = guilds.indexOfFirst { it.id == guild.id }
        if (existingIndex != -1) {
            guilds[existingIndex] = guild
            if (selectedGuild?.id == guild.id) {
                selectedGuild = guild
            }
        } else {
            guilds.add(guild)
        }
        
        // Re-sort
        val sorted = guilds.sortedBy { g -> 
            val pos = order.indexOf(g.id)
            if (pos == -1) Int.MAX_VALUE else pos 
        }
        guilds.clear()
        guilds.addAll(sorted)
    }

    fun handleGuildDelete(guildId: String) {
        guilds.removeAll { it.id == guildId }
        allGuildChannels.remove(guildId)
        if (selectedGuild?.id == guildId) {
            selectedGuild = null
            selectedChannel = null
        }
    }

    fun setPrivateChannels(newChannels: List<Channel>) {
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
