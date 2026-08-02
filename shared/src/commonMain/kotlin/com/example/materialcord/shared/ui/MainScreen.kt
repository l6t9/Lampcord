package com.example.materialcord.shared.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Guild
import com.example.materialcord.shared.model.Message
import com.example.materialcord.shared.model.Channel
import com.example.materialcord.shared.model.GuildFolder
import com.example.materialcord.shared.model.Member
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    chatState: ChatState = koinInject()
) {
    if (!chatState.isConnected) {
        if (chatState.isConnecting) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = androidx.compose.ui.Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            LoginScreen(onLoginSuccess = { })
        }
    } else {
        Box(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainer)) {
            Row(modifier = Modifier.fillMaxSize()) {
                // Sidebar (Guilds Rail)
                Column(
                    modifier = Modifier
                        .width(80.dp)
                        .fillMaxHeight()
                        .background(MaterialTheme.colorScheme.surfaceContainer)
                        .padding(top = 32.dp) // Space for persistent top bar overlay
                        .padding(vertical = 12.dp),
                    horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    // Discord Logo / Home / DMs
                    Surface(
                        modifier = Modifier.size(48.dp),
                        onClick = {
                            chatState.selectedGuild = null
                            chatState.selectedChannel = null
                        },
                        shape = if (chatState.selectedGuild == null) MaterialTheme.shapes.medium else MaterialTheme.shapes.extraLarge,
                        color = if (chatState.selectedGuild == null) Color(0xFF5865F2) else MaterialTheme.colorScheme.surfaceVariant
                    ) {
                        Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                            Icon(
                                imageVector = MaterialcordIcons.Brand.Discord,
                                contentDescription = "Home",
                                tint = if (chatState.selectedGuild == null) Color.White else MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.size(28.dp)
                            )
                        }
                    }

                    HorizontalDivider(modifier = Modifier.width(32.dp), color = MaterialTheme.colorScheme.outlineVariant)

                    // Guild list with folders
                    LazyColumn(
                        modifier = Modifier.weight(1f),
                        horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                        contentPadding = PaddingValues(bottom = 80.dp) // Space for AccountPanel overlay
                    ) {
                        val folders = chatState.userSettings?.guild_folders ?: emptyList()
                        
                        if (folders.isEmpty()) {
                            items(chatState.guilds) { guild ->
                                GuildIcon(
                                    guild = guild,
                                    isSelected = chatState.selectedGuild?.id == guild.id,
                                    onClick = { chatState.selectGuild(guild) }
                                )
                            }
                        } else {
                            items(folders) { folder ->
                                if (folder.id == null && folder.guild_ids.size == 1) {
                                    val guild = chatState.guilds.find { it.id == folder.guild_ids.first() }
                                    if (guild != null) {
                                        GuildIcon(
                                            guild = guild,
                                            isSelected = chatState.selectedGuild?.id == guild.id,
                                            onClick = { chatState.selectGuild(guild) }
                                        )
                                    }
                                } else {
                                    GuildFolderItem(folder, chatState)
                                }
                            }
                        }
                    }
                }

                // The rest of the app content area
                Row(modifier = Modifier.weight(1f).fillMaxHeight().padding(top = 32.dp)) {
                    // Sidebar (Channels or DMs)
                    Box(
                        modifier = Modifier
                            .width(240.dp)
                            .fillMaxHeight()
                            .background(MaterialTheme.colorScheme.surfaceContainer)
                    ) {
                        if (chatState.selectedGuild != null) {
                            GuildChannelList(chatState)
                        } else {
                            DMList(chatState)
                        }
                    }

                    // Main View (Chat + Header + Member List)
                    Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                        val selectedChannel = chatState.selectedChannel
                        val selectedGuild = chatState.selectedGuild

                        // Channel Info Bar (Below Persistent Top Bar)
                        if (selectedChannel != null) {
                            Surface(
                                modifier = Modifier.fillMaxWidth().height(48.dp),
                                color = MaterialTheme.colorScheme.surfaceContainer,
                                shadowElevation = 0.dp,
                                tonalElevation = 0.dp
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                                    verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                                ) {
                                    val displayChannel = chatState.selectedThread ?: selectedChannel
                                    
                                    if (selectedGuild == null) {
                                        // DM style
                                        val recipient = selectedChannel.recipients?.firstOrNull()
                                        val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                                        Text("@", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        Spacer(modifier = Modifier.width(8.dp))
                                        Text(text = name, style = MaterialTheme.typography.titleSmall)
                                    } else {
                                        val icon = when(displayChannel.type) {
                                            15 -> "F" // Forum
                                            10, 11, 12 -> ">" // Thread
                                            else -> "#"
                                        }
                                        Text(icon, style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        Spacer(modifier = Modifier.width(8.dp))
                                        Text(text = displayChannel.name ?: "unnamed", style = MaterialTheme.typography.titleSmall)
                                    }
                                    
                                    if (displayChannel.topic?.isNotBlank() == true) {
                                        Spacer(modifier = Modifier.width(12.dp))
                                        Text("•", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        Spacer(modifier = Modifier.width(12.dp))
                                        Text(
                                            text = displayChannel.topic,
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.weight(1f)
                                        )
                                    } else {
                                        Spacer(modifier = Modifier.weight(1f))
                                    }
                                    
                                    // Search bar (Far Right)
                                    Surface(
                                        modifier = Modifier.width(160.dp).height(24.dp),
                                        shape = RoundedCornerShape(4.dp),
                                        color = MaterialTheme.colorScheme.surfaceContainerHigh
                                    ) {
                                        Row(
                                            modifier = Modifier.padding(horizontal = 8.dp),
                                            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                                        ) {
                                            Text("Search", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            Spacer(Modifier.weight(1f))
                                            Icon(MaterialcordIcons.Filled.Search, null, modifier = Modifier.size(14.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                                        }
                                    }
                                }
                            }
                        }

                        // Horizontal layout for Chat Area and Member List
                        Row(modifier = Modifier.fillMaxWidth().weight(1f)) {
                            // Chat Area
                            Column(
                                modifier = Modifier
                                    .weight(1f)
                                    .fillMaxHeight()
                                    .background(MaterialTheme.colorScheme.surface)
                            ) {
                                Surface(
                                    modifier = Modifier.fillMaxSize(),
                                    color = MaterialTheme.colorScheme.surface,
                                    shape = RoundedCornerShape(0.dp)
                                ) {
                                    Column(modifier = Modifier.fillMaxSize()) {
                                        if (chatState.selectedChannel != null) {
                                            if (chatState.selectedChannel?.type == 15 && chatState.selectedThread == null) {
                                                ForumPostList(chatState)
                                            } else {
                                                ChatArea(
                                                    modifier = Modifier.weight(1f),
                                                    chatState = chatState
                                                )
                                                
                                                // Message Input Bar
                                                var messageText by remember { mutableStateOf("") }

                                                Surface(
                                                    color = MaterialTheme.colorScheme.surface,
                                                    modifier = Modifier.fillMaxWidth()
                                                ) {
                                                    Column {
                                                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
                                                        Row(
                                                            modifier = Modifier
                                                                .fillMaxWidth()
                                                                .height(68.dp)
                                                                .padding(horizontal = 8.dp),
                                                            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                                                        ) {
                                                            IconButton(
                                                                onClick = { /* TODO: Add */ },
                                                            ) {
                                                                Surface(
                                                                    modifier = Modifier.size(32.dp),
                                                                    shape = androidx.compose.foundation.shape.CircleShape,
                                                                    color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                                                                ) {
                                                                    Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                                                                        Icon(
                                                                            imageVector = MaterialcordIcons.Filled.Add,
                                                                            contentDescription = "Add",
                                                                            modifier = Modifier.size(20.dp)
                                                                        )
                                                                    }
                                                                }
                                                            }
                                                            
                                                            val placeholderText = when {
                                                                chatState.selectedThread != null -> "Reply to thread..."
                                                                selectedChannel != null && selectedGuild == null -> {
                                                                    val recipient = selectedChannel.recipients?.firstOrNull()
                                                                    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                                                                    "Message @$name"
                                                                }
                                                                selectedChannel != null -> "Message #${selectedChannel.name ?: "unnamed"}"
                                                                else -> "Message"
                                                            }

                                                            TextField(
                                                                value = messageText,
                                                                onValueChange = { messageText = it },
                                                                modifier = Modifier
                                                                    .weight(1f)
                                                                    .padding(horizontal = 4.dp),
                                                                placeholder = { Text(placeholderText, style = MaterialTheme.typography.bodyMedium) },
                                                                shape = RoundedCornerShape(24.dp),
                                                                colors = TextFieldDefaults.colors(
                                                                    focusedTextColor = MaterialTheme.colorScheme.onSurface,
                                                                    unfocusedTextColor = MaterialTheme.colorScheme.onSurface,
                                                                    focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                                                    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                                                    focusedIndicatorColor = Color.Transparent,
                                                                    unfocusedIndicatorColor = Color.Transparent,
                                                                    disabledIndicatorColor = Color.Transparent,
                                                                    cursorColor = MaterialTheme.colorScheme.primary
                                                                )
                                                            )
                                                            
                                                            if (messageText.isNotBlank()) {
                                                                Spacer(modifier = Modifier.width(4.dp))
                                                                IconButton(
                                                                    onClick = {
                                                                        chatState.sendMessage(messageText)
                                                                        messageText = ""
                                                                    }
                                                                ) {
                                                                    Surface(
                                                                        modifier = Modifier.size(32.dp),
                                                                        shape = androidx.compose.foundation.shape.CircleShape,
                                                                        color = MaterialTheme.colorScheme.primary
                                                                    ) {
                                                                        Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                                                                            Icon(
                                                                                imageVector = MaterialcordIcons.AutoMirrored.Filled.ArrowForward,
                                                                                contentDescription = "Send",
                                                                                tint = MaterialTheme.colorScheme.onPrimary,
                                                                                modifier = Modifier.size(18.dp)
                                                                            )
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = androidx.compose.ui.Alignment.Center) {
                                                Text("Select a channel to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                    }
                                }
                            }

                            // Member List (Right Sidebar)
                            if (chatState.selectedGuild != null && chatState.selectedChannel?.type != 15) {
                                Box(
                                    modifier = Modifier
                                        .width(240.dp)
                                        .fillMaxHeight()
                                        .background(MaterialTheme.colorScheme.surfaceContainer)
                                ) {
                                    MemberList(chatState)
                                }
                            }
                        }
                    }
                }
            }
            
            // Integrated Top Bar Overlay (covers full width including server list)
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(32.dp)
                    .background(MaterialTheme.colorScheme.surfaceContainer)
            ) {
                Box(contentAlignment = androidx.compose.ui.Alignment.CenterEnd, modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                    // Global actions could go here (Help, Inbox, etc)
                }
            }

            // Account Panel overlays at the bottom START (overlaps both sidebars)
            Box(modifier = Modifier.align(androidx.compose.ui.Alignment.BottomStart).width(320.dp)) {
                AccountPanel(chatState)
            }
        }
    }
}

@Composable
fun GuildFolderItem(folder: GuildFolder, chatState: ChatState) {
    var expanded by remember { mutableStateOf(false) }
    val folderColor = folder.color?.let { Color(it.toLong() or 0xFF000000L) } ?: Color(0xFF5865F2)

    Surface(
        color = if (expanded) MaterialTheme.colorScheme.surface else Color.Transparent,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier.width(52.dp)
    ) {
        Column(
            horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
            modifier = Modifier.padding(vertical = 4.dp)
        ) {
            Surface(
                modifier = Modifier.size(48.dp),
                onClick = { expanded = !expanded },
                shape = MaterialTheme.shapes.extraLarge,
                color = if (expanded) Color.Transparent else folderColor.copy(alpha = 0.2f)
            ) {
                Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                    Icon(
                        imageVector = if (expanded) MaterialcordIcons.Filled.FolderOpen else MaterialcordIcons.Filled.Folder,
                        contentDescription = folder.name ?: "Folder",
                        tint = if (expanded) MaterialTheme.colorScheme.primary else folderColor,
                        modifier = Modifier.size(24.dp)
                    )
                }
            }

            if (expanded) {
                Column(
                    modifier = Modifier.padding(top = 8.dp, bottom = 4.dp),
                    horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    folder.guild_ids.forEach { guildId ->
                        val guild = chatState.guilds.find { it.id == guildId }
                        if (guild != null) {
                            GuildIcon(
                                guild = guild,
                                isSelected = chatState.selectedGuild?.id == guild.id,
                                onClick = { chatState.selectGuild(guild) }
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun AccountPanel(chatState: ChatState) {
    var showMenu by remember { mutableStateOf(false) }
    
    chatState.currentUser?.let { user ->
        val userAvatarUrl = user.avatar?.let { 
            "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
        }
        
        Surface(
            modifier = Modifier.fillMaxWidth().height(68.dp),
            onClick = { showMenu = !showMenu },
            color = MaterialTheme.colorScheme.surfaceContainer,
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp),
                verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
            ) {
                Box {
                    Surface(
                        modifier = Modifier.size(38.dp),
                        shape = androidx.compose.foundation.shape.CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer
                    ) {
                        if (userAvatarUrl != null) {
                            AsyncImage(model = userAvatarUrl, contentDescription = "Me", modifier = Modifier.fillMaxSize())
                        } else {
                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = androidx.compose.ui.Alignment.Center) {
                                Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.labelLarge)
                            }
                        }
                    }
                    
                    // Status dot
                    Surface(
                        modifier = Modifier.size(12.dp).align(androidx.compose.ui.Alignment.BottomEnd),
                        shape = androidx.compose.foundation.shape.CircleShape,
                        color = Color(0xFF43B581),
                        border = androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.surfaceContainer)
                    ) {}
                }
                
                Spacer(modifier = Modifier.width(12.dp))
                
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = user.global_name ?: user.username,
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = "Online",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                IconButton(
                    onClick = { /* TODO: Open Settings */ },
                    colors = IconButtonDefaults.filledTonalIconButtonColors()
                ) {
                    Icon(
                        imageVector = MaterialcordIcons.Filled.Settings,
                        contentDescription = "Settings",
                        modifier = Modifier.size(20.dp)
                    )
                }
            }

            DropdownMenu(
                expanded = showMenu,
                onDismissRequest = { showMenu = false }
            ) {
                DropdownMenuItem(
                    text = {
                        Column {
                            Text(user.global_name ?: user.username, style = MaterialTheme.typography.titleSmall)
                            Text("@${user.username}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    },
                    onClick = { },
                    enabled = false
                )
                HorizontalDivider()
                DropdownMenuItem(
                    leadingIcon = { Icon(MaterialcordIcons.Filled.Settings, null, modifier = Modifier.size(18.dp)) },
                    text = { Text("User Settings") },
                    onClick = { showMenu = false }
                )
                DropdownMenuItem(
                    leadingIcon = { Text("🚪") },
                    text = { Text("Log Out") },
                    onClick = { 
                        showMenu = false
                    }
                )
            }
        }
    }
}

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
                            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
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
            shape = RoundedCornerShape(topStart = 16.dp)
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
                    verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
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
            if (alpha > 0.5f) {
                HorizontalDivider(
                    modifier = Modifier.align(androidx.compose.ui.Alignment.BottomCenter),
                    color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = (alpha - 0.5f) * 2f)
                )
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
            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
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
fun ForumPostList(chatState: ChatState) {
    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(chatState.forumThreads) { thread ->
            ForumPostItem(thread, onClick = { chatState.selectThread(thread) })
        }
    }
}

@Composable
fun ForumPostItem(thread: Channel, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(8.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(
                text = thread.name ?: "unnamed post",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface
            )
            Spacer(modifier = Modifier.height(4.dp))
            Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Text(
                    text = "${thread.message_count ?: 0} messages",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (thread.last_message_id != null) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("•", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "Last active recently", // Simplification
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
fun DMList(chatState: ChatState) {
    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(topStart = 16.dp)
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            Surface(
                modifier = Modifier.fillMaxWidth().height(48.dp),
                color = MaterialTheme.colorScheme.surfaceContainer,
                shadowElevation = 0.dp,
                tonalElevation = 0.dp
            ) {
                Box(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                    contentAlignment = androidx.compose.ui.Alignment.CenterStart
                ) {
                    Text(
                        text = "Direct Messages",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                }
            }

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
                            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                        ) {
                            Surface(
                                modifier = Modifier.size(32.dp),
                                shape = androidx.compose.foundation.shape.CircleShape,
                                color = MaterialTheme.colorScheme.secondaryContainer
                            ) {
                                if (avatarUrl != null) {
                                    AsyncImage(model = avatarUrl, contentDescription = name, modifier = Modifier.fillMaxSize())
                                } else {
                                    Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
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

@Composable
fun GuildIcon(
    guild: Guild,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    val iconUrl = if (guild.icon != null) {
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png"
    } else null

    Surface(
        modifier = Modifier.size(48.dp),
        onClick = onClick,
        shape = if (isSelected) MaterialTheme.shapes.medium else MaterialTheme.shapes.extraLarge,
        color = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
    ) {
        if (iconUrl != null) {
            AsyncImage(
                model = iconUrl,
                contentDescription = guild.name,
                modifier = Modifier.fillMaxSize()
            )
        } else {
            Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                Text(
                    text = guild.name?.take(1) ?: "?",
                    color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun ChatArea(
    modifier: Modifier = Modifier,
    chatState: ChatState
) {
    LazyColumn(
        modifier = modifier.fillMaxSize().padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        reverseLayout = true
    ) {
        items(chatState.messages) { message ->
            MessageItem(message)
        }
    }
}

@Composable
fun MessageItem(message: Message) {
    Row(modifier = Modifier.fillMaxWidth()) {
        val avatarUrl = message.author.avatar?.let { 
            "https://cdn.discordapp.com/avatars/${message.author.id}/$it.png"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = "Avatar",
                modifier = Modifier.size(40.dp).clip(MaterialTheme.shapes.extraLarge)
            )
        } else {
            Surface(
                modifier = Modifier.size(40.dp),
                shape = MaterialTheme.shapes.extraLarge,
                color = MaterialTheme.colorScheme.primaryContainer
            ) {
                Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                    Text(
                        message.author.username.take(1).uppercase(),
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                }
            }
        }
        
        Spacer(modifier = Modifier.width(12.dp))
        
        Column {
            Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Text(
                    text = message.author.global_name ?: message.author.username,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.primary
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = message.timestamp.take(10), // Simplistic timestamp
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Text(
                text = message.content,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface
            )

            if (message.attachments.isNotEmpty()) {
                Spacer(modifier = Modifier.height(8.dp))
                MessageAttachments(message.attachments)
            }
        }
    }
}

@Composable
fun MessageAttachments(attachments: List<com.example.materialcord.shared.model.Attachment>) {
    val images = attachments.filter { it.content_type?.startsWith("image/") == true }
    
    if (images.isNotEmpty()) {
        MessageMosaic(images)
    }
    
    // Non-image attachments could be listed here
}

@Composable
fun MessageMosaic(images: List<com.example.materialcord.shared.model.Attachment>) {
    val spacing = 4.dp
    val maxWidth = 500.dp
    
    Box(modifier = Modifier.widthIn(max = maxWidth).clip(RoundedCornerShape(8.dp))) {
        when (images.size) {
            1 -> {
                val image = images[0]
                val aspectRatio = if (image.width != null && image.height != null) {
                    image.width.toFloat() / image.height.toFloat()
                } else 1f
                
                AsyncImage(
                    model = image.proxy_url,
                    contentDescription = image.filename,
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(aspectRatio.coerceIn(0.5f, 2f))
                        .clip(RoundedCornerShape(8.dp))
                )
            }
            2 -> {
                Row(modifier = Modifier.height(200.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    images.forEach { image ->
                        AsyncImage(
                            model = image.proxy_url,
                            contentDescription = image.filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                }
            }
            3 -> {
                Row(modifier = Modifier.height(300.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    AsyncImage(
                        model = images[0].proxy_url,
                        contentDescription = images[0].filename,
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                        modifier = Modifier.weight(1f).fillMaxHeight()
                    )
                    Column(modifier = Modifier.weight(1f).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[1].proxy_url,
                            contentDescription = images[1].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxWidth()
                        )
                        AsyncImage(
                            model = images[2].proxy_url,
                            contentDescription = images[2].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxWidth()
                        )
                    }
                }
            }
            4 -> {
                Column(modifier = Modifier.height(300.dp), verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Row(modifier = Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[0].proxy_url,
                            contentDescription = images[0].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                        AsyncImage(
                            model = images[1].proxy_url,
                            contentDescription = images[1].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                    Row(modifier = Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[2].proxy_url,
                            contentDescription = images[2].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                        AsyncImage(
                            model = images[3].proxy_url,
                            contentDescription = images[3].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                }
            }
            else -> {
                // Simplified grid for more than 4
                val rows = (images.size + 1) / 2
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    for (i in 0 until rows) {
                        Row(modifier = Modifier.height(150.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            val first = i * 2
                            if (first < images.size) {
                                AsyncImage(
                                    model = images[first].proxy_url,
                                    contentDescription = images[first].filename,
                                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                    modifier = Modifier.weight(1f).fillMaxHeight()
                                )
                            }
                            val second = i * 2 + 1
                            if (second < images.size) {
                                AsyncImage(
                                    model = images[second].proxy_url,
                                    contentDescription = images[second].filename,
                                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                    modifier = Modifier.weight(1f).fillMaxHeight()
                                )
                            } else if (first < images.size) {
                                Spacer(modifier = Modifier.weight(1f))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun MemberList(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxSize()) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(0.dp)
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(top = 8.dp),
                contentPadding = PaddingValues(bottom = 80.dp)
            ) {
                items(chatState.members) { member ->
                    MemberItem(member)
                }
            }
        }
    }
}

@Composable
fun MemberItem(member: Member) {
    val user = member.user ?: return
    val avatarUrl = member.avatar?.let {
        "https://cdn.discordapp.com/guilds/${user.id}/users/${user.id}/avatars/$it.png"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
    }

    Surface(
        modifier = Modifier.fillMaxWidth().height(44.dp).padding(horizontal = 8.dp),
        onClick = { },
        color = Color.Transparent,
        shape = MaterialTheme.shapes.small
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp),
            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
        ) {
            Surface(
                modifier = Modifier.size(32.dp),
                shape = androidx.compose.foundation.shape.CircleShape,
                color = MaterialTheme.colorScheme.secondaryContainer
            ) {
                if (avatarUrl != null) {
                    AsyncImage(model = avatarUrl, contentDescription = user.username, modifier = Modifier.fillMaxSize())
                } else {
                    Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                        Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                    }
                }
            }
            Spacer(modifier = Modifier.width(12.dp))
            Column {
                Text(
                    text = member.nick ?: user.global_name ?: user.username,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}
