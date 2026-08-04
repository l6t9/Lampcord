package me.lampu.lampcord.shared.ui.components

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
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun GuildChannelList(chatState: ChatState) {
    val guild = chatState.selectedGuild
    val bannerUrl = guild?.banner?.let { 
        "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" 
    }
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }
    var menuExpanded by remember { mutableStateOf(false) }

    val alpha by remember(bannerUrl) {
        derivedStateOf {
            if (bannerUrl == null) 1f
            else if (scrollState.firstVisibleItemIndex > 0) 1f
            else (scrollState.firstVisibleItemScrollOffset.toFloat() / 200f).coerceIn(0f, 1f)
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
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
            }
    ) {
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
                contentPadding = PaddingValues(top = if (bannerUrl == null) 48.dp else 0.dp, bottom = 68.dp)
            ) {
                if (chatState.channels.isEmpty() && chatState.selectedGuild != null) {
                    items(15) {
                        ChannelSkeleton()
                    }
                } else {
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

                    items(rootChannels, key = { it.id }) { channel ->
                        Box(Modifier.animateItem()) {
                            ChannelItem(channel, chatState)
                        }
                    }
                    
                    items(categories, key = { it.id }) { category ->
                        var collapsed by remember { mutableStateOf(false) }
                        Column(Modifier.animateItem()) {
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
                                    imageVector = if (collapsed) Icons.Filled.ChevronRight else Icons.Filled.KeyboardArrowDown,
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
        }

        VerticalScrollbar(
            state = scrollState,
            modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
            isVisible = isHovered
        )

        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            color = MaterialTheme.colorScheme.surfaceContainer.copy(alpha = alpha),
            shadowElevation = 0.dp,
            tonalElevation = 0.dp,
            onClick = { menuExpanded = true },
            shape = if (alpha > 0.99f) RoundedCornerShape(0.dp) else RoundedCornerShape(topStart = 16.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
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
                        style = MaterialTheme.typography.titleSmall.copy(
                            shadow = if (bannerUrl != null && alpha < 0.5f) {
                                androidx.compose.ui.graphics.Shadow(
                                    color = Color.Black.copy(alpha = 0.8f),
                                    offset = androidx.compose.ui.geometry.Offset(0f, 2f),
                                    blurRadius = 12f
                                )
                            } else null
                        ),
                        color = contentColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f)
                    )
                    Icon(
                        imageVector = if (menuExpanded) Icons.Filled.Close else Icons.Filled.KeyboardArrowDown,
                        contentDescription = "Menu",
                        modifier = Modifier.size(16.dp),
                        tint = contentColor
                    )
                }

                DropdownMenu(
                    expanded = menuExpanded,
                    onDismissRequest = { menuExpanded = false },
                    modifier = Modifier.width(220.dp)
                ) {
                    DropdownMenuItem(
                        text = { Text("Mark As Read") },
                        onClick = { 
                            guild?.let { chatState.markGuildAsRead(it.id) }
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.Check, null, modifier = Modifier.size(18.dp)) }
                    )
                    DropdownMenuItem(
                        text = { Text("Server Profile") },
                        onClick = { 
                            chatState.currentUser?.let { chatState.showProfile(it.id) }
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.AccountCircle, null, modifier = Modifier.size(18.dp)) }
                    )
                    HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                    DropdownMenuItem(
                        text = { Text("Leave Server", color = Color.Red) },
                        onClick = { 
                            guild?.let { chatState.leaveGuild(it.id) }
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.Logout, null, tint = Color.Red, modifier = Modifier.size(18.dp)) }
                    )
                    if (chatState.userSettings?.developer_mode == true) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        DropdownMenuItem(
                            text = { Text("Copy ID") },
                            onClick = { 
                                guild?.let { setClipboardText(it.id) }
                                menuExpanded = false 
                            },
                            leadingIcon = { Icon(Icons.Filled.Dns, null, modifier = Modifier.size(18.dp)) }
                        )
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
