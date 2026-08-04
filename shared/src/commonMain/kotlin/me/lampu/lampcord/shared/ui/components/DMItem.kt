package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun DMItem(channel: Channel, chatState: ChatState) {
    val isSelected = chatState.selectedChannel?.id == channel.id
    val recipient = channel.recipients?.firstOrNull()
    val avatarUrl = recipient?.avatar?.let { 
        "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png"
    }
    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"

    val contextMenuItems = remember(channel, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark as Read", Icons.Filled.Check) { /* TODO */ },
            ContextMenuItem("Profile", Icons.Filled.AccountCircle) { recipient?.let { chatState.showProfile(it.id) } },
            ContextMenuItem("Close DM", Icons.Filled.Close, color = Color.Red) { /* TODO */ }
        )
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(channel.id) })
        }
        items
    }
    
    ContextMenu(items = contextMenuItems) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp).padding(horizontal = 8.dp),
            onClick = { chatState.selectChannel(channel) },
            color = if (isSelected) 
                MaterialTheme.colorScheme.surfaceVariant 
            else Color.Transparent,
            shape = MaterialTheme.shapes.small
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(modifier = Modifier.size(32.dp)) {
                    Surface(
                        modifier = Modifier.fillMaxSize(),
                        shape = androidx.compose.foundation.shape.CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer
                    ) {
                        if (avatarUrl != null) {
                            AsyncImage(model = avatarUrl, contentDescription = name, modifier = Modifier.fillMaxSize())
                        } else {
                            Box(contentAlignment = Alignment.Center) {
                                Text(name.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                            }
                        }
                    }
                    
                    if (recipient != null) {
                        val bgColor = if (isSelected) MaterialTheme.colorScheme.surfaceVariant else MaterialTheme.colorScheme.surfaceContainerLow
                        StatusIndicator(
                            status = chatState.getUserStatus(recipient.id),
                            size = 14.dp,
                            modifier = Modifier.align(Alignment.BottomEnd).offset(x = 2.dp, y = 2.dp),
                            borderColor = bgColor,
                            backgroundColor = bgColor
                        )
                    }
                }
                Spacer(modifier = Modifier.width(12.dp))
                Text(
                    text = name,
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (isSelected) 
                        MaterialTheme.colorScheme.onSurface 
                    else MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}
