package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import me.lampu.lampcord.shared.settings.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.rememberGuildMediaUrls
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ChannelItem
import me.lampu.lampcord.shared.ui.components.ChannelSkeleton
import me.lampu.lampcord.shared.ui.components.VerticalScrollbar
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.handCursor

@Composable
fun GuildChannelList(
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val allMembers by userStore.members.collectAsState()
    
    val member = remember(guild, currentUser, allMembers) {
        val g = guild
        val u = currentUser
        if (g == null || u == null) null
        else allMembers[g.id]?.get(u.id)
    }
    val showHidden = settingsStore.showHiddenChannels

    val guildMedia = rememberGuildMediaUrls(
        guildId = guild?.id,
        guildIconUrl = null,
        guildBannerUrl = guild?.banner?.let {
            "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600"
        }
    )
    val bannerUrl = guildMedia.banner
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }
    var menuExpanded by remember { mutableStateOf(false) }
    var showLeaveDialog by remember { mutableStateOf(false) }

    if (showLeaveDialog) {
        LeaveServerDialog(
            guildName = guild?.name ?: "this server",
            onDismiss = { showLeaveDialog = false },
            onConfirm = {
                guild?.let { g ->
                    guildStore.leaveGuild(g.id) { if (navigationStore.selectedGuild?.id == g.id) navigationStore.selectHome() }
                }
                showLeaveDialog = false
            }
        )
    }

    val alpha = remember(bannerUrl) {
        if (bannerUrl == null) 1f
        else if (scrollState.firstVisibleItemIndex > 0) 1f
        else (scrollState.firstVisibleItemScrollOffset.toFloat() / 200f).coerceIn(0f, 1f)
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
        val allGuildChannels by guildStore.allGuildChannels.collectAsState()
        val visibleChannels = remember(guild, member, showHidden, allGuildChannels) {
            val g = guild
            val u = currentUser
            if (g == null) emptyList()
            else allGuildChannels.values.filter { channel ->
                if (channel.guild_id != g.id) return@filter false
                if (showHidden) return@filter true

                val currentMember = member
                if (currentMember == null) return@filter true
                PermissionHelper.canViewChannel(currentMember, g, channel, u?.id)
            }
        }

        val allChannelsForThisGuild = remember(guild, allGuildChannels) {
            val g = guild
            if (g == null) emptyList()
            else allGuildChannels.values.filter { it.guild_id == g.id }
        }

        val categories = visibleChannels.filter { it.type == 4 }.distinctBy { it.id }.sortedBy { it.position ?: 0 }
        val rootChannels = visibleChannels.filter { it.parent_id == null && it.type != 4 }
            .distinctBy { it.id }
            .sortedWith(compareBy({ it.type == 2 || it.type == 13 }, { it.position ?: 0 }))

        LazyColumn(
            state = scrollState,
            modifier = Modifier
                .fillMaxSize(),
            contentPadding = PaddingValues(top = if (bannerUrl == null) 48.dp else 0.dp, bottom = 68.dp)
        ) {
            if (allChannelsForThisGuild.isEmpty() && navigationStore.selectedGuild != null) {
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
                    Box((if (Settings.shared.reduceMotion) Modifier else Modifier.animateItem()).padding(vertical = if (settingsStore.messageSpacingMode == me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT) 1.dp else 0.dp)) {
                        ChannelItem(channel)
                    }
                }
                
                items(categories, key = { it.id }) { category ->
                    Box(if (Settings.shared.reduceMotion) Modifier else Modifier.animateItem()) {
                        GuildCategoryItem(category, visibleChannels)
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
                    navigationStore.isServerMenuVisible = true
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
                        modifier = Modifier.weight(1f).then(
                            if (!Settings.shared.marqueeEnabled) Modifier else Modifier.basicMarquee(
                                iterations = 1,
                                initialDelayMillis = 3000,
                                velocity = 30.dp
                            )
                        )
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
                    val g = guild
                    val u = currentUser
                    val showChannelsAndRoles = g?.features?.contains("COMMUNITY") == true
                    if (showChannelsAndRoles) {
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Channels & Roles") },
                            onClick = { 
                                navigationStore.isChannelsAndRolesVisible = true
                                navigationStore.selectedChannel = null
                                navigationStore.selectedThread = null
                                menuExpanded = false 
                            },
                            leadingIcon = { Icon(Icons.Filled.Flag, null, modifier = Modifier.size(18.dp)) }
                        )
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                    }

                    DropdownMenuItem(
                        modifier = Modifier.handCursor(),
                        text = { Text("Mark As Read") },
                        onClick = { 
                            g?.let { guildStore.markGuildAsRead(it.id) }
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.Check, null, modifier = Modifier.size(18.dp)) }
                    )
                    DropdownMenuItem(
                        modifier = Modifier.handCursor(),
                        text = { Text("Edit Profile") },
                        onClick = { 
                            navigationStore.navigateToSettings("PROFILES")
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.AccountCircle, null, modifier = Modifier.size(18.dp)) }
                    )
                    
                    val canManageGuild = remember(g, member) {
                        if (g == null || member == null) false
                        else PermissionHelper.hasPermission(member, g, null, Permission.MANAGE_GUILD, u?.id)
                    }

                    if (canManageGuild) {
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Server Settings") },
                            onClick = { 
                                navigationStore.isServerSettingsVisible = true
                                menuExpanded = false 
                            },
                            leadingIcon = { Icon(Icons.Filled.Settings, null, modifier = Modifier.size(18.dp)) }
                        )
                    }
                    HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                    val errorColor = MaterialTheme.colorScheme.error
                    DropdownMenuItem(
                        modifier = Modifier.handCursor(),
                        text = { Text("Leave Server", color = errorColor) },
                        onClick = { 
                            showLeaveDialog = true
                            menuExpanded = false 
                        },
                        leadingIcon = { Icon(Icons.Filled.Logout, null, tint = errorColor, modifier = Modifier.size(18.dp)) }
                    )
                    if (settingsStore.userSettings?.developer_mode == true) {
                        HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Copy ID") },
                            onClick = { 
                                g?.let { setClipboardText(it.id) }
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
