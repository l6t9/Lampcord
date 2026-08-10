package me.lampu.lampcord.shared.ui.components.guilds

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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ChannelItem
import me.lampu.lampcord.shared.ui.components.ChannelSkeleton
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.VerticalScrollbar
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun GuildChannelList(chatState: ChatState) {
    val guild = chatState.selectedGuild
    val member = chatState.currentMember
    val showHidden = chatState.settingsStore.showHiddenChannels

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
        val allChannels = chatState.channels
        val visibleChannels by remember(guild, member, showHidden) {
            derivedStateOf {
                if (guild == null || member == null) allChannels.toList()
                else allChannels.filter { channel ->
                    PermissionHelper.canViewChannel(member, guild, channel, chatState.currentUser?.id) || showHidden
                }
            }
        }

        val categories = visibleChannels.filter { it.type == 4 }.sortedBy { it.position ?: 0 }
        val rootChannels = visibleChannels.filter { it.parent_id == null && it.type != 4 }.sortedBy { it.position ?: 0 }

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
                    Box(Modifier.animateItem()) {
                        GuildCategoryItem(category, visibleChannels, chatState)
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
            color = MaterialTheme.colorScheme.surface.copy(alpha = alpha),
            shadowElevation = 0.dp,
            tonalElevation = 0.dp,
            onClick = { 
                if (getPlatformName() == "android" || getPlatformName() == "ios") {
                    chatState.isServerMenuVisible = true
                } else {
                    menuExpanded = true 
                }
            }
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
                    val showChannelsAndRoles = guild?.features?.contains("COMMUNITY") == true
                    if (showChannelsAndRoles) {
                        DropdownMenuItem(
                            text = { Text("Channels & Roles") },
                            onClick = { 
                                chatState.isChannelsAndRolesVisible = true
                                chatState.selectedChannel = null
                                chatState.selectedThread = null
                                menuExpanded = false 
                            },
                            leadingIcon = { Icon(Icons.Filled.Flag, null, modifier = Modifier.size(18.dp)) }
                        )
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                    }

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
                    DropdownMenuItem(
                        text = { Text("Server Settings") },
                        onClick = { 
                            chatState.isServerSettingsVisible = true
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.Settings, null, modifier = Modifier.size(18.dp)) }
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

