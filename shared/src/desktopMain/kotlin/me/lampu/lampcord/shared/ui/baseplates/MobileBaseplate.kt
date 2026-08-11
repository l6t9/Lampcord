package me.lampu.lampcord.shared.ui.baseplates

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
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.compose.ui.input.key.*
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.guilds.ServerBottomSheet
import me.lampu.lampcord.shared.ui.components.chat.PinnedMessagesScreen
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.SettingsScreen
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun MobileBaseplate(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    userStore: UserStore
) {
    val panelState = rememberDiscordPanelsState()
    val selectedChannel = navigationStore.selectedChannel
    val selectedThread = navigationStore.selectedThread
    val activeChannel = selectedThread ?: selectedChannel
    val keyboardController = LocalSoftwareKeyboardController.current

    LaunchedEffect(panelState.currentValue) {
        if (panelState.currentValue != DiscordPanelValue.Center) {
            keyboardController?.hide()
        }
    }

    Box(
        Modifier
            .fillMaxSize()
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown && event.key == Key.Escape) {
                    if (panelState.currentValue != DiscordPanelValue.Center) {
                        panelState.close()
                        return@onPreviewKeyEvent true
                    }
                }
                false
            }
    ) {
        val swipeEnabled = me.lampu.lampcord.shared.settings.Settings.shared.chatGestures == me.lampu.lampcord.shared.settings.ChatGestures.SWIPE_TO_MEMBERS
        DiscordPanels(
            state = panelState,
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.surface),
            swipeEnabled = swipeEnabled,
            startPanel = {
                Sidebar()
            },
            centerPanel = {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 2.dp
                ) {
                    Scaffold(
                        contentWindowInsets = WindowInsets(0, 0, 0, 0),
                        topBar = {
                            if (activeChannel != null) {
                                TopAppBar(
                                    windowInsets = TopAppBarDefaults.windowInsets.union(WindowInsets.statusBars),
                                    title = {
                                        Column {
                                            Text(
                                                text = if (activeChannel.type == 1) {
                                                    val recipient = activeChannel.recipients?.firstOrNull()
                                                    recipient?.let { it.global_name ?: it.username } ?: "Chat"
                                                } else activeChannel.name ?: "Chat",
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = FontWeight.Bold
                                            )
                                            if (activeChannel.topic?.isNotBlank() == true) {
                                                Text(
                                                    text = activeChannel.topic,
                                                    style = MaterialTheme.typography.labelSmall,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis
                                                )
                                            }
                                        }
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
                            targetState = if (activeChannel != null) activeChannel.id else if (navigationStore.isFriendsSelected) "friends" else "none",
                            transitionSpec = {
                                (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                    fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                                )
                            },
                            modifier = Modifier
                                .padding(top = padding.calculateTopPadding())
                                .fillMaxSize()
                                .background(MaterialTheme.colorScheme.surface),
                            label = "MainContentTransition"
                        ) { target ->
                            Box(Modifier.fillMaxSize()) {
                                if (activeChannel != null && target == activeChannel.id) {
                                    if (activeChannel.type == 2 || activeChannel.type == 13) {
                                        VoiceArea(activeChannel)
                                    } else {
                                        Column(
                                            modifier = Modifier
                                                .fillMaxSize()
                                        ) {
                                            Box(modifier = Modifier.weight(1f)) {
                                                ChatArea(modifier = Modifier.fillMaxSize())
                                            }
                                            ChatInputBar(activeChannel)
                                        }
                                    }
                                } else if (target == "friends") {
                                    FriendsList()
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
                            val profile = profileStore.sidebarProfile
                            if (profile != null) {
                                ProfileCard(
                                    profile = profile,
                                    showBorder = true,
                                    isSidebar = true,
                                    showMemberSince = true,
                                    modifier = Modifier.fillMaxSize(),
                                    onExpand = { 
                                        profileStore.showProfile(profile.user.id, navigationStore.selectedGuild?.id)
                                        panelState.close()
                                    }
                                )
                            } else if (profileStore.isSidebarProfileLoading) {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    ContainedLoadingIndicator()
                                }
                            } else {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    Text("Profile not loaded", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                        } else {
                            MemberList(
                                header = {
                                    activeChannel?.let {
                                        MemberHeader(it)
                                    }
                                }
                            )
                        }
                    }
                }
            }
        )

        if (navigationStore.isAttachmentViewerVisible) {
            AttachmentViewer(
                items = navigationStore.attachmentViewerItems,
                selectedIndex = navigationStore.attachmentViewerIndex,
                onIndexChange = { navigationStore.attachmentViewerIndex = it },
                onDismiss = { navigationStore.closeAttachmentViewer() }
            )
        }

        // User Profile Sheet
        if (profileStore.isProfileLoading || profileStore.selectedProfile != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    profileStore.selectedProfile = null
                    profileStore.isProfileLoading = false
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
                if (profileStore.selectedProfile != null) {
                    ProfileCard(
                        profile = profileStore.selectedProfile!!,
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
        if (navigationStore.isSettingsVisible) {
            SettingsScreen(onDismiss = { navigationStore.isSettingsVisible = false })
        }

        // Server Settings
        if (navigationStore.isServerSettingsVisible) {
            ServerSettings(onDismiss = { navigationStore.isServerSettingsVisible = false })
        }

        // Server Menu Bottom Sheet
        if (navigationStore.isServerMenuVisible) {
            navigationStore.selectedGuild?.let { guild ->
                ServerBottomSheet(guild, onDismiss = { navigationStore.isServerMenuVisible = false })
            }
        }

        // Pinned Messages
        if (navigationStore.isPinsVisible) {
            PinnedMessagesScreen(onDismiss = { navigationStore.isPinsVisible = false })
        }

        // Search Screen
        if (navigationStore.isSearchVisible) {
            SearchScreen(onDismiss = { navigationStore.isSearchVisible = false })
        }
    }
}
