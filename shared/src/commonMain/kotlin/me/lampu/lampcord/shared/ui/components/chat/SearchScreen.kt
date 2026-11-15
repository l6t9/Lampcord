package me.lampu.lampcord.shared.ui.components.chat

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back
import me.lampu.lampcord.shared.generated.resources.search
import me.lampu.lampcord.shared.generated.resources.search_panel_in
import me.lampu.lampcord.shared.generated.resources.search_panel_fallback
import me.lampu.lampcord.shared.generated.resources.search_panel_options
import me.lampu.lampcord.shared.generated.resources.search_panel_history
import me.lampu.lampcord.shared.generated.resources.search_panel_no_results
import me.lampu.lampcord.shared.generated.resources.search_panel_failed
import me.lampu.lampcord.shared.generated.resources.search_panel_no_user
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
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Member
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberRowEnv
import me.lampu.lampcord.shared.ui.LocalCurrentUser
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.kit.clickableCursor

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
    userStore: UserStore = koinInject()
) {
    val searchInput = remember {
        val scoped = withChannelScope(
            searchStore.searchQuery,
            navigationStore.selectedChannel?.takeIf { navigationStore.selectedGuild != null }?.name
        )
        SearchInputState(
            autocompleteStore,
            navigationStore,
            onQueryChange = { searchStore.searchQuery = it },
            onFilterCommitted = {
                val q = searchStore.searchQuery
                if (q.isBlank()) searchStore.searchResults.clear()
                else searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
            }
        ).apply { setQuery(scoped) }
    }

    val resultsListState = rememberLazyListState()

    LaunchedEffect(Unit) {
        val q = searchInput.toQuery()
        if (q != searchStore.searchQuery) searchStore.searchQuery = q
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

            val searchContextLabel = when {
                navigationStore.selectedGuild != null -> navigationStore.selectedGuild?.name ?: navigationStore.selectedChannel?.name
                navigationStore.selectedChannel != null -> dmSearchLabel(navigationStore.selectedChannel, userStore)
                else -> null
            } ?: stringResource(Res.string.search_panel_fallback)
            SearchInputBar(
                state = searchInput,
                externalQuery = searchStore.searchQuery,
                placeholder = "${stringResource(Res.string.search_panel_in)} $searchContextLabel",
                onSearch = {
                    searchStore.searchQuery = searchInput.finalQuery()
                    searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
                trailing = { SearchFilterActions(searchInput) }
            )

            LaunchedEffect(Unit) { if (searchStore.activeTab != "messages") searchStore.activeTab = "messages" }

            val suggestions = searchInput.suggestions()
            if (suggestions.isNotEmpty()) {
                SearchSuggestionList(suggestions = suggestions, modifier = Modifier.weight(1f))
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
                                            stringResource(Res.string.search_panel_history),
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
                                val isDmSearch = navigationStore.selectedGuild == null

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

@Composable
private fun SearchMemberList(query: String, onPick: (String) -> Unit) {
    val memberListStore: MemberListStore = koinInject()
    val presenceStore: PresenceStore = koinInject()
    val relationshipStore: RelationshipStore = koinInject()
    val settingsStore: SettingsStore = koinInject()
    val navigationStore: NavigationStore = koinInject()
    val relationshipTypes = relationshipStore.relationshipTypes.collectAsState()
    val currentUser = LocalCurrentUser.current
    val userSettings = settingsStore.userSettings
    val isTouch = remember { getPlatformName().let { it == "android" || it == "ios" } }
    val reduceMotion = Settings.shared.reduceMotion
    val env = remember(navigationStore.selectedGuild, currentUser?.id, userSettings?.status, userSettings?.developer_mode, reduceMotion) {
        MemberRowEnv(
            guild = navigationStore.selectedGuild,
            currentUserId = currentUser?.id,
            currentUserStatus = userSettings?.status,
            developerMode = userSettings?.developer_mode == true,
            presenceStore = presenceStore,
            relationshipTypes = relationshipTypes,
            animate = !isTouch && !reduceMotion,
            isTouch = isTouch,
            loadImages = true
        )
    }
    val guildApi: GuildApi = koinInject()
    val guildId = navigationStore.selectedGuild?.id
    var serverMembers by remember { mutableStateOf<List<Member>>(emptyList()) }
    LaunchedEffect(guildId, query) {
        if (guildId == null) return@LaunchedEffect
        delay(250)
        serverMembers = guildApi.searchGuildMembers(guildId, query, limit = 100)
    }
    val filtered by remember(query) {
        derivedStateOf {
            val cached = memberListStore.memberListItems.mapNotNull { it?.member }.filter { matchesMemberQuery(it, query) }
            (serverMembers + cached).distinctBy { it.user?.id }
        }
    }
    LazyColumn(modifier = Modifier.fillMaxSize()) {
        items(filtered) { member ->
            MemberItem(member, env)
        }
    }
}

private fun matchesMemberQuery(member: Member, query: String): Boolean {
    if (query.isBlank()) return true
    val username = member.user?.username.orEmpty()
    val display = member.nick ?: member.user?.global_name ?: username
    return display.contains(query, ignoreCase = true) || username.contains(query, ignoreCase = true)
}
