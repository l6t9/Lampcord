package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
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

    val status = recipient?.let { chatState.getUserStatus(it.id) } ?: "offline"

    var isHovered by remember { mutableStateOf(false) }

    val isUnread by remember(channel, chatState.readStates[channel.id]) {
        derivedStateOf { chatState.isUnread(channel) }
    }

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
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .padding(horizontal = 8.dp)
                .pointerInput(Unit) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            when (event.type) {
                                PointerEventType.Enter -> isHovered = true
                                PointerEventType.Exit -> isHovered = false
                            }
                        }
                    }
                },
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
                    AvatarWithDecoration(
                        avatarUrl = avatarUrl,
                        decorationData = recipient?.avatar_decoration_data ?: recipient?.collectibles?.avatar_decoration,
                        size = 32.dp,
                        status = status
                    )
                }
                Spacer(modifier = Modifier.width(12.dp))
                UsernameView(
                    name = name,
                    style = recipient?.display_name_styles,
                    baseStyle = MaterialTheme.typography.bodyMedium,
                    color = Color.White,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    ignoreEffects = !isHovered,
                    ignoreColors = !isHovered
                )
            }
        }
    }
}
