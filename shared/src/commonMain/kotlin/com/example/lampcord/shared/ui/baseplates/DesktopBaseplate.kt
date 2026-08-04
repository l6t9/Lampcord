package com.example.lampcord.shared.ui.baseplates

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.components.*
import com.example.lampcord.shared.ui.icons.Icons
import com.example.lampcord.shared.ui.SettingsScreen

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun DesktopBaseplate(chatState: ChatState) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surfaceContainer)
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown) {
                    if (event.isCtrlPressed && event.key == Key.K) {
                        chatState.isQuickSwitcherVisible = true
                        return@onPreviewKeyEvent true
                    }
                }
                false
            }
    ) {
        Row(modifier = Modifier.fillMaxSize()) {
            // Sidebar (Guilds Rail)
            val railScrollState = rememberLazyListState()
            var isRailHovered by remember { mutableStateOf(false) }

            Column(
                modifier = Modifier
                    .width(80.dp)
                    .fillMaxHeight()
                    .background(MaterialTheme.colorScheme.surfaceContainer)
                    .padding(top = 32.dp) // Space for persistent top bar overlay
                    .padding(vertical = 12.dp)
                    .pointerInput(Unit) {
                        awaitPointerEventScope {
                            while (true) {
                                val event = awaitPointerEvent()
                                when (event.type) {
                                    PointerEventType.Enter -> isRailHovered = true
                                    PointerEventType.Exit -> isRailHovered = false
                                }
                            }
                        }
                    },
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Discord Logo / Home / DMs
                val isHomeSelected = chatState.selectedGuild == null
                val homeCornerRadius by animateDpAsState(
                    targetValue = if (isHomeSelected) 12.dp else 16.dp,
                    animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)
                )
                val homeColor by animateColorAsState(
                    targetValue = if (isHomeSelected) Color(0xFF5865F2) else MaterialTheme.colorScheme.surfaceVariant,
                    animationSpec = spring(stiffness = 400f)
                )
                val homeIndicatorHeight by animateDpAsState(
                    targetValue = if (isHomeSelected) 40.dp else 0.dp,
                    animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)
                )

                Box(
                    modifier = Modifier.fillMaxWidth().height(48.dp),
                    contentAlignment = Alignment.Center
                ) {
                    // Left side indicator
                    Box(
                        modifier = Modifier
                            .align(Alignment.CenterStart)
                            .width(4.dp)
                            .height(homeIndicatorHeight)
                            .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                            .background(MaterialTheme.colorScheme.onSurface)
                    )

                    Surface(
                        modifier = Modifier.size(48.dp),
                        onClick = { chatState.selectHome() },
                        shape = RoundedCornerShape(homeCornerRadius),
                        color = homeColor,
                        shadowElevation = 0.dp,
                        tonalElevation = 0.dp,
                        border = null
                    ) {
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                imageVector = Icons.Brand.Discord,
                                contentDescription = "Home",
                                tint = if (isHomeSelected) Color.White else MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.size(28.dp)
                            )
                        }
                    }
                }

                HorizontalDivider(modifier = Modifier.width(32.dp), color = MaterialTheme.colorScheme.outlineVariant)

                // Guild list with folders
                Box(modifier = Modifier.weight(1f)) {
                    LazyColumn(
                        state = railScrollState,
                        modifier = Modifier.fillMaxSize(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                        contentPadding = PaddingValues(bottom = 68.dp) // Match AccountPanel height
                    ) {
                        val folders = chatState.userSettings?.guild_folders ?: emptyList()
                        
                        if (folders.isEmpty()) {
                            items(chatState.guilds) { guild ->
                                GuildIcon(
                                    guild = guild,
                                    isSelected = chatState.selectedGuild?.id == guild.id,
                                    chatState = chatState,
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
                                            chatState = chatState,
                                            onClick = { chatState.selectGuild(guild) }
                                        )
                                    }
                                } else {
                                    GuildFolderItem(folder, chatState)
                                }
                            }
                        }
                    }

                    VerticalScrollbar(
                        state = railScrollState,
                        modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
                        isVisible = isRailHovered
                    )
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
                    AnimatedContent(
                        targetState = chatState.selectedGuild != null,
                        transitionSpec = {
                            fadeIn() togetherWith fadeOut()
                        },
                        label = "SidebarTransition"
                    ) { isGuild ->
                        if (isGuild) {
                            GuildChannelList(chatState)
                        } else {
                            DMList(chatState)
                        }
                    }
                }

                // Main View (Chat + Header + Member List)
                val selectedChannel = chatState.selectedChannel
                val selectedThread = chatState.selectedThread
                val selectedGuild = chatState.selectedGuild
                val activeChannel = selectedThread ?: selectedChannel

                Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                    AnimatedContent(
                        targetState = activeChannel,
                        transitionSpec = {
                            fadeIn(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) togetherWith 
                            fadeOut(animationSpec = spring(stiffness = Spring.StiffnessMediumLow))
                        },
                        modifier = Modifier.fillMaxSize(),
                        label = "MainViewTransition"
                    ) { targetChannel ->
                        if (targetChannel != null) {
                            Column(modifier = Modifier.fillMaxSize()) {
                                ChannelHeader(targetChannel, chatState)

                                Row(modifier = Modifier.fillMaxWidth().weight(1f)) {
                                    // Chat Area & Input Bar
                                    Column(
                                        modifier = Modifier
                                            .weight(1f)
                                            .fillMaxHeight()
                                            .background(MaterialTheme.colorScheme.surface)
                                    ) {
                                        Box(modifier = Modifier.weight(1f)) {
                                            if (targetChannel.type == 15 && selectedThread == null) {
                                                ForumPostList(chatState)
                                            } else {
                                                ChatArea(
                                                    modifier = Modifier.fillMaxSize(),
                                                    chatState = chatState
                                                )
                                            }
                                        }

                                        val isForumChannel = targetChannel.type == 15
                                        if (!isForumChannel) {
                                            ChatInputBar(targetChannel, chatState)
                                        }
                                    }

                                    // Member List
                                    val hasMemberList = targetChannel.guild_id != null && targetChannel.type != 15
                                    if (hasMemberList) {
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
                        } else {
                            Box(modifier = Modifier.fillMaxSize()) {
                                if (selectedGuild == null) {
                                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        Text("Select a friend to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    }
                                } else {
                                    // While loading guild channels
                                    ChatSkeleton()
                                }
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
            Box(contentAlignment = Alignment.CenterEnd, modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                // Global actions could go here (Help, Inbox, etc)
            }
        }

        // Account Panel overlays at the bottom START (overlaps both sidebars)
        Box(modifier = Modifier.align(Alignment.BottomStart).width(320.dp)) {
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

        // Settings Screen Overlay
        if (chatState.isSettingsVisible) {
            SettingsScreen(
                chatState = chatState,
                onDismiss = { chatState.isSettingsVisible = false }
            )
        }

        if (chatState.isQuickSwitcherVisible) {
            QuickSwitcher(
                chatState = chatState,
                onDismiss = { chatState.isQuickSwitcherVisible = false }
            )
        }
    }
}
