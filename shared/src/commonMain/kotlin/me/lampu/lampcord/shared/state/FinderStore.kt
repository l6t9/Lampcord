package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import kotlin.time.Duration.Companion.milliseconds

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
    private val guildStore: GuildStore,
    private val scope: CoroutineScope
) {
    private val _searchQuery = MutableStateFlow("")
    var searchQuery: String
        get() = _searchQuery.value
        set(value) { _searchQuery.value = value }

    private val _recentChannelIds = MutableStateFlow<List<String>>(emptyList())
    
    @OptIn(FlowPreview::class)
    val results: StateFlow<List<FinderResult>> = combine(
        _searchQuery.debounce(200.milliseconds),
        _recentChannelIds,
        guildStore.guilds,
        guildStore.privateChannels,
        guildStore.allGuildChannels
    ) { query, recents, guilds, privateChannels, allGuildChannels ->
        performSearch(query.lowercase().trim(), recents, guilds, privateChannels, allGuildChannels)
    }.flowOn(Dispatchers.Default)
    .stateIn(scope, SharingStarted.WhileSubscribed(5000), emptyList())

    private fun performSearch(
        query: String, 
        recents: List<String>,
        guilds: List<Guild>,
        privateChannels: List<Channel>,
        allGuildChannels: Map<String, Channel>
    ): List<FinderResult> {
        if (query.isEmpty()) {
            val list = mutableListOf<FinderResult>()
            recents.distinct().take(20).forEach { id ->
                val dm = privateChannels.find { it.id == id }
                if (dm != null) {
                    list.add(FinderResult.DirectMessage(dm, 0f))
                } else {
                    val channel = allGuildChannels[id]
                    if (channel != null) {
                        val guild = guilds.find { it.id == channel.guild_id }
                        list.add(FinderResult.Channel(channel, guild, 0f))
                    }
                }
            }
            return list
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
        allGuildChannels.values.forEach { channel ->
            val name = channel.name?.lowercase() ?: ""
            if (name.contains(query)) {
                val guild = guilds.find { it.id == channel.guild_id }
                list.add(FinderResult.Channel(channel, guild, if (name.startsWith(query)) 2f else 1f))
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

        return list.sortedByDescending { it.score }.distinctBy { it.key }
    }

    fun addRecent(channelId: String) {
        val current = _recentChannelIds.value.toMutableList()
        current.remove(channelId)
        current.add(0, channelId)
        if (current.size > 50) current.removeAt(current.lastIndex)
        _recentChannelIds.value = current
    }
}
