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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QuickSwitcher(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var query by remember { mutableStateOf("") }
    
    val results by remember(query, chatState.guilds, chatState.channels) {
        derivedStateOf {
            if (query.isBlank()) {
                // TODO: Recently visited
                emptyList<SwitcherResult>()
            } else {
                val q = query.lowercase()
                val guildMatches = chatState.guilds.filter { it.name?.lowercase()?.contains(q) == true }
                    .map { SwitcherResult.GuildResult(it) }
                
                // This only searches channels of the CURRENT guild. 
                // A better implementation would search ALL cached channels.
                val channelMatches = chatState.channels.filter { it.name?.lowercase()?.contains(q) == true }
                    .map { SwitcherResult.ChannelResult(it, chatState.selectedGuild) }
                
                guildMatches + channelMatches
            }
        }
    }

    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = Modifier
                .width(500.dp)
                .heightIn(max = 400.dp),
            shape = RoundedCornerShape(8.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 6.dp
        ) {
            Column {
                TextField(
                    value = query,
                    onValueChange = { query = it },
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
                
                HorizontalDivider()
                
                LazyColumn(
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(8.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    items(results) { result ->
                        SwitcherResultItem(result) {
                            when (result) {
                                is SwitcherResult.GuildResult -> chatState.selectGuild(result.guild)
                                is SwitcherResult.ChannelResult -> chatState.selectChannel(result.channel)
                            }
                            onDismiss()
                        }
                    }
                    
                    if (results.isEmpty() && query.isNotBlank()) {
                        item {
                            Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                                Text("No results found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SwitcherResultItem(result: SwitcherResult, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
        shape = RoundedCornerShape(4.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val icon = when (result) {
                is SwitcherResult.GuildResult -> Icons.Filled.Dns
                is SwitcherResult.ChannelResult -> Icons.Filled.Tag
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
                    is SwitcherResult.GuildResult -> result.guild.name ?: "Unnamed Guild"
                    is SwitcherResult.ChannelResult -> result.channel.name ?: "unnamed-channel"
                }
                Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                
                val subtitle = when (result) {
                    is SwitcherResult.GuildResult -> "Server"
                    is SwitcherResult.ChannelResult -> result.guild?.name ?: "Channel"
                }
                Text(subtitle, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

sealed class SwitcherResult {
    data class GuildResult(val guild: Guild) : SwitcherResult()
    data class ChannelResult(val channel: Channel, val guild: Guild?) : SwitcherResult()
}
