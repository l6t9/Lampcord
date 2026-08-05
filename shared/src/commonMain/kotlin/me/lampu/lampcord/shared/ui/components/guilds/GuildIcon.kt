package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.AsyncImage
import androidx.compose.foundation.background
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp

@Composable
fun GuildIcon(
    guild: Guild,
    isSelected: Boolean,
    chatState: ChatState,
    onClick: () -> Unit
) {
    val isAnimated = guild.icon?.startsWith("a_") == true
    val iconUrl = if (guild.icon != null) {
        val ext = if (isAnimated && isSelected) "gif" else "png"
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.$ext?size=96"
    } else null

    val isUnread by remember(guild.id, chatState.readStates) {
        derivedStateOf { chatState.guilds.find { it.id == guild.id }?.channels?.any { chatState.isUnread(it) } ?: false }
    }
    val mentionCount by remember(guild.id, chatState.readStates) {
        derivedStateOf { chatState.guilds.find { it.id == guild.id }?.channels?.sumOf { chatState.getMentionCount(it.id) } ?: 0 }
    }

    val contextMenuItems = remember(guild, isSelected, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark as Read", Icons.Filled.Check) {
                chatState.markGuildAsRead(guild.id)
            },
            ContextMenuItem("Server Profile", Icons.Filled.AccountCircle) {
                chatState.currentUser?.let { chatState.showProfile(it.id) }
            }
        )
        if (!isSelected) {
            items.add(ContextMenuItem("Leave Server", Icons.Filled.Logout, color = Color.Red) { /* TODO */ })
        }
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(guild.id) })
        }
        items
    }

    ContextMenu(items = contextMenuItems) {
        Box(contentAlignment = Alignment.Center) {
            if (isUnread && !isSelected) {
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterStart)
                        .padding(start = 0.dp)
                        .width(4.dp)
                        .height(8.dp)
                        .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                        .background(MaterialTheme.colorScheme.onSurface)
                )
            }

            RegularGuildItem(
                isSelected = isSelected,
                onClick = onClick,
                selectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent,
                unselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.surfaceVariant else Color.Transparent
            ) {
                if (iconUrl != null) {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = guild.name,
                        modifier = Modifier.fillMaxSize(),
                        filterQuality = FilterQuality.Medium
                    )
                } else {
                    val initials = remember(guild.name) {
                        guild.name?.split(" ")?.mapNotNull { it.firstOrNull() }?.joinToString("") ?: "?"
                    }
                    Text(
                        text = initials,
                        style = MaterialTheme.typography.titleMedium,
                        color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary,
                        textAlign = TextAlign.Center,
                        maxLines = 1,
                        fontSize = if (initials.length > 3) 12.sp else 16.sp
                    )
                }
            }

            if (mentionCount > 0) {
                Surface(
                    color = MaterialTheme.colorScheme.error,
                    shape = CircleShape,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(bottom = 2.dp, end = 2.dp)
                        .height(20.dp)
                        .widthIn(min = 20.dp),
                    shadowElevation = 2.dp
                ) {
                    Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(horizontal = 4.dp)) {
                        Text(
                            text = mentionCount.toString(),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onError,
                            fontWeight = FontWeight.Bold,
                            fontSize = 10.sp
                        )
                    }
                }
            }
        }
    }
}
