package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun ChannelItem(channel: Channel, chatState: ChatState) {
    val isSelected = chatState.selectedChannel?.id == channel.id
    val isUnread by remember(channel, chatState.readStates[channel.id]) {
        derivedStateOf { chatState.isUnread(channel) }
    }
    val mentionCount by remember(channel, chatState.readStates[channel.id]) {
        derivedStateOf { chatState.getMentionCount(channel.id) }
    }

    val contextMenuItems = remember(channel, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark as Read", Icons.Filled.Check) { /* TODO */ },
            ContextMenuItem("Copy Link", Icons.Filled.Link) {
                val guildId = channel.guild_id ?: "@me"
                setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
            }
        )
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(channel.id) })
        }
        items
    }

    ContextMenu(items = contextMenuItems) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(44.dp),
            contentAlignment = Alignment.CenterStart
        ) {
            if (isUnread && !isSelected) {
                Box(
                    modifier = Modifier
                        .size(width = 4.dp, height = 12.dp)
                        .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                        .background(MaterialTheme.colorScheme.onSurface)
                )
            }

            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(40.dp)
                    .padding(horizontal = 8.dp),
                onClick = { chatState.selectChannel(channel) },
                color = if (isSelected) 
                    MaterialTheme.colorScheme.surfaceVariant 
                else Color.Transparent,
                shape = RoundedCornerShape(12.dp)
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val contentColor = if (isSelected || isUnread) 
                        MaterialTheme.colorScheme.onSurface 
                    else MaterialTheme.colorScheme.onSurfaceVariant
                    
                    if (channel.type == 15) {
                        Icon(
                            imageVector = Icons.Outlined.Forum,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp),
                            tint = contentColor
                        )
                    } else if (channel.type == 2) {
                        //Icon(
                        //    imageVector = Icons.Outlined.Call,
                        //    contentDescription = null,
                        //    modifier = Modifier.size(20.dp),
                        //    tint = contentColor
                        //)
                        Text(
                            text = "V",
                            style = MaterialTheme.typography.bodyLarge,
                            color = contentColor,
                            modifier = Modifier.width(20.dp)
                        )
                    } else {
                        Icon(
                            imageVector = Icons.Filled.Tag,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp),
                            tint = contentColor
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Text(
                        text = channel.name ?: "unnamed",
                        style = MaterialTheme.typography.bodyLarge,
                        color = contentColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    
                    if (mentionCount > 0) {
                        Surface(
                            color = MaterialTheme.colorScheme.error,
                            shape = CircleShape,
                            modifier = Modifier.height(20.dp).widthIn(min = 20.dp)
                        ) {
                            Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(horizontal = 6.dp)) {
                                Text(
                                    text = mentionCount.toString(),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onError,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ChannelSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(34.dp)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(16.dp),
            shape = RoundedCornerShape(4.dp)
        )
        Spacer(modifier = Modifier.width(8.dp))
        ShimmerBox(
            modifier = Modifier
                .width(120.dp)
                .height(12.dp),
            shape = RoundedCornerShape(6.dp)
        )
    }
}
