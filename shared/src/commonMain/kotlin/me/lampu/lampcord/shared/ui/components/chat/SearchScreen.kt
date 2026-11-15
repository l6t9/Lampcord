package me.lampu.lampcord.shared.ui.components.chat

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back
import me.lampu.lampcord.shared.generated.resources.messages
import me.lampu.lampcord.shared.generated.resources.search
import me.lampu.lampcord.shared.generated.resources.search_panel_fallback
import me.lampu.lampcord.shared.generated.resources.search_panel_files
import me.lampu.lampcord.shared.generated.resources.search_panel_history
import me.lampu.lampcord.shared.generated.resources.search_panel_in
import me.lampu.lampcord.shared.generated.resources.search_panel_links
import me.lampu.lampcord.shared.generated.resources.search_panel_media
import me.lampu.lampcord.shared.generated.resources.search_panel_no_results
import me.lampu.lampcord.shared.generated.resources.search_panel_failed
import me.lampu.lampcord.shared.generated.resources.search_panel_no_user
import me.lampu.lampcord.shared.generated.resources.search_panel_options
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.LocalCurrentUser
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberRowEnv
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.utils.CleanUtils
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.generated.resources.pins

private data class SearchTab(val id: String, val label: String)

private val SEARCH_TABS = listOf(
    SearchTab("messages", "Messages"),
    SearchTab("media", "Media"),
    SearchTab("pins", "Pins"),
    SearchTab("links", "Links"),
    SearchTab("files", "Files"),
)

@Composable
private fun searchTabLabel(tab: SearchTab): String = when (tab.id) {
    "messages" -> stringResource(Res.string.messages)
    "media" -> stringResource(Res.string.search_panel_media)
    "pins" -> stringResource(Res.string.pins)
    "links" -> stringResource(Res.string.search_panel_links)
    "files" -> stringResource(Res.string.search_panel_files)
    else -> tab.label
}

