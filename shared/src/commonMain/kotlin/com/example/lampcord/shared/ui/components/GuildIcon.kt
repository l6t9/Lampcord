package com.example.lampcord.shared.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.unit.dp
import com.example.lampcord.shared.model.Guild
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.icons.Icons
import com.example.lampcord.shared.utils.setClipboardText

@Composable
fun GuildIcon(
    guild: Guild,
    isSelected: Boolean,
    chatState: ChatState,
    onClick: () -> Unit
) {
    val iconUrl = if (guild.icon != null) {
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=96"
    } else null

    val cornerRadius by animateDpAsState(
        targetValue = if (isSelected) 12.dp else 16.dp,
        animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)
    )
    val color by animateColorAsState(
        targetValue = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant,
        animationSpec = spring(stiffness = 400f)
    )
    val indicatorHeight by animateDpAsState(
        targetValue = if (isSelected) 40.dp else 0.dp,
        animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)
    )

    val contextMenuItems = remember(guild, isSelected, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark as Read", Icons.Filled.Check) { /* TODO */ },
            ContextMenuItem("Server Profile", Icons.Filled.AccountCircle) { /* TODO */ }
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
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp),
            contentAlignment = Alignment.Center
        ) {
            // Left side indicator
            Box(
                modifier = Modifier
                    .align(Alignment.CenterStart)
                    .width(4.dp)
                    .height(indicatorHeight)
                    .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                    .background(MaterialTheme.colorScheme.onSurface)
            )

            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(cornerRadius))
                    .background(if (iconUrl != null) Color.Transparent else color)
                    .clickable { onClick() },
                contentAlignment = Alignment.Center
            ) {
                if (iconUrl != null) {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = guild.name,
                        modifier = Modifier.fillMaxSize(),
                        filterQuality = FilterQuality.Medium
                    )
                } else {
                    Text(
                        text = guild.name?.take(1) ?: "?",
                        color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}
