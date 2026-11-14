package me.lampu.lampcord.shared.ui.components.chat

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_clear
import me.lampu.lampcord.shared.generated.resources.search
import me.lampu.lampcord.shared.generated.resources.search_filter_menu_title
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.graphics.Color
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

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
    messageStore: MessageStore = koinInject()
) {
    val searchInput = remember {
        val scoped = withChannelScope(
            searchStore.searchQuery,
            navigationStore.selectedChannel?.takeIf { navigationStore.selectedGuild != null }?.name
        )
        searchStore.searchQuery = scoped
        SearchInputState(
            autocompleteStore,
            navigationStore,
            onQueryChange = { searchStore.searchQuery = it },
            onFilterCommitted = {
                if (searchStore.searchQuery.isBlank()) searchStore.searchResults.clear()
                else searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
            }
        ).apply { setQuery(scoped) }
    }

    val resultsListState = rememberLazyListState()

    LaunchedEffect(Unit) {
        if (searchStore.searchQuery.isNotBlank() && searchStore.searchResults.isEmpty()) {
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
            TopAppBar(
                title = { Text(stringResource(Res.string.search)) },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(Res.string.back))
                    }
                },
            )

            SearchInputBar(
                state = searchInput,
                externalQuery = searchStore.searchQuery,
                placeholder = "Search in ${navigationStore.selectedGuild?.name ?: navigationStore.selectedChannel?.name ?: "Discord"}",
                onSearch = {
                    searchStore.searchQuery = searchInput.finalQuery()
                    searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
                trailing = { SearchFilterActions(searchInput) }
            )

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(rememberScrollState())
                    .padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                listOf("messages" to "Messages", "media" to "Media", "links" to "Links", "files" to "Files").forEach { (id, label) ->
                    val selected = searchStore.activeTab == id
                    Text(
                        text = label,
                        style = MaterialTheme.typography.labelLarge,
                        color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .background(if (selected) MaterialTheme.colorScheme.primaryContainer else Color.Transparent)
                            .clickableCursor {
                                searchStore.activeTab = id
                                if (searchStore.searchQuery.isNotBlank()) {
                                    searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                                }
                            }
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                    )
                }
            }

            if (searchInput.suggestions().isNotEmpty()) {
                SearchSuggestionList(state = searchInput, modifier = Modifier.weight(1f))
            } else {
            Box(modifier = Modifier.weight(1f)) {
                if (searchStore.isSearchLoading) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        ContainedLoadingIndicator()
                    }
                } else if (searchStore.searchResults.isEmpty()) {
                    LazyColumn(modifier = Modifier.fillMaxSize()) {
                        if (searchStore.searchQuery.isEmpty()) {
                            item {
                                Text(
                                    "Search Options",
                                    style = MaterialTheme.typography.titleSmall,
                                    modifier = Modifier.padding(16.dp),
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                            items(SEARCH_FILTER_DEFS) { def ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .clickableCursor {
                                            searchInput.startFilter(def.key)
                                        }
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
                                            "History",
                                            style = MaterialTheme.typography.titleSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                        IconButton(onClick = {
                                            searchStore.searchHistory.clear()
                                            me.lampu.lampcord.shared.settings.Settings.shared.searchHistoryJson = "[]"
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
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    Text("No results found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                        }
                    }
                } else {
                    Column {
                        Text(
                            "${searchStore.totalSearchResults} Results",
                            modifier = Modifier.padding(16.dp),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

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
                                    Row(
                                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Text(
                                            text = channel?.name ?: "unknown-channel",
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
