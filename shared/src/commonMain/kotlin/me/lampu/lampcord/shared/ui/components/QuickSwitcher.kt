package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.FinderStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuickSwitcher(
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    entityStore: EntityStore = koinInject()
) {
    val results by finderStore.results.collectAsState()
    val query by finderStore.searchQueryFlow.collectAsState()

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
                            value = query,
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
                                FinderResultRow(result, onClick = {
                                    when (result) {
                                        is FinderResult.Guild -> navigationStore.selectGuild(result.guild) { gatewayManager.sendSubscription(it) }
                                        is FinderResult.Channel -> navigationStore.selectChannel(result.channel)
                                        is FinderResult.DirectMessage -> navigationStore.selectChannel(result.channel)
                                        is FinderResult.UserResult -> result.dmChannelId?.let { channelId ->
                                            entityStore.channels.value[channelId]?.let { channel ->
                                                navigationStore.selectChannel(channel)
                                            }
                                        }
                                    }
                                    onDismiss()
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
        }
    }
}

