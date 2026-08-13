package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.ui.components.AutocompletePicker
import org.koin.compose.koinInject

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
                    .widthIn(max = 1200.dp)
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
    val searchOptions = remember {
        listOf(
            SearchOption("from", Icons.Filled.Person, "user"),
            SearchOption("mentions", Icons.Rounded.AlternateEmail, "user"),
            SearchOption("has", Icons.Filled.Link, "link, embed or file"),
            SearchOption("in", Icons.Filled.Tag, "channel"),
            SearchOption("sort", Icons.AutoMirrored.Filled.List, "old"),
            SearchOption("authorType", Icons.Filled.Person, "user, bot or webhook"),
            SearchOption("exclude", Icons.Filled.Remove, "link, embed or file"),
            SearchOption("before", Icons.Filled.History, "specific date"),
            SearchOption("during", Icons.Filled.History, "specific date"),
            SearchOption("after", Icons.Filled.History, "specific date")
        )
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(
                title = {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(40.dp)
                            .background(
                                MaterialTheme.colorScheme.surfaceContainerHigh,
                                RoundedCornerShape(20.dp)
                            )
                            .padding(horizontal = 12.dp),
                        contentAlignment = Alignment.CenterStart
                    ) {
                        if (searchStore.searchQuery.isEmpty()) {
                            val targetName = navigationStore.selectedGuild?.name ?: navigationStore.selectedChannel?.name ?: "Discord"
                            Text(
                                "Search in $targetName",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                            )
                        }
                        BasicTextField(
                            value = searchStore.searchQuery,
                            onValueChange = {
                                searchStore.searchQuery = it
                                if (it.isNotBlank()) {
                                    val lastPart = it.split(" ").last()
                                    val (type, query) = when {
                                        lastPart.startsWith("from:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(5)
                                        lastPart.startsWith("mentions:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(9)
                                        lastPart.startsWith("in:", ignoreCase = true) -> AutocompleteType.CHANNEL to lastPart.substring(3)
                                        lastPart.startsWith("@") -> AutocompleteType.MENTION to lastPart.substring(1)
                                        lastPart.startsWith("#") -> AutocompleteType.CHANNEL to lastPart.substring(1)
                                        else -> null to ""
                                    }
                                    autocompleteStore.updateAutocomplete(type, query, navigationStore.selectedGuild)
                                } else {
                                    autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                            textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                            keyboardActions = KeyboardActions(onSearch = {
                                searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                            })
                        )
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    if (searchStore.searchQuery.isNotEmpty()) {
                        IconButton(onClick = {
                            searchStore.searchQuery = ""
                            searchStore.searchResults.clear()
                            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                        }) {
                            Icon(Icons.Filled.Close, contentDescription = "Clear")
                        }
                    }
                }
            )

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
                            items(searchOptions) { option ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .clickable {
                                            searchStore.searchQuery += "${option.key}:"
                                            val type = when (option.key) {
                                                "from", "mentions" -> AutocompleteType.USER
                                                "in" -> AutocompleteType.CHANNEL
                                                else -> null
                                            }
                                            autocompleteStore.updateAutocomplete(type, "", navigationStore.selectedGuild)
                                        }
                                        .padding(horizontal = 16.dp, vertical = 12.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Icon(
                                        imageVector = option.icon,
                                        contentDescription = null,
                                        modifier = Modifier.size(24.dp),
                                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                    Spacer(Modifier.width(24.dp))
                                    Column {
                                        Text(
                                            text = buildString {
                                                append(option.key)
                                                append(": ")
                                                append(option.valueHint)
                                            },
                                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold)
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
                                            .clickable {
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
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(
                                bottom = 80.dp + WindowInsets.ime.union(WindowInsets.navigationBars).asPaddingValues().calculateBottomPadding()
                            )
                        ) {
                            items(searchStore.searchResults) { message ->
                                val channel = guildStore.allGuildChannels.value[message.channel_id] ?: guildStore.privateChannels.value.find { it.id == message.channel_id }
                                val guild = guildStore.guilds.value.firstOrNull { it.id == message.guild_id }

                                Column(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 4.dp)
                                        .clickable {
                                            if (guild != null) {
                                                navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                                            }
                                            if (channel != null) {
                                                navigationStore.selectChannel(channel, explicitlySelected = true)
                                                messageStore.scrollToMessageId = message.id
                                                onDismiss()
                                            }
                                        }
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
                                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f)
                                    ) {
                                        MessageItem(message = message)
                                    }
                                }
                            }
                        }
                    }
                }

                // Autocomplete Overlay
                if (autocompleteStore.autocompleteType != null) {
                    AutocompletePicker(
                        type = autocompleteStore.autocompleteType!!,
                        query = autocompleteStore.autocompleteQuery,
                        selectedIndex = autocompleteStore.autocompleteSelectedIndex,
                        modifier = Modifier.align(Alignment.TopCenter).fillMaxWidth().padding(horizontal = 8.dp),
                        onItemSelected = { item ->
                            val currentQuery = searchStore.searchQuery
                            val parts = currentQuery.split(" ").toMutableList()
                            if (parts.isNotEmpty()) {
                                val lastPart = parts.last()
                                val prefix = if (lastPart.contains(":")) lastPart.substringBefore(":") + ":" else ""
                                val replacement = item.searchReplacement ?: item.replacement
                                parts[parts.lastIndex] = prefix + replacement
                                searchStore.searchQuery = parts.joinToString(" ") + " "
                            }
                            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                        }
                    )
                }
            }
        }

        if (getPlatformName() == "android" && searchStore.searchQuery.isNotBlank() && !searchStore.isSearchLoading) {
            ExtendedFloatingActionButton(
                onClick = { searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel) },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp)
                    .navigationBarsPadding()
                    .imePadding(),
                icon = { Icon(Icons.Filled.Search, null) },
                text = { Text("Search") },
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary
            )
        }
    }
}

data class SearchOption(
    val key: String,
    val icon: ImageVector,
    val valueHint: String
)
