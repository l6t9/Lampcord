package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.SearchResponse
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
    var isLoadingMore by mutableStateOf(false)
    var activeTab by mutableStateOf("messages")
    var totalSearchResults by mutableStateOf(0)
    val searchHistory = mutableStateListOf<String>()

    private var activeQuery = ""
    private var searchOffset = 0
    private var apiTotal = 0
    private var lastPageFull = false

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

        activeQuery = query
        searchOffset = 0
        apiTotal = 0
        lastPageFull = false
        isSearchLoading = true
        searchResults.clear()

        scope.launch {
            val response = fetchPage(query, selectedGuild, selectedChannel, 0)
            if (response != null) appendPage(response)
            isSearchLoading = false
        }
    }

    fun loadMoreResults(selectedGuild: Guild?, selectedChannel: Channel?) {
        if (isSearchLoading || isLoadingMore || !hasMoreResults) return
        isLoadingMore = true
        val query = activeQuery
        scope.launch {
            val response = fetchPage(query, selectedGuild, selectedChannel, searchOffset)
            if (response != null) appendPage(response)
            isLoadingMore = false
        }
    }

    private suspend fun fetchPage(
        query: String,
        selectedGuild: Guild?,
        selectedChannel: Channel?,
        offset: Int
    ): SearchResponse? {
        val parsed = parseSearchQuery(query)
        val authorIds = parsed.values("from").map { resolveUserValue(it, selectedGuild, selectedChannel) }
        val mentionIds = parsed.values("mentions").map { resolveUserValue(it, selectedGuild, selectedChannel) }
        val channelIds = parsed.values("in").map { resolveChannelValue(it) }
        val has = parsed.values("has") + parsed.values("exclude").map { "-$it" } + tabHasFilters()
        val authorTypes = parsed.values("authortype")
        val sortOrder = parsed.values("sort").firstOrNull()?.let { if (it.equals("old", ignoreCase = true)) "asc" else "desc" }
        val content = parsed.text.takeIf { it.isNotBlank() }

        var minId: String? = null
        var maxId: String? = null
        parsed.tokens.filter { it.key == "before" || it.key == "after" || it.key == "during" }.forEach { token ->
            val bounds = searchDateBounds(token.key, token.value) ?: return@forEach
            bounds.first?.let { minId = it }
            bounds.second?.let { maxId = it }
        }

        return if (selectedGuild != null) {
            messageApi.searchGuildMessages(
                guildId = selectedGuild.id,
                content = content,
                authorIds = authorIds,
                mentions = mentionIds,
                has = has,
                channelIds = channelIds,
                minId = minId,
                maxId = maxId,
                sortOrder = sortOrder,
                authorTypes = authorTypes,
                offset = offset
            )
        } else if (selectedChannel != null) {
            messageApi.searchChannelMessages(
                channelId = selectedChannel.id,
                content = content,
                authorIds = authorIds,
                mentions = mentionIds,
                has = has,
                minId = minId,
                maxId = maxId,
                sortOrder = sortOrder,
                authorTypes = authorTypes,
                offset = offset
            )
        } else null
    }

    private fun tabHasFilters(): List<String> = when (activeTab) {
        "media" -> listOf("image")
        "links" -> listOf("link")
        "files" -> listOf("file")
        else -> emptyList()
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
        searchOffset = 0
        apiTotal = 0
        lastPageFull = false
    }
}
