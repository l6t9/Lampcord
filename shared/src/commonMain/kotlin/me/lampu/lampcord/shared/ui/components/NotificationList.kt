package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun NotificationList(
    readStateStore: ReadStateStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val readStates by readStateStore.readStates.collectAsState()
    val allChannels by guildStore.allGuildChannels.collectAsState()
    val privateChannels by guildStore.privateChannels.collectAsState()
    val guilds by guildStore.guilds.collectAsState()

    val unreadChannels = readStates.values
        .filter { it.mention_count > 0 }
        .mapNotNull { state ->
            val channel = allChannels[state.id]
                ?: privateChannels.firstOrNull { it.id == state.id }
            if (channel != null) channel to state.mention_count else null
        }
        .sortedByDescending { it.second }

    Column(modifier = Modifier.fillMaxSize()) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            color = MaterialTheme.colorScheme.surfaceContainer
        ) {
            Box(contentAlignment = Alignment.Center) {
                Text("Notifications", style = MaterialTheme.typography.titleSmall)
            }
        }

        if (unreadChannels.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        imageVector = Icons.Filled.Notifications,
                        contentDescription = null,
                        modifier = Modifier.size(48.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
                    )
                    Spacer(Modifier.height(8.dp))
                    Text(
                        "No new notifications",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    )
                }
            }
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(unreadChannels) { (channel, count) ->
                    val guild = guilds.firstOrNull { it.id == channel.guild_id }
                    Surface(
                        onClick = { navigationStore.selectChannel(channel) },
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 2.dp),
                        shape = MaterialTheme.shapes.small,
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
                    ) {
                        Row(
                            modifier = Modifier.padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = if (guild != null) "${guild.name}" else "Direct Messages",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.primary
                                )
                                Text(
                                    text = "#${channel.name ?: "unknown"}",
                                    style = MaterialTheme.typography.bodyLarge
                                )
                            }
                            Badge(
                                containerColor = MaterialTheme.colorScheme.error,
                                contentColor = MaterialTheme.colorScheme.onError
                            ) {
                                Text(count.toString())
                            }
                        }
                    }
                }
            }
        }
    }
}
