package com.example.materialcord.shared.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.components.*
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
        Box(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
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
                            .background(MaterialTheme.colorScheme.surfaceContainer) // Parent sidebar background
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

            // User Profile Dialog
            chatState.selectedProfile?.let { profile ->
                UserProfileDialog(
                    profile = profile,
                    chatState = chatState,
                    onDismiss = { chatState.selectedProfile = null }
                )
            }
        }
    }
}
