package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.DateTimeUtils
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ThreadPanel(
    onDismiss: () -> Unit = { },
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    channelApi: ChannelApi = koinInject()
) {
    val parentChannel = navigationStore.selectedChannel ?: return
    val allChannels by guildStore.allGuildChannels.collectAsState()

    val activeThreads = remember(parentChannel.id, allChannels) {
        allChannels.values.filter {
            it.parent_id == parentChannel.id &&
                it.type in listOf(10, 11, 12) &&
                it.thread_metadata?.archived != true
        }.sortedByDescending { it.lastMessageId() ?: it.id }
    }

    var archivedThreads by remember(parentChannel.id) { mutableStateOf<List<Channel>>(emptyList()) }
    var isLoadingArchived by remember(parentChannel.id) { mutableStateOf(false) }
    var hasLoadedArchived by remember(parentChannel.id) { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    val platform = me.lampu.lampcord.shared.utils.getPlatformName()
    val isMobile = platform == "android" || platform == "ios"

    Column(
        modifier = Modifier
            .then(if (isMobile) Modifier.fillMaxWidth() else Modifier.width(340.dp))
            .fillMaxHeight()
    ) {
        TopAppBar(
            title = { Text("Threads", style = MaterialTheme.typography.titleLarge) },
            navigationIcon = {
                IconButton(onClick = {
                    if (isMobile) onDismiss() else navigationStore.isThreadPanelVisible = false
                }) {
                    Icon(
                        imageVector = if (isMobile) Icons.AutoMirrored.Filled.ArrowBack else Icons.Filled.Close,
                        contentDescription = stringResource(Res.string.back)
                    )
                }
            },
            colors = TopAppBarDefaults.topAppBarColors(
                containerColor = MaterialTheme.colorScheme.surface
            )
        )

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(0.dp)
        ) {
            if (activeThreads.isNotEmpty()) {
                item {
                    Text(
                        "Active",
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
                    )
                }
                items(activeThreads, key = { it.id }) { thread ->
                    ThreadPanelItem(thread) {
                        navigationStore.isThreadPanelVisible = false
                        navigationStore.selectThread(thread, explicitlySelected = true)
                    }
                }
            } else if (!hasLoadedArchived) {
                item {
                    Box(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 24.dp)) {
                        Text(
                            "No active threads",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            item {
                if (!hasLoadedArchived) {
                    Box(Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                        FilledTonalButton(
                            onClick = {
                                isLoadingArchived = true
                                scope.launch {
                                    val result = channelApi.getArchivedPublicThreads(parentChannel.id)
                                    archivedThreads = result?.threads ?: emptyList()
                                    isLoadingArchived = false
                                    hasLoadedArchived = true
                                }
                            },
                            enabled = !isLoadingArchived
                        ) {
                            if (isLoadingArchived) {
                                CircularProgressIndicator(
                                    Modifier.size(16.dp),
                                    strokeWidth = 2.dp,
                                    color = MaterialTheme.colorScheme.onSecondaryContainer
                                )
                                Spacer(Modifier.width(8.dp))
                            }
                            Text(if (isLoadingArchived) "Loading" else "Show archived")
                        }
                    }
                } else if (archivedThreads.isNotEmpty()) {
                    Text(
                        "Archived",
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
                    )
                }
            }

            if (hasLoadedArchived && archivedThreads.isNotEmpty()) {
                items(archivedThreads, key = { it.id }) { thread ->
                    ThreadPanelItem(thread) {
                        navigationStore.isThreadPanelVisible = false
                        navigationStore.selectThread(thread, explicitlySelected = true)
                    }
                }
            } else if (hasLoadedArchived && archivedThreads.isEmpty() && activeThreads.isEmpty()) {
                item {
                    Box(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 24.dp)) {
                        Text(
                            "No threads found",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun ThreadPanelItem(thread: Channel, onClick: () -> Unit) {
    val isArchived = thread.thread_metadata?.archived == true
    val lastActive = remember(thread.last_message_id, thread.id) {
        val id = thread.lastMessageId() ?: thread.id
        id.toLongOrNull()?.let { snowflake ->
            val ts = (snowflake shr 22) + 1420070400000L
            DateTimeUtils.formatDiscordTimestamp(ts / 1000, "R")
        }
    }
    val supporting = buildString {
        if (isArchived) append("Archived")
        if (lastActive != null) {
            if (isNotEmpty()) append(" · ")
            append("Last active $lastActive")
        }
    }.ifEmpty { null }

    ListItem(
        headlineContent = {
            Text(
                text = thread.name ?: "Unnamed thread",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                color = if (isArchived) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface
            )
        },
        supportingContent = supporting?.let { text ->
            { Text(text = text, maxLines = 1, overflow = TextOverflow.Ellipsis) }
        },
        leadingContent = {
            Icon(
                imageVector = Icons.Rounded.Topic,
                contentDescription = null,
                tint = if (isArchived) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.primary
            )
        },
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        colors = ListItemDefaults.colors(containerColor = Color.Transparent),
        tonalElevation = 0.dp
    )
}
