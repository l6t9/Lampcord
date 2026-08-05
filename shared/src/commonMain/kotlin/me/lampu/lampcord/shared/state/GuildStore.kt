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

    fun setGuilds(newGuilds: List<Guild>, order: List<String>) {
        val sorted = newGuilds.sortedBy { guild -> 
            val pos = order.indexOf(guild.id)
            if (pos == -1) Int.MAX_VALUE else pos 
        }
        guilds.clear()
        guilds.addAll(sorted)
    }

    fun handleGuildCreate(guild: Guild, order: List<String>) {
        val existingIndex = guilds.indexOfFirst { it.id == guild.id }
        if (existingIndex != -1) {
            guilds[existingIndex] = guild
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
