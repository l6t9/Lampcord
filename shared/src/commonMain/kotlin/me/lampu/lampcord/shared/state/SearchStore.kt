package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.SearchResponse
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings

class SearchStore(
    private val messageApi: MessageApi,
    private val guildApi: GuildApi,
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
    var isLoadingMore by mutableStateOf(false)
    var searchFailed by mutableStateOf(false)
    var unresolvedUser by mutableStateOf<String?>(null)
    var activeTab by mutableStateOf("messages")
    var totalSearchResults by mutableStateOf(0)
    val searchHistory = mutableStateListOf<String>()

    private var activeSearch: ResolvedSearch? = null
    private var searchOffset = 0
    private var apiTotal = 0
    private var lastPageFull = false
    private var generation = 0

    val hasMoreResults: Boolean
        get() = searchOffset <= 9_975 && (searchOffset < apiTotal || lastPageFull)

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
        if (query.isBlank() && activeTab == "messages") return

        if (query.isNotBlank() && !searchHistory.contains(query)) {
            searchHistory.add(0, query)
            if (searchHistory.size > 20) searchHistory.removeAt(searchHistory.lastIndex)
            Settings.shared.searchHistoryJson = json.encodeToString(searchHistory.toList())
        }

        searchOffset = 0
        apiTotal = 0
        lastPageFull = false
        isSearchLoading = true
        searchResults.clear()
        searchFailed = false
        unresolvedUser = null
        val gen = ++generation

        scope.launch {
            val search = resolveSearch(query, selectedGuild, selectedChannel)
            if (gen != generation) return@launch
            val unresolved = search.unresolvedUser
            if (unresolved != null) {
                unresolvedUser = unresolved
                isSearchLoading = false
                return@launch
            }
            activeSearch = search
            val responses = fetchBatch(search, selectedGuild, selectedChannel, 0)
            if (gen == generation) {
                appendBatch(responses)
                searchFailed = responses.firstOrNull() == null
                isSearchLoading = false
            }
        }
    }

    fun loadMoreResults(selectedGuild: Guild?, selectedChannel: Channel?) {
        if (isSearchLoading || isLoadingMore || !hasMoreResults) return
        val search = activeSearch ?: return
        isLoadingMore = true
        val offset = searchOffset
        val gen = generation
        scope.launch {
            val responses = fetchBatch(search, selectedGuild, selectedChannel, offset)
            if (gen == generation) appendBatch(responses)
            isLoadingMore = false
        }
    }

    private fun tabHasFilters(): List<String> = when (activeTab) {
        "media" -> listOf("image")
        "links" -> listOf("link")
        "files" -> listOf("file")
        else -> emptyList()
    }

    private class ResolvedSearch(
        val content: String?,
        val authorIds: List<String>,
        val mentionIds: List<String>,
        val has: List<String>,
        val channelIds: List<String>,
        val authorTypes: List<String>,
        val sortOrder: String?,
        val minId: String?,
        val maxId: String?,
        val unresolvedUser: String? = null,
    ) {
        companion object {
            fun unresolved(name: String) = ResolvedSearch(
                content = null,
                authorIds = emptyList(),
                mentionIds = emptyList(),
                has = emptyList(),
                channelIds = emptyList(),
                authorTypes = emptyList(),
                sortOrder = null,
                minId = null,
                maxId = null,
                unresolvedUser = name,
            )
        }
    }

    private suspend fun resolveSearch(query: String, selectedGuild: Guild?, selectedChannel: Channel?): ResolvedSearch {
        val parsed = parseSearchQuery(query)
        var minId: String? = null
        var maxId: String? = null
        parsed.tokens.filter { it.key == "before" || it.key == "after" || it.key == "during" }.forEach { token ->
            val bounds = searchDateBounds(token.key, token.value) ?: return@forEach
            bounds.first?.let { minId = it }
            bounds.second?.let { maxId = it }
        }
        val authorIds = mutableListOf<String>()
        for (name in parsed.values("from")) {
            authorIds += resolveUserValue(name, selectedGuild, selectedChannel) ?: return ResolvedSearch.unresolved(name)
        }
        val mentionIds = mutableListOf<String>()
        for (name in parsed.values("mentions")) {
            mentionIds += resolveUserValue(name, selectedGuild, selectedChannel) ?: return ResolvedSearch.unresolved(name)
        }
        return ResolvedSearch(
            content = parsed.text.takeIf { it.isNotBlank() },
            authorIds = authorIds,
            mentionIds = mentionIds,
            has = parsed.values("has") + parsed.values("exclude").map { "-$it" } + tabHasFilters(),
            channelIds = parsed.values("in").map { resolveChannelValue(it) },
            authorTypes = parsed.values("authortype"),
            sortOrder = parsed.values("sort").firstOrNull()?.let { if (it.equals("old", ignoreCase = true)) "asc" else "desc" },
            minId = minId,
            maxId = maxId,
        )
    }

    private suspend fun fetchPage(
        search: ResolvedSearch,
        selectedGuild: Guild?,
        selectedChannel: Channel?,
        offset: Int
    ): SearchResponse? {
        return if (selectedGuild != null) {
            messageApi.searchGuildMessages(
                guildId = selectedGuild.id,
                content = search.content,
                authorIds = search.authorIds,
                mentions = search.mentionIds,
                has = search.has,
                channelIds = search.channelIds,
                minId = search.minId,
                maxId = search.maxId,
                sortOrder = search.sortOrder,
                authorTypes = search.authorTypes,
                offset = offset
            )
        } else if (selectedChannel != null) {
            messageApi.searchChannelMessages(
                channelId = selectedChannel.id,
                content = search.content,
                authorIds = search.authorIds,
                mentions = search.mentionIds,
                has = search.has,
                minId = search.minId,
                maxId = search.maxId,
                sortOrder = search.sortOrder,
                authorTypes = search.authorTypes,
                offset = offset
            )
        } else null
    }

    private suspend fun fetchBatch(
        search: ResolvedSearch,
        selectedGuild: Guild?,
        selectedChannel: Channel?,
        startOffset: Int
    ): List<SearchResponse?> = coroutineScope {
        val first = async { fetchPage(search, selectedGuild, selectedChannel, startOffset) }
        val second = async { fetchPage(search, selectedGuild, selectedChannel, startOffset + 25) }
        val firstResponse = first.await()
        if (firstResponse == null || firstResponse.messages.isEmpty()) {
            second.cancel()
            return@coroutineScope listOf(firstResponse)
        }
        listOf(firstResponse, second.await())
    }

    private fun appendBatch(responses: List<SearchResponse?>) {
        for (response in responses) {
            if (response == null) break
            appendPage(response)
        }
    }

    private fun appendPage(response: SearchResponse) {
        if (response.messages.isEmpty()) {
            apiTotal = searchOffset
            lastPageFull = false
            return
        }
        val hits = response.messages.flatMap { group -> group.filter { it.hit } }
        searchResults.addAll(hits)
        searchOffset += 25
        apiTotal = response.total_results
        lastPageFull = response.messages.size >= 25
        totalSearchResults = response.total_results
    }

    private fun matchesName(member: Member, name: String): Boolean =
        member.user?.username?.equals(name, ignoreCase = true) == true ||
            member.nick?.equals(name, ignoreCase = true) == true ||
            member.user?.global_name?.equals(name, ignoreCase = true) == true

    private suspend fun resolveUserValue(value: String, selectedGuild: Guild?, selectedChannel: Channel?): String? {
        if (value.toLongOrNull() != null) return value
        val username = value.removePrefix("@")
        val cached = if (selectedGuild != null) {
            memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.find { matchesName(it, username) }
        } else if (selectedChannel != null && selectedChannel.guild_id == null) {
            val dmUsers = mutableListOf<User>()
            selectedChannel.recipients?.let { dmUsers.addAll(it) }
            if (selectedChannel.recipients.isNullOrEmpty() && !selectedChannel.recipient_ids.isNullOrEmpty()) {
                selectedChannel.recipient_ids.forEach { id ->
                    userStore.getUser(id)?.let { dmUsers.add(it) }
                }
            }
            userStore.currentUser.value?.let { dmUsers.add(it) }
            dmUsers.distinctBy { it.id }.map { Member(user = it) }.find { matchesName(it, username) }
        } else null

        cached?.user?.id?.let { return it }
        if (selectedGuild == null) return null
        guildApi.searchGuildMembers(selectedGuild.id, username, limit = 1000)
            .firstOrNull { matchesName(it, username) }?.user?.id?.let { return it }
        return guildApi.getGuildMembers(selectedGuild.id, limit = 1000)
            .firstOrNull { matchesName(it, username) }?.user?.id
    }

    private fun resolveChannelValue(value: String): String {
        if (value.toLongOrNull() != null) return value
        val name = value.removePrefix("#")
        return guildStore.allGuildChannels.value.values
            .find { it.name?.equals(name, ignoreCase = true) == true }?.id ?: value
    }

    fun clear() {
        generation++
        unresolvedUser = null
        searchResults.clear()
        searchQuery = ""
        isSearchVisible = false
        searchOffset = 0
        apiTotal = 0
        lastPageFull = false
    }
}
