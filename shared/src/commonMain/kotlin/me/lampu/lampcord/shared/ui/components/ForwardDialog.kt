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
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3Api::class)
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

    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = Modifier
                .width(440.dp)
                .heightIn(max = 500.dp),
            shape = RoundedCornerShape(12.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 6.dp
        ) {
            Column {
                Text(
                    "Forward Message",
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.padding(16.dp)
                )
                
                TextField(
                    value = query,
                    onValueChange = { query = it },
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                    placeholder = { Text("Search for a channel or DM") },
                    leadingIcon = { Icon(Icons.Filled.Search, null) },
                    colors = TextFieldDefaults.colors(
                        focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                        unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                        focusedIndicatorColor = Color.Transparent,
                        unfocusedIndicatorColor = Color.Transparent
                    ),
                    shape = RoundedCornerShape(8.dp),
                    singleLine = true
                )
                
                Spacer(Modifier.height(8.dp))
                
                LazyColumn(
                    modifier = Modifier.weight(1f),
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

                Row(
                    modifier = Modifier.fillMaxWidth().padding(16.dp),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("Cancel")
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
