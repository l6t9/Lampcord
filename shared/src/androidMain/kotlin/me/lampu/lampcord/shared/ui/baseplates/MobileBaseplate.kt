package me.lampu.lampcord.shared.ui.baseplates

import androidx.activity.compose.BackHandler
import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
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
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.SettingsScreen
import io.github.materiiapps.panels.SwipePanels
import io.github.materiiapps.panels.SwipePanelsValue
import io.github.materiiapps.panels.rememberSwipePanelsState

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun MobileBaseplate(chatState: ChatState) {
    val panelState = rememberDiscordPanelsState()
    val selectedChannel = chatState.selectedChannel
    val selectedThread = chatState.selectedThread
    val activeChannel = selectedThread ?: selectedChannel

    BackHandler(enabled = panelState.currentValue != DiscordPanelValue.Center) {
        panelState.close()
    }

    Box(Modifier.fillMaxSize()) {
        DiscordPanels(
            state = panelState,
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.surface),
            startPanel = {
                Sidebar(
                    chatState = chatState,
                    modifier = Modifier
                        .fillMaxSize()
                        .systemBarsPadding()
                        .padding(start = 6.dp)
                )
            },
            centerPanel = {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 2.dp
                ) {
                    Scaffold(
                        topBar = {
                            if (activeChannel != null) {
                                TopAppBar(
                                    title = {
                                        Text(activeChannel.name ?: "Chat", style = MaterialTheme.typography.titleMedium)
                                    },
                                    navigationIcon = {
                                        IconButton(onClick = { panelState.openStart() }) {
                                            Icon(Icons.Filled.Menu, "Channels")
                                        }
                                    },
                                    actions = {
                                        if (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3) {
                                            IconButton(onClick = { panelState.openEnd() }) {
                                                Icon(
                                                    imageVector = if (activeChannel.type == 1) Icons.Filled.Person else Icons.Filled.Group,
                                                    contentDescription = if (activeChannel.type == 1) "Profile" else "Members"
                                                )
                                            }
                                        }
                                    },
                                    colors = TopAppBarDefaults.topAppBarColors(
                                        containerColor = MaterialTheme.colorScheme.surface
                                    )
                                )
                            }
                        }
                    ) { padding ->
                        val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
                        val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

                        AnimatedContent(
                            targetState = if (activeChannel != null) activeChannel.id else if (chatState.isFriendsSelected) "friends" else "none",
                            transitionSpec = {
                                (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                    fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                                )
                            },
                            modifier = Modifier.padding(padding).fillMaxSize().background(MaterialTheme.colorScheme.surface),
                            label = "MainContentTransition"
                        ) { target ->
                            Box(Modifier.fillMaxSize()) {
                                if (activeChannel != null && target == activeChannel.id) {
                                    if (activeChannel.type == 2 || activeChannel.type == 13) {
                                        VoiceArea(activeChannel, chatState)
                                    } else {
                                        Column(modifier = Modifier.fillMaxSize()) {
                                            Box(modifier = Modifier.weight(1f)) {
                                                ChatArea(modifier = Modifier.fillMaxSize(), chatState = chatState)
                                            }
                                            ChatInputBar(activeChannel, chatState)
                                        }
                                    }
                                } else if (target == "friends") {
                                    FriendsList(chatState)
                                } else {
                                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                            Icon(Icons.Brand.Discord, null, modifier = Modifier.size(80.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.2f))
                                            Spacer(Modifier.height(24.dp))
                                            Text("Select a channel to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            Spacer(Modifier.height(24.dp))
                                            Button(onClick = { panelState.openStart() }) {
                                                Text("Open Drawer")
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if (panelState.currentValue != DiscordPanelValue.Center) {
                        Box(
                            modifier = Modifier
                                .zIndex(1f)
                                .fillMaxSize()
                                .clickable(
                                    interactionSource = remember { MutableInteractionSource() },
                                    indication = null,
                                    onClick = { panelState.close() },
                                )
                        )
                    }
                }
            },
            endPanel = {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .systemBarsPadding()
                        .padding(end = 6.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Surface(
                        modifier = Modifier.weight(1f),
                        shape = MaterialTheme.shapes.medium,
                        tonalElevation = 1.dp
                    ) {
                        if (activeChannel?.type == 1) {
                            val profile = chatState.sidebarProfile
                            if (profile != null) {
                                ProfileCard(
                                    profile = profile,
                                    chatState = chatState,
                                    showBorder = true,
                                    isSidebar = true,
                                    showMemberSince = true,
                                    modifier = Modifier.fillMaxSize(),
                                    onExpand = { 
                                        chatState.showProfile(profile.user.id)
                                        panelState.close()
                                    }
                                )
                            } else if (chatState.isSidebarProfileLoading) {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    ContainedLoadingIndicator()
                                }
                            } else {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    Text("Profile not loaded", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                        } else {
                            MemberList(chatState)
                        }
                    }
                    
                    if (activeChannel?.guild_id != null || activeChannel?.type == 3) {
                        NavButtonRow(
                            listOf(
                                NavButtonData(
                                    icon = Icons.Filled.Group,
                                    title = "Members",
                                    selected = chatState.isFriendsSelected,
                                    onClick = {
                                        chatState.selectedGuild = null
                                        chatState.selectedChannel = null
                                        chatState.isFriendsSelected = true
                                        panelState.close()
                                    }
                                ),
                                NavButtonData(
                                    icon = Icons.Filled.Search,
                                    title = "Search",
                                    onClick = { chatState.isQuickSwitcherVisible = true }
                                ),
                                NavButtonData(
                                    icon = Icons.Outlined.AlternateEmail,
                                    title = "Mentions",
                                    onClick = { /* Mentions */ }
                                )
                            )
                        )
                    }
                }
            }
        )

        if (chatState.isAttachmentViewerVisible) {
            AttachmentViewer(
                items = chatState.attachmentViewerItems,
                selectedIndex = chatState.attachmentViewerIndex,
                onIndexChange = { chatState.attachmentViewerIndex = it },
                onDismiss = { chatState.closeAttachmentViewer() }
            )
        }

        // User Profile Sheet
        if (chatState.isProfileLoading || chatState.selectedProfile != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    chatState.selectedProfile = null
                    chatState.isProfileLoading = false
                },
                shape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp),
                containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                dragHandle = {
                    Box(
                        modifier = Modifier
                            .padding(vertical = 12.dp)
                            .size(width = 40.dp, height = 4.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
                    )
                }
            ) {
                if (chatState.selectedProfile != null) {
                    ProfileCard(
                        profile = chatState.selectedProfile!!,
                        chatState = chatState,
                        modifier = Modifier.fillMaxWidth().wrapContentHeight(),
                        showBorder = false,
                        isExpanded = true
                    )
                } else {
                    Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                        ContainedLoadingIndicator()
                    }
                }
                Spacer(Modifier.navigationBarsPadding().height(16.dp))
            }
        }

        // Settings Screen
        if (chatState.isSettingsVisible) {
            SettingsScreen(chatState, onDismiss = { chatState.isSettingsVisible = false })
        }
    }
}
