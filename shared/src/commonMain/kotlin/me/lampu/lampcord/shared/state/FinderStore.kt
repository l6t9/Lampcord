package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.User

sealed class FinderResult {
    abstract val score: Float
    abstract val key: String
    
    data class Channel(
        val channel: me.lampu.lampcord.shared.model.Channel, 
        val guild: me.lampu.lampcord.shared.model.Guild?, 
        override val score: Float
    ) : FinderResult() {
        override val key: String = channel.id
    }
    
    data class DirectMessage(
        val channel: me.lampu.lampcord.shared.model.Channel, 
        override val score: Float
    ) : FinderResult() {
        override val key: String = channel.id
    }
    
    data class Guild(
        val guild: me.lampu.lampcord.shared.model.Guild, 
        override val score: Float
    ) : FinderResult() {
        override val key: String = guild.id
    }
}

class FinderStore(
    private val guildStore: GuildStore
) {
    var searchQuery by mutableStateOf("")
    
    // Tracks IDs of recently visited or messaged channels
    val recentChannelIds = mutableStateListOf<String>()

    val results by derivedStateOf {
        val query = searchQuery.lowercase().trim()
        val guilds = guildStore.guilds
        val allGuildChannels = guildStore.allGuildChannels
        val privateChannels = guildStore.privateChannels

        if (query.isEmpty()) {
            // Return recents
            val recents = mutableListOf<FinderResult>()
            recentChannelIds.distinct().take(20).forEach { id ->
                // Check if it's a DM
                privateChannels.find { it.id == id }?.let {
                    recents.add(FinderResult.DirectMessage(it, 0f))
                } ?: run {
                    // Check if it's a guild channel
                    allGuildChannels.forEach { (guildId, channels) ->
                        channels.find { it.id == id }?.let { channel ->
                            val guild = guilds.find { it.id == guildId }
                            recents.add(FinderResult.Channel(channel, guild, 0f))
                        }
                    }
                }
            }
            return@derivedStateOf recents
        }

        val list = mutableListOf<FinderResult>()

        // Search Guilds
        guilds.forEach { guild ->
            val name = guild.name?.lowercase() ?: ""
            if (name.contains(query)) {
                list.add(FinderResult.Guild(guild, if (name.startsWith(query)) 2f else 1f))
            }
        }

        // Search Channels
        allGuildChannels.forEach { (guildId, channels) ->
            val guild = guilds.find { it.id == guildId }
            channels.forEach { channel ->
                val name = channel.name?.lowercase() ?: ""
                if (name.contains(query)) {
                    list.add(FinderResult.Channel(channel, guild, if (name.startsWith(query)) 2f else 1f))
                }
            }
        }

        // Search DMs
        privateChannels.forEach { channel ->
            val name = channel.name?.lowercase() ?: ""
            val recipients = channel.recipients?.mapNotNull { it.username?.lowercase() } ?: emptyList()
            val globalNames = channel.recipients?.mapNotNull { it.global_name?.lowercase() } ?: emptyList()
            
            if (name.contains(query) || recipients.any { it.contains(query) } || globalNames.any { it.contains(query) }) {
                list.add(FinderResult.DirectMessage(channel, if (name.startsWith(query)) 2f else 1f))
            }
        }

        list.sortedByDescending { it.score }.distinctBy { it.key }
    }

    fun addRecent(channelId: String) {
        recentChannelIds.remove(channelId)
        recentChannelIds.add(0, channelId)
        if (recentChannelIds.size > 50) recentChannelIds.removeAt(recentChannelIds.lastIndex)
    }
}
