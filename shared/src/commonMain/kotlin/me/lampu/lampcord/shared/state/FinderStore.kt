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
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.User
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
        override val score: Float,
        val recipient: me.lampu.lampcord.shared.model.User? = null
    ) : FinderResult() {
        override val key: String = channel.id
    }

    data class Guild(
        val guild: me.lampu.lampcord.shared.model.Guild,
        override val score: Float
    ) : FinderResult() {
        override val key: String = guild.id
    }

    data class UserResult(
        val user: me.lampu.lampcord.shared.model.User,
        override val score: Float,
        val nickname: String? = null,
        val dmChannelId: String? = null
    ) : FinderResult() {
        override val key: String = "user:${user.id}"
    }
}

class FinderStore(
    private val guildStore: GuildStore,
    private val userStore: UserStore,
    private val navigationStoreProvider: () -> NavigationStore,
    private val gatewayManager: GatewayManager,
    private val scope: CoroutineScope
) {
    private val _searchQuery = MutableStateFlow("")

    val searchQueryFlow: StateFlow<String> = _searchQuery

    var searchQuery: String
        get() = _searchQuery.value
        set(value) {
            _searchQuery.value = value
        }

    private val _recentChannelIds = MutableStateFlow<List<String>>(emptyList())

    @OptIn(FlowPreview::class)
    val results: StateFlow<List<FinderResult>> = combine(
        _searchQuery.debounce(200.milliseconds),
        _recentChannelIds,
        guildStore.guilds,
        guildStore.privateChannels,
        guildStore.allGuildChannels,
        userStore.users,
        userStore.members
    ) { values ->
        @Suppress("UNCHECKED_CAST")
        performSearch(
            query = values[0] as String,
            recents = values[1] as List<String>,
            guilds = values[2] as List<Guild>,
            privateChannels = values[3] as List<Channel>,
            allGuildChannels = values[4] as Map<String, Channel>,
            users = values[5] as Map<String, User>,
            members = values[6] as Map<String, Map<String, Member>>
        )
    }.flowOn(Dispatchers.Default)
        .stateIn(scope, SharingStarted.WhileSubscribed(5000), emptyList())

    /**
     * Mirrors Discord: search covers the local cache, and typing a query additionally asks the
     * gateway to pull matching members of the selected server, so an offline member the client
     * has never seen still turns up.
     */
    @OptIn(FlowPreview::class)
    private val memberPrefetch = scope.launch {
        _searchQuery.debounce(900.milliseconds).collect { raw ->
            val query = raw.trim().removePrefix("@").removePrefix("from:")
            if (query.length < 2) return@collect
            val guildId = navigationStoreProvider().selectedGuild?.id ?: return@collect
            gatewayManager.requestGuildMembers(guildId, query)
        }
    }

    private fun performSearch(
        query: String,
        recents: List<String>,
        guilds: List<Guild>,
        privateChannels: List<Channel>,
        allGuildChannels: Map<String, Channel>,
        users: Map<String, User>,
        members: Map<String, Map<String, Member>>
    ): List<FinderResult> {
        val list = mutableListOf<FinderResult>()

        if (query.isEmpty()) {
            recents.distinct().take(20).forEach { id ->
                val dm = privateChannels.find { it.id == id }
                if (dm != null) {
                    list.add(FinderResult.DirectMessage(dm, 0f, FinderMatching.dmRecipient(dm, users)))
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

        guilds.forEach { guild ->
            val name = guild.name?.lowercase() ?: ""
            if (name.contains(query)) {
                list.add(FinderResult.Guild(guild, if (name.startsWith(query)) 2f else 1f))
            }
        }

        allGuildChannels.values.forEach { channel ->
            val name = channel.name?.lowercase() ?: ""
            if (name.contains(query)) {
                val guild = guilds.find { it.id == channel.guild_id }
                list.add(FinderResult.Channel(channel, guild, if (name.startsWith(query)) 2f else 1f))
            }
        }

        val dmByUser = HashMap<String, Channel>()
        privateChannels.forEach { channel ->
            val recipient = FinderMatching.dmRecipient(channel, users)
            if (recipient != null) dmByUser[recipient.id] = channel
        }

        privateChannels.forEach { channel ->
            val recipient = FinderMatching.dmRecipient(channel, users)
            val name = channel.name?.lowercase() ?: ""
            val aliases = buildList {
                recipient?.username?.lowercase()?.let { add(it) }
                recipient?.global_name?.lowercase()?.let { add(it) }
            }
            if (name.contains(query) || aliases.any { it.contains(query) }) {
                val prefix = name.startsWith(query) || aliases.any { it.startsWith(query) }
                list.add(
                    FinderResult.DirectMessage(channel, if (prefix) 2f else 1f, recipient)
                )
            }
        }

        if (query.isNotEmpty()) {
            users.values.forEach { user ->
                if (user.bot == true) return@forEach
                val aliases = FinderMatching.aliases(user, members)
                val rank = FinderMatching.rank(user, aliases, query)
                if (rank == null) return@forEach
                list.add(
                    FinderResult.UserResult(
                        user = user,
                        score = rank,
                        nickname = aliases.firstOrNull { it != user.username },
                        dmChannelId = dmByUser[user.id]?.id
                    )
                )
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