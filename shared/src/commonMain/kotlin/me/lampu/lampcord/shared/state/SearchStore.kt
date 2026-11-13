package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings

class SearchStore(
    private val messageApi: MessageApi,
    private val memberListStore: MemberListStore,
    private val guildStore: GuildStore,
    private val userStore: UserStore,
    private val json: Json,
    private val scope: CoroutineScope
) {
    var isSearchVisible by mutableStateOf(false)
    var searchQuery by mutableStateOf("")
    val searchResults = mutableStateListOf<Message>()
    var isSearchLoading by mutableStateOf(false)
    var totalSearchResults by mutableStateOf(0)
    val searchHistory = mutableStateListOf<String>()

    init {
        try {
            val historyJson = Settings.shared.searchHistoryJson
            if (historyJson.isNotEmpty()) {
                val history = json.decodeFromString<List<String>>(historyJson)
                searchHistory.addAll(history)
            }
        } catch (e: Exception) { }
    }

    fun performSearch(selectedGuild: Guild?, selectedChannel: Channel?) {
        val query = searchQuery
        if (query.isBlank()) return

        if (!searchHistory.contains(query)) {
            searchHistory.add(0, query)
            if (searchHistory.size > 20) searchHistory.removeAt(searchHistory.lastIndex)
            Settings.shared.searchHistoryJson = json.encodeToString(searchHistory.toList())
        }

        isSearchLoading = true
        searchResults.clear()

        scope.launch {
            val parsed = parseSearchQuery(query)
            val authorIds = parsed.values("from").map { resolveUserValue(it, selectedGuild, selectedChannel) }
            val mentionIds = parsed.values("mentions").map { resolveUserValue(it, selectedGuild, selectedChannel) }
            val channelIds = parsed.values("in").map { resolveChannelValue(it) }
            val has = parsed.values("has")
            val excludes = parsed.values("exclude")
            val authorTypes = parsed.values("authortype")
            val sortOrder = parsed.values("sort").firstOrNull()?.let { if (it.equals("old", ignoreCase = true)) "asc" else "desc" }
            val before = parsed.values("before").firstOrNull()?.let(::resolveSearchDate)
            val after = parsed.values("after").firstOrNull()?.let(::resolveSearchDate)
            val during = parsed.values("during").firstOrNull()?.let(::resolveSearchDate)
            val content = parsed.text.takeIf { it.isNotBlank() }

            val response = if (selectedGuild != null) {
                messageApi.searchGuildMessages(
                    guildId = selectedGuild.id,
                    content = content,
                    authorIds = authorIds,
                    mentions = mentionIds,
                    has = has,
                    channelIds = channelIds,
                    before = before,
                    after = after,
                    during = during,
                    sortOrder = sortOrder,
                    authorTypes = authorTypes
                )
            } else if (selectedChannel != null) {
                messageApi.searchChannelMessages(
                    channelId = selectedChannel.id,
                    content = content,
                    authorIds = authorIds,
                    mentions = mentionIds,
                    has = has,
                    before = before,
                    after = after,
                    during = during,
                    sortOrder = sortOrder,
                    authorTypes = authorTypes
                )
            } else null

            response?.let {
                val hits = it.messages.flatMap { group ->
                    group.filter { message ->
                        message.hit && excludes.none { kind -> message.matchesSearchExclusion(kind) }
                    }
                }
                searchResults.addAll(hits)
                totalSearchResults = if (excludes.isEmpty()) it.total_results else hits.size
            }
            isSearchLoading = false
        }
    }

    private fun resolveUserValue(value: String, selectedGuild: Guild?, selectedChannel: Channel?): String {
        if (value.toLongOrNull() != null) return value
        val username = value.removePrefix("@")
        val member = if (selectedGuild != null) {
            memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.find {
                it.user?.username?.equals(username, ignoreCase = true) == true ||
                        it.nick?.equals(username, ignoreCase = true) == true ||
                        it.user?.global_name?.equals(username, ignoreCase = true) == true
            }
        } else if (selectedChannel != null && selectedChannel.guild_id == null) {
            val dmUsers = mutableListOf<User>()
            selectedChannel.recipients?.let { dmUsers.addAll(it) }
            if (selectedChannel.recipients.isNullOrEmpty() && !selectedChannel.recipient_ids.isNullOrEmpty()) {
                selectedChannel.recipient_ids.forEach { id ->
                    userStore.getUser(id)?.let { dmUsers.add(it) }
                }
            }
            userStore.currentUser.value?.let { dmUsers.add(it) }
            dmUsers.distinctBy { it.id }.map { me.lampu.lampcord.shared.model.Member(user = it) }.find {
                it.user?.username?.equals(username, ignoreCase = true) == true ||
                        it.user?.global_name?.equals(username, ignoreCase = true) == true
            }
        } else null

        return member?.user?.id ?: value
    }

    private fun resolveChannelValue(value: String): String {
        if (value.toLongOrNull() != null) return value
        val name = value.removePrefix("#")
        return guildStore.allGuildChannels.value.values
            .find { it.name?.equals(name, ignoreCase = true) == true }?.id ?: value
    }

    fun clear() {
        searchResults.clear()
        searchQuery = ""
        isSearchVisible = false
    }
}
