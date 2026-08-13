package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ThreadPanel(
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    channelApi: ChannelApi = koinInject()
) {
    val parentChannel = navigationStore.selectedChannel ?: return
    val allChannels by guildStore.allGuildChannels.collectAsState()
    
    val activeThreads = remember(parentChannel.id, allChannels.size) {
        allChannels.values.filter { it.parent_id == parentChannel.id && it.type in listOf(10, 11, 12) && it.thread_metadata?.archived != true }
            .sortedByDescending { it.lastMessageId() ?: it.id }
    }

    var archivedThreads by remember { mutableStateOf<List<Channel>>(emptyList()) }
    var isLoadingArchived by remember { mutableStateOf(false) }
    var hasLoadedArchived by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .width(340.dp)
            .fillMaxHeight()
            .background(MaterialTheme.colorScheme.surface)
    ) {
        // Header
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            tonalElevation = 1.dp
        ) {
            Row(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(Icons.Filled.Tag, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                Spacer(Modifier.width(8.dp))
                Text(text = "Threads", style = MaterialTheme.typography.titleSmall)
                Spacer(Modifier.weight(1f))
                IconButton(onClick = { navigationStore.isThreadPanelVisible = false }) {
                    Icon(Icons.Filled.Close, "Close")
                }
            }
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (activeThreads.isNotEmpty()) {
                item {
                    Text(
                        "Active Threads",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(Modifier.height(8.dp))
                }
                items(activeThreads, key = { it.id }) { thread ->
                    ThreadPanelItem(thread) {
                        navigationStore.selectThread(thread, explicitlySelected = true)
                    }
                }
            }

            item {
                Spacer(Modifier.height(16.dp))
                if (!hasLoadedArchived) {
                    androidx.compose.material3.TextButton(
                        onClick = {
                            isLoadingArchived = true
                            scope.launch {
                                val result = channelApi.getArchivedPublicThreads(parentChannel.id)
                                archivedThreads = result?.threads ?: emptyList()
                                isLoadingArchived = false
                                hasLoadedArchived = true
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        enabled = !isLoadingArchived
                    ) {
                        if (isLoadingArchived) {
                            androidx.compose.material3.CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp)
                        } else {
                            Text("Load Archived Threads")
                        }
                    }
                } else if (archivedThreads.isNotEmpty()) {
                    Text(
                        "Archived Threads",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(Modifier.height(8.dp))
                }
            }

            if (hasLoadedArchived && archivedThreads.isNotEmpty()) {
                items(archivedThreads, key = { it.id }) { thread ->
                    ThreadPanelItem(thread) {
                        navigationStore.selectThread(thread, explicitlySelected = true)
                    }
                }
            } else if (hasLoadedArchived && archivedThreads.isEmpty() && activeThreads.isEmpty()) {
                item {
                    Box(Modifier.fillMaxWidth().padding(top = 32.dp), contentAlignment = Alignment.Center) {
                        Text("No threads found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}

@Composable
fun ThreadPanelItem(thread: Channel, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(
                text = thread.name ?: "Unnamed Thread",
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            val lastActiveId = thread.lastMessageId()
            if (lastActiveId != null) {
                Text(
                    text = "Last active ${thread.lastMessageId() ?: "unknown"}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}
