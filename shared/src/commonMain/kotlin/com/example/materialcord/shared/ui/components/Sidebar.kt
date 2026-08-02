package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Channel
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons

@Composable
fun GuildChannelList(chatState: ChatState) {
    val guild = chatState.selectedGuild
    val bannerUrl = guild?.banner?.let { 
        "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" 
    }
    val scrollState = rememberLazyListState()
    val alpha by remember(bannerUrl) {
        derivedStateOf {
            if (bannerUrl == null) 1f
            else if (scrollState.firstVisibleItemIndex > 0) 1f
            else (scrollState.firstVisibleItemScrollOffset.toFloat() / 200f).coerceIn(0f, 1f)
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(topStart = 16.dp),
            tonalElevation = 0.dp
        ) {
            val channels = chatState.channels
            val categories = channels.filter { it.type == 4 }.sortedBy { it.position ?: 0 }
            val rootChannels = channels.filter { it.parent_id == null && it.type != 4 }.sortedBy { it.position ?: 0 }

            LazyColumn(
                state = scrollState,
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(top = 0.dp, bottom = 80.dp)
            ) {
                if (bannerUrl != null) {
                    item {
                        AsyncImage(
                            model = bannerUrl,
                            contentDescription = "Server Banner",
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(135.dp)
                                .clip(RoundedCornerShape(topStart = 16.dp))
                        )
                    }
                }

                item { Spacer(modifier = Modifier.height(8.dp)) }

                items(rootChannels) { channel ->
                    ChannelItem(channel, chatState)
                }
                
                categories.forEach { category ->
                    item(key = category.id) {
                        var collapsed by remember { mutableStateOf(false) }
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { collapsed = !collapsed }
                                .padding(horizontal = 16.dp, vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = category.name ?: "Category",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Icon(
                                imageVector = if (collapsed) MaterialcordIcons.Filled.ChevronRight else MaterialcordIcons.Filled.KeyboardArrowDown,
                                contentDescription = if (collapsed) "Expand" else "Collapse",
                                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.size(16.dp)
                            )
                        }
                        if (!collapsed) {
                            val categoryChannels = channels.filter { it.parent_id == category.id }.sortedBy { it.position ?: 0 }
                            Column {
                                categoryChannels.forEach { channel ->
                                    ChannelItem(channel, chatState)
                                }
                            }
                        }
                    }
                }
            }
        }

        // Overlay solidified header
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            color = MaterialTheme.colorScheme.surfaceContainer.copy(alpha = alpha),
            shadowElevation = 0.dp,
            tonalElevation = 0.dp,
            onClick = { /* TODO: Guild Menu */ },
            shape = if (alpha > 0.99f) RoundedCornerShape(0.dp) else RoundedCornerShape(topStart = 16.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                // Shadow gradient for readability on white/bright banners
                if (bannerUrl != null) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(
                                Brush.verticalGradient(
                                    colors = listOf(
                                        Color.Black.copy(alpha = 0.5f * (1f - alpha)),
                                        Color.Transparent
                                    )
                                )
                            )
                    )
                }

                Row(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val contentColor = if (bannerUrl != null && alpha < 0.5f) Color.White else MaterialTheme.colorScheme.onSurface
                    
                    Text(
                        text = guild?.name ?: "Guild",
                        style = MaterialTheme.typography.titleSmall,
                        color = contentColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    Icon(
                        imageVector = MaterialcordIcons.Filled.KeyboardArrowDown,
                        contentDescription = "Menu",
                        modifier = Modifier.size(16.dp),
                        tint = contentColor
                    )
                }
            }
        }
    }
}

@Composable
fun ChannelItem(channel: Channel, chatState: ChatState) {
    val isSelected = chatState.selectedChannel?.id == channel.id
    Surface(
        modifier = Modifier.fillMaxWidth().height(34.dp).padding(horizontal = 8.dp),
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
            val icon = when(channel.type) {
                2 -> "V" // Voice
                15 -> "F" // Forum
                else -> "#"
            }
            Text(
                text = icon,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.width(16.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = channel.name ?: "unnamed",
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

@Composable
fun DMList(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainer)) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            color = MaterialTheme.colorScheme.surfaceContainer,
            shadowElevation = 0.dp,
            tonalElevation = 0.dp
        ) {
            Box(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                Text(
                    text = "Direct Messages",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface
                )
            }
        }
        
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(topStart = 16.dp),
            tonalElevation = 0.dp
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(top = 8.dp),
                contentPadding = PaddingValues(bottom = 80.dp)
            ) {
                items(chatState.privateChannels) { channel ->
                    val isSelected = chatState.selectedChannel?.id == channel.id
                    val recipient = channel.recipients?.firstOrNull()
                    val avatarUrl = recipient?.avatar?.let { 
                        "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png"
                    }
                    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                    
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
                            Surface(
                                modifier = Modifier.size(32.dp),
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
        }
    }
}