private fun dmSearchLabel(channel: me.lampu.lampcord.shared.model.Channel?, userStore: UserStore): String? {
    if (channel == null) return null
    if (!channel.name.isNullOrBlank() && channel.type != 1) return channel.name
    val recipients = channel.recipients.orEmpty().ifEmpty {
        channel.recipient_ids.orEmpty().mapNotNull { userStore.getUser(it) }
    }
    val isGroup = channel.type == 3 || recipients.size > 1
    return if (isGroup) {
        channel.name?.takeIf { it.isNotBlank() } ?: "Group DM"
    } else {
        val other = recipients.firstOrNull { it.id != userStore.currentUser.value?.id } ?: recipients.firstOrNull()
        val name = other?.global_name ?: other?.username
        if (name != null) "@$name" else "Direct Message"
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchScreen(
    onDismiss: () -> Unit
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
    if (isMobile) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surface
        ) {
            SearchScreenContent(onDismiss)
        }
    } else {
        androidx.compose.ui.window.Dialog(
            onDismissRequest = onDismiss,
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
        ) {
            Surface(
                modifier = Modifier
                    .widthIn(max = 1400.dp)
                    .fillMaxWidth(0.95f)
                    .heightIn(max = 850.dp)
                    .fillMaxHeight(0.9f)
                    .clip(MaterialTheme.shapes.large),
                shape = MaterialTheme.shapes.large,
                color = MaterialTheme.colorScheme.surfaceContainerLowest,
            ) {
                Column {
                    Surface(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f)
                            .padding(8.dp),
                        shape = MaterialTheme.shapes.medium,
                        color = MaterialTheme.colorScheme.surfaceContainer
                    ) {
                        SearchScreenContent(onDismiss)
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SearchScreenContent(
    onDismiss: () -> Unit,
    searchStore: SearchStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    autocompleteStore: AutocompleteStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    messageStore: MessageStore = koinInject(),
    userStore: UserStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
) {
    val searchInput = remember {
        val scoped = withChannelScope(
            searchStore.searchQuery,
            navigationStore.selectedChannel?.takeIf { navigationStore.selectedGuild != null }?.id
        )
        SearchInputState(
            autocompleteStore,
            navigationStore,
            onQueryChange = { searchStore.searchQuery = it },
            onFilterCommitted = {
                val q = searchStore.searchQuery
                if (q.isBlank() && searchStore.activeTab == "messages") searchStore.searchResults.clear()
                else searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
            },
            chipDisplay = { key, value ->
                if (key == "in") {
                    val ch = guildStore.allGuildChannels.value[value]
                        ?: guildStore.privateChannels.value.find { it.id == value }
                    ch?.name?.let { CleanUtils.cleanChannelName(it) } ?: value
                } else value
            }
        ).apply { setQuery(scoped) }
    }

    val resultsListState = rememberLazyListState()

    LaunchedEffect(Unit) {
        if (searchStore.activeTab !in SEARCH_TABS.map { it.id }) searchStore.activeTab = "messages"
        val q = searchInput.toQuery()
        if (q != searchStore.searchQuery) searchStore.searchQuery = q
        if ((searchStore.searchQuery.isNotBlank() || searchStore.activeTab != "messages") && searchStore.searchResults.isEmpty()) {
            searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
        }
    }

    LaunchedEffect(resultsListState) {
        snapshotFlow {
            val lastVisible = resultsListState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1
            lastVisible to searchStore.searchResults.size
        }.collect { (lastVisible, size) ->
            if (size > 0 && lastVisible >= size - 5) {
                searchStore.loadMoreResults(navigationStore.selectedGuild, navigationStore.selectedChannel)
            }
        }
    }

    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current

    DisposableEffect(Unit) {
        onDispose {
            searchStore.searchQuery = ""
            searchStore.searchResults.clear()
            searchStore.activeTab = "messages"
            autocompleteStore.clear(isSearch = true)
        }
    }

    PlatformBackHandler(enabled = true) {
        if (searchInput.fieldFocused) {
            focusManager.clearFocus()
            keyboardController?.hide()
        } else {
            onDismiss()
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize().imePadding()) {
            val searchContextLabel = when {
                navigationStore.selectedGuild != null -> {
                    navigationStore.selectedChannel
                        ?.takeIf { it.guild_id != null }
                        ?.name
                        ?.let { CleanUtils.cleanChannelName(it) }
                        ?: navigationStore.selectedGuild?.name
                }
                navigationStore.selectedChannel != null -> dmSearchLabel(navigationStore.selectedChannel, userStore)
                else -> null
            } ?: stringResource(Res.string.search_panel_fallback)

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 4.dp, end = 8.dp, top = 8.dp, bottom = 4.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = onDismiss) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(Res.string.back))
                }
                SearchInputBar(
                    state = searchInput,
                    externalQuery = searchStore.searchQuery,
                    placeholder = "${stringResource(Res.string.search_panel_in)} $searchContextLabel",
                    onSearch = {
                        searchStore.searchQuery = searchInput.finalQuery()
                        searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                    },
                    modifier = Modifier.weight(1f),
                    trailing = { SearchClearButton(searchInput) }
                )
                Spacer(Modifier.width(4.dp))
                SearchFilterButton(searchInput)
            }

            val suggestions = searchInput.suggestions()
            if (suggestions.isNotEmpty()) {
                SearchSuggestionList(suggestions = suggestions, modifier = Modifier.weight(1f))
            } else {
                val hasQuery = searchStore.searchQuery.isNotBlank() || searchStore.activeTab != "messages"
                if (hasQuery) {
                    val selectedTabIndex = SEARCH_TABS.indexOfFirst { it.id == searchStore.activeTab }.coerceAtLeast(0)
                    PrimaryScrollableTabRow(
                        selectedTabIndex = selectedTabIndex,
                        edgePadding = 12.dp,
                        divider = {}
                    ) {
                        SEARCH_TABS.forEachIndexed { index, tab ->
                            Tab(
                                selected = index == selectedTabIndex,
                                onClick = {
                                    if (searchStore.activeTab != tab.id) {
                                        searchStore.activeTab = tab.id
                                        searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                                    }
                                },
                                text = { Text(searchTabLabel(tab), maxLines = 1) }
                            )
                        }
                    }
                }

                Box(modifier = Modifier.weight(1f)) {
                    if (searchStore.isSearchLoading) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            ContainedLoadingIndicator()
                        }
                    } else if (searchStore.searchResults.isEmpty()) {
                        LazyColumn(modifier = Modifier.fillMaxSize()) {
                            if (!hasQuery) {
                                item {
                                    Text(
                                        stringResource(Res.string.search_panel_options),
                                        style = MaterialTheme.typography.titleSmall,
                                        modifier = Modifier.padding(16.dp),
                                        color = MaterialTheme.colorScheme.primary
                                    )
                                }
                                items(SEARCH_FILTER_DEFS) { def ->
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clickableCursor { searchInput.startFilter(def.key) }
                                            .padding(horizontal = 16.dp, vertical = 12.dp),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Icon(
                                            imageVector = searchFilterIcon(def.key),
                                            contentDescription = null,
                                            modifier = Modifier.size(24.dp),
                                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                        Spacer(Modifier.width(24.dp))
                                        Column {
                                            Text(
                                                text = "${def.key}:",
                                                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold)
                                            )
                                            Text(
                                                text = def.label,
                                                style = MaterialTheme.typography.bodySmall,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                        }
                                    }
                                }

                                if (searchStore.searchHistory.isNotEmpty()) {
                                    item {
                                        Row(
                                            modifier = Modifier.fillMaxWidth().padding(16.dp),
                                            horizontalArrangement = Arrangement.SpaceBetween,
                                            verticalAlignment = Alignment.CenterVertically
                                        ) {
                                            Text(
                                                stringResource(Res.string.search_panel_history),
                                                style = MaterialTheme.typography.titleSmall,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                            IconButton(onClick = {
                                                searchStore.searchHistory.clear()
                                                Settings.shared.searchHistoryJson = "[]"
                                            }) {
                                                Icon(Icons.Filled.Delete, null, modifier = Modifier.size(18.dp))
                                            }
                                        }
                                    }
                                    items(searchStore.searchHistory) { query ->
                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .clickableCursor {
                                                    searchStore.searchQuery = query
                                                    searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                                                }
                                                .padding(horizontal = 16.dp, vertical = 12.dp),
                                            verticalAlignment = Alignment.CenterVertically
                                        ) {
                                            Icon(
                                                imageVector = Icons.Filled.History,
                                                contentDescription = null,
                                                modifier = Modifier.size(20.dp),
                                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                            Spacer(Modifier.width(24.dp))
                                            Text(query, style = MaterialTheme.typography.bodyMedium)
                                        }
                                    }
                                }
                            } else {
                                item {
                                    Box(
                                        modifier = Modifier.fillMaxSize().clickableCursor {
                                            if (searchStore.searchFailed) {
                                                searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                                            }
                                        },
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Text(
                                            text = searchStore.unresolvedUser?.let { stringResource(Res.string.search_panel_no_user, it) }
                                                ?: if (searchStore.searchFailed) stringResource(Res.string.search_panel_failed) else stringResource(Res.string.search_panel_no_results),
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }
                            }
                        }
                    } else {
                        Column {
                            Text(
                                "${searchStore.totalSearchResults} Results",
                                modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )

                            LazyColumn(
                                state = resultsListState,
                                modifier = Modifier.fillMaxSize(),
                                contentPadding = PaddingValues(
                                    bottom = 80.dp + WindowInsets.ime.union(WindowInsets.navigationBars).asPaddingValues().calculateBottomPadding()
                                )
                            ) {
                                items(searchStore.searchResults) { message ->
                                    val channel = guildStore.allGuildChannels.value[message.channel_id] ?: guildStore.privateChannels.value.find { it.id == message.channel_id }
                                    val guild = guildStore.guilds.value.firstOrNull { it.id == message.guild_id }
                                    val isDmSearch = navigationStore.selectedGuild == null
                                    val channelIcon = searchResultChannelIcon(
                                        channel = channel,
                                        guild = guild,
                                        memberListStore = memberListStore,
                                        currentUserId = userStore.currentUser.value?.id
                                    )

                                    val onClick = {
                                        if (guild != null) {
                                            navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                                        }
                                        navigationStore.selectChannelById(message.channel_id, explicitlySelected = true)
                                        messageStore.jumpToMessage(message.channel_id, message.id)
                                        onDismiss()
                                    }

                                    Column(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(vertical = 4.dp)
                                            .clickableCursor(onClick = onClick)
                                    ) {
                                        if (!isDmSearch) {
                                            Row(
                                                modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                                                verticalAlignment = Alignment.CenterVertically
                                            ) {
                                                Icon(
                                                    imageVector = channelIcon,
                                                    contentDescription = null,
                                                    modifier = Modifier.size(16.dp),
                                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                                Spacer(Modifier.width(6.dp))
                                                Text(
                                                    text = channel?.name?.let { CleanUtils.cleanChannelName(it) } ?: "unknown-channel",
                                                    style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                                                    color = MaterialTheme.colorScheme.onSurface
                                                )
                                                if (guild != null) {
                                                    Spacer(Modifier.width(8.dp))
                                                    Text(
                                                        text = guild.name ?: "unknown-server",
                                                        style = MaterialTheme.typography.labelSmall,
                                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                                    )
                                                }
                                            }
                                        }

                                        Surface(
                                            modifier = Modifier.padding(horizontal = 8.dp),
                                            shape = RoundedCornerShape(12.dp),
                                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
                                            onClick = onClick
                                        ) {
                                            MessageItem(
                                                message = message,
                                                isPreview = true
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

private fun searchResultChannelIcon(
    channel: Channel?,
    guild: me.lampu.lampcord.shared.model.Guild?,
    memberListStore: MemberListStore,
    currentUserId: String?,
): ImageVector {
    if (channel == null) return Icons.Filled.Tag
    val locked = if (guild != null && currentUserId != null) {
        val me = memberListStore.memberListItems.mapNotNull { it?.member }.firstOrNull { it.user?.id == currentUserId }
        if (me != null) !PermissionHelper.canViewChannel(me, guild, channel, currentUserId) else false
    } else false
    return when (channel.type) {
        2, 13 -> if (locked) Icons.Filled.Lock else Icons.AutoMirrored.Filled.VolumeUp
        5 -> if (locked) Icons.Filled.Lock else Icons.Filled.Campaign
        15 -> if (locked) Icons.Filled.Lock else Icons.Rounded.Forum
        else -> if (locked) Icons.Filled.Lock else Icons.Filled.Tag
    }
}
