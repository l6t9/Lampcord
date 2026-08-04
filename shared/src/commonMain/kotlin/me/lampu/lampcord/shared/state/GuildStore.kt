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

    fun handleGuildCreate(guild: Guild) {
        val index = guilds.indexOfFirst { it.id == guild.id }
        if (index == -1) {
            guilds.add(guild)
        } else {
            guilds[index] = guild
        }
    }
}
