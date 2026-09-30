package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.input.nestedscroll.nestedScroll
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.FinderStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GlobalSearchScreen(
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    entityStore: EntityStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    val results by finderStore.results.collectAsState()
    val query by finderStore.searchQueryFlow.collectAsState()
    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()

    Scaffold(
        modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        topBar = {
            LargeTopAppBar(
                title = { Text("Search") },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                scrollBehavior = scrollBehavior
            )
        }
    ) { padding ->
        Column(modifier = Modifier.fillMaxSize().padding(padding)) {
            TextField(
                value = query,
                onValueChange = { finderStore.searchQuery = it },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                placeholder = { Text("Where would you like to go?") },
                leadingIcon = { Icon(Icons.Filled.Search, null) },
                shape = RoundedCornerShape(24.dp),
                colors = TextFieldDefaults.colors(
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                    disabledIndicatorColor = Color.Transparent
                ),
                singleLine = true
            )
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(results) { result ->
                    FinderResultRow(result, onClick = {
                        when (result) {
                            is FinderResult.Guild -> navigationStore.selectGuild(result.guild) { gatewayManager.sendSubscription(it) }
                            is FinderResult.Channel -> navigationStore.selectChannel(result.channel, explicitlySelected = true)
                            is FinderResult.DirectMessage -> navigationStore.selectChannel(result.channel, explicitlySelected = true)
                            is FinderResult.UserResult -> result.dmChannelId?.let { channelId ->
                                entityStore.channels.value[channelId]?.let { channel ->
                                    navigationStore.selectChannel(channel, explicitlySelected = true)
                                }
                            } ?: profileStore.showProfile(result.user.id, navigationStore.selectedGuild?.id)
                        }
                    })
                }

                if (results.isEmpty() && query.isNotBlank()) {
                    item {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .padding(32.dp), contentAlignment = Alignment.Center
                        ) {
                            Text(
                                "No results found",
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}

