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
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.FinderStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GlobalSearchScreen(
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    gatewayManager: GatewayManager = koinInject()
) {
    val results by finderStore.results.collectAsState()
    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()

    Scaffold(
        modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
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
                value = finderStore.searchQuery,
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
                    FinderResultItem(result, onClick = {
                        when (result) {
                            is FinderResult.Guild -> navigationStore.selectGuild(result.guild) { gatewayManager.sendSubscription(it) }
                            is FinderResult.Channel -> navigationStore.selectChannel(result.channel, explicitlySelected = true)
                            is FinderResult.DirectMessage -> navigationStore.selectChannel(result.channel, explicitlySelected = true)
                        }
                    })
                }

                if (results.isEmpty() && finderStore.searchQuery.isNotBlank()) {
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

@Composable
private fun FinderResultItem(result: FinderResult, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val icon = when (result) {
                is FinderResult.Guild -> Icons.Filled.Dns
                is FinderResult.Channel -> if (result.channel.type == 2 || result.channel.type == 13) Icons.Filled.VolumeUp else if (result.channel.type == 5) Icons.Filled.Campaign else Icons.Filled.Tag
                is FinderResult.DirectMessage -> Icons.Filled.Person
            }
            
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(Modifier.width(16.dp))
            
            Column {
                val title = when (result) {
                    is FinderResult.Guild -> result.guild.name ?: "Unnamed Guild"
                    is FinderResult.Channel -> result.channel.name ?: "unnamed-channel"
                    is FinderResult.DirectMessage -> result.channel.recipients?.firstOrNull()?.let { it.global_name ?: it.username } ?: "Direct Message"
                }
                Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                
                val subtitle = when (result) {
                    is FinderResult.Guild -> "Server"
                    is FinderResult.Channel -> result.guild?.name ?: "Channel"
                    is FinderResult.DirectMessage -> "Direct Message"
                }
                Text(subtitle, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
