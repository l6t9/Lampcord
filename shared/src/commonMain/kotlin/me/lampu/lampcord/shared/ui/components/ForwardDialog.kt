package me.lampu.lampcord.shared.ui.components

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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ForwardDialog(
    message: Message,
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var query by remember { mutableStateOf("") }
    
    val results by remember(query, chatState.guilds, chatState.guildStore.allGuildChannels) {
        derivedStateOf {
            val q = query.lowercase()
            val allChannels = chatState.guilds.flatMap { guild -> 
                val channels = chatState.guildStore.allGuildChannels[guild.id] ?: emptyList()
                channels.filter { it.type in listOf(0, 5) }.map { it to guild }
            } + chatState.privateChannels.map { it to null }

            if (q.isBlank()) {
                allChannels.take(10)
            } else {
                allChannels.filter { (channel, guild) ->
                    channel.name?.lowercase()?.contains(q) == true || 
                    guild?.name?.lowercase()?.contains(q) == true ||
                    channel.recipients?.any { it.username?.lowercase()?.contains(q) == true || it.global_name?.lowercase()?.contains(q) == true } == true
                }
            }
        }
    }

    Dialog(
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
                // Header / Titlebar
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp, start = 16.dp, end = 16.dp)
                ) {
                    Text(
                        text = "Forward Message",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.align(Alignment.Center)
                    )
                    IconButton(
                        onClick = onDismiss,
                        modifier = Modifier.align(Alignment.CenterEnd)
                    ) {
                        Icon(Icons.Default.Close, "Close", tint = MaterialTheme.colorScheme.onSurface, modifier = Modifier.size(20.dp))
                    }
                }

                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .padding(8.dp),
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.surfaceContainer
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        TextField(
                            value = query,
                            onValueChange = { query = it },
                            modifier = Modifier.fillMaxWidth(),
                            placeholder = { Text("Search for a channel or DM") },
                            leadingIcon = { Icon(Icons.Filled.Search, null) },
                            colors = TextFieldDefaults.colors(
                                focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                                unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                                focusedIndicatorColor = Color.Transparent,
                                unfocusedIndicatorColor = Color.Transparent
                            ),
                            shape = MaterialTheme.shapes.medium,
                            singleLine = true
                        )

                        Spacer(Modifier.height(16.dp))

                        Surface(
                            shape = MaterialTheme.shapes.medium,
                            color = MaterialTheme.colorScheme.surfaceContainerLow,
                            modifier = Modifier.fillMaxWidth().weight(1f)
                        ) {
                            LazyColumn(
                                modifier = Modifier.fillMaxWidth(),
                                contentPadding = PaddingValues(8.dp),
                                verticalArrangement = Arrangement.spacedBy(4.dp)
                            ) {
                                items(results) { (channel, guild) ->
                                    ForwardResultItem(channel, guild) {
                                        chatState.forwardMessage(channel, message)
                                        onDismiss()
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
private fun ForwardResultItem(channel: Channel, guild: me.lampu.lampcord.shared.model.Guild?, onClick: () -> Unit) {
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
            val icon = if (guild != null) {
                if (channel.type == 5) Icons.Filled.Campaign else Icons.Filled.Tag
            } else {
                Icons.Filled.Person
            }
            
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(Modifier.width(12.dp))
            
            Column {
                val title = if (guild != null) {
                    channel.name ?: "unnamed-channel"
                } else {
                    channel.recipients?.firstOrNull()?.let { it.global_name ?: it.username } ?: "Direct Message"
                }
                Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                
                val subtitle = guild?.name ?: "Direct Message"
                Text(subtitle, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
