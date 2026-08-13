package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.settings.Settings

class SearchStore(
    private val messageApi: MessageApi,
    private val memberListStore: MemberListStore,
    private val guildStore: GuildStore,
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
        // Load history from settings
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
        
        // Save to history
        if (!searchHistory.contains(query)) {
            searchHistory.add(0, query)
            if (searchHistory.size > 20) searchHistory.removeAt(searchHistory.lastIndex)
            Settings.shared.searchHistoryJson = json.encodeToString(searchHistory.toList())
        }

        isSearchLoading = true
        searchResults.clear()
        
        scope.launch {
            // Parse filters
            val filters = mutableMapOf<String, String>()
            val contentWords = mutableListOf<String>()
            
            val parts = query.split(" ")
            for (part in parts) {
                if (part.contains(":")) {
                    val split = part.split(":", limit = 2)
                    val key = split[0].lowercase()
                    val value = split[1]
                    if (value.isNotEmpty()) {
                        val finalValue = when(key) {
                            "from", "mentions" -> {
                                // Try to resolve username to ID if it's not already an ID
                                if (value.toLongOrNull() == null) {
                                    val username = if (value.startsWith("@")) value.substring(1) else value
                                    val member = memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.find { 
                                        it.user?.username?.equals(username, ignoreCase = true) == true || 
                                        it.nick?.equals(username, ignoreCase = true) == true ||
                                        it.user?.global_name?.equals(username, ignoreCase = true) == true
                                    }
                                    member?.user?.id ?: value
                                } else value
                            }
                            "in" -> {
                                if (value.toLongOrNull() == null) {
                                    val name = if (value.startsWith("#")) value.substring(1) else value
                                    val channel = guildStore.allGuildChannels.value.values.find { it.name?.equals(name, ignoreCase = true) == true }
                                    channel?.id ?: value
                                } else value
                            }
                            else -> value
                        }
                        filters[key] = finalValue
                        continue
                    }
                }
                contentWords.add(part)
            }
            
            val content = contentWords.joinToString(" ").takeIf { it.isNotBlank() }
            
            val response = if (selectedGuild != null) {
                messageApi.searchGuildMessages(
                    guildId = selectedGuild.id,
                    content = content,
                    authorId = filters["from"],
                    mentions = filters["mentions"],
                    has = filters["has"],
                    channelId = filters["in"],
                    before = filters["before"],
                    after = filters["after"],
                    during = filters["during"],
                    sort = filters["sort"],
                    authorType = filters["authortype"]
                )
            } else if (selectedChannel != null) {
                messageApi.searchChannelMessages(
                    channelId = selectedChannel.id,
                    content = content,
                    authorId = filters["from"],
                    mentions = filters["mentions"],
                    has = filters["has"],
                    before = filters["before"],
                    after = filters["after"],
                    during = filters["during"],
                    sort = filters["sort"],
                    authorType = filters["authortype"]
                )
            } else null
            
            response?.let {
                totalSearchResults = it.total_results
                val hits = it.messages.flatMap { group -> 
                    group.filter { message -> message.hit }
                }
                searchResults.addAll(hits)
            }
            isSearchLoading = false
        }
    }

    fun clear() {
        searchResults.clear()
        searchQuery = ""
        isSearchVisible = false
    }
}
