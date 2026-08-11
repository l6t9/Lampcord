package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuickSwitcher(
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    gatewayManager: GatewayManager = koinInject()
) {
    val results by finderStore.results.collectAsState()
    
    LaunchedEffect(Unit) {
        finderStore.searchQuery = ""
    }

    Dialog(
        onDismissRequest = onDismiss,
        properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Surface(
            modifier = Modifier
                .widthIn(max = 600.dp)
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
                    Column {
                        TextField(
                            value = finderStore.searchQuery,
                            onValueChange = { finderStore.searchQuery = it },
                            modifier = Modifier.fillMaxWidth(),
                            placeholder = { Text("Where would you like to go?") },
                            colors = TextFieldDefaults.colors(
                                focusedContainerColor = Color.Transparent,
                                unfocusedContainerColor = Color.Transparent,
                                focusedIndicatorColor = Color.Transparent,
                                unfocusedIndicatorColor = Color.Transparent
                            ),
                            singleLine = true
                        )

                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                        LazyColumn(
                            modifier = Modifier.weight(1f),
                            contentPadding = PaddingValues(8.dp),
                            verticalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            items(results) { result ->
                                FinderResultItem(result, onClick = {
                                    when (result) {
                                        is FinderResult.Guild -> navigationStore.selectGuild(result.guild) { gatewayManager.sendSubscription(it) }
                                        is FinderResult.Channel -> navigationStore.selectChannel(result.channel)
                                        is FinderResult.DirectMessage -> navigationStore.selectChannel(result.channel)
                                    }
                                    onDismiss()
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
        }
    }
}

@Composable
private fun FinderResultItem(result: FinderResult, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
        shape = RoundedCornerShape(8.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(8.dp),
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
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(Modifier.width(12.dp))
            
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
