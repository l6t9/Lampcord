package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.SettingsScreen
import me.lampu.lampcord.shared.ui.components.AttachmentViewer
import me.lampu.lampcord.shared.ui.components.ChatArea
import me.lampu.lampcord.shared.ui.components.ChatInputBar
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.DiscordPanelValue
import me.lampu.lampcord.shared.ui.components.DiscordPanels
import me.lampu.lampcord.shared.ui.components.FriendsList
import me.lampu.lampcord.shared.ui.components.MemberList
import me.lampu.lampcord.shared.ui.components.Sidebar
import me.lampu.lampcord.shared.ui.components.VoiceArea
import me.lampu.lampcord.shared.ui.components.chat.PinnedMessagesScreen
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.guilds.ServerBottomSheet
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.rememberDiscordPanelsState
import me.lampu.lampcord.shared.ui.icons.Icons

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
                Sidebar(modifier = Modifier.systemBarsPadding())
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
                                        val isThread = navigationStore.selectedThread != null
                                        if (isThread) {
                                            IconButton(onClick = { navigationStore.selectedThread = null }) {
                                                Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                                            }
                                        } else if (activeChannel.type == 1 || activeChannel.type == 3 || activeChannel.guild_id == null) {
                                            IconButton(onClick = { panelState.openStart() }) {
                                                Icon(Icons.Filled.Menu, "Channels")
                                            }
                                        } else {
                                            val channelIcon = when (activeChannel.type) {
                                                15 -> Icons.Rounded.Forum
                                                2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                                                5 -> Icons.Filled.Campaign
                                                else -> Icons.Filled.Tag
                                            }
                                            Box(
                                                modifier = Modifier
                                                    .padding(start = 8.dp)
                                                    .size(36.dp)
                                                    .clip(RoundedCornerShape(10.dp))
                                                    .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                                                    .clickable { panelState.openStart() },
                                                contentAlignment = Alignment.Center
                                            ) {
                                                Icon(
                                                    imageVector = channelIcon,
                                                    contentDescription = "Channels",
                                                    modifier = Modifier.size(20.dp),
                                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                        }
                                    },
                                    actions = {
                                        if (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3) {
                                            IconButton(onClick = { navigationStore.isSearchVisible = true }) {
                                                Icon(
                                                    imageVector = Icons.Filled.Search,
                                                    contentDescription = "Search",
                                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                            if (activeChannel.type != 2 && activeChannel.type != 13) {
                                                IconButton(onClick = { navigationStore.isPinsVisible = true }) {
                                                    Icon(
                                                        imageVector = Icons.Filled.PushPin,
                                                        contentDescription = "Pins",
                                                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                    )
                                                }
                                            }
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
            val isProfileExpanded = profileStore.isProfileExpanded
            val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

            LaunchedEffect(isProfileExpanded) {
                if (isProfileExpanded) sheetState.expand()
            }

            ModalBottomSheet(
                onDismissRequest = {
                    profileStore.selectedProfile = null
                    profileStore.isProfileExpanded = false
                    profileStore.isProfileLoading = false
                },
                sheetState = sheetState,
                shape = if (isProfileExpanded) {
                    RoundedCornerShape(0.dp)
                } else {
                    RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp)
                },
                containerColor = if (isProfileExpanded) {
                    Color.Transparent
                } else {
                    MaterialTheme.colorScheme.surfaceContainerLow
                },
                contentWindowInsets = { WindowInsets(0, 0, 0, 0) },
                dragHandle = if (isProfileExpanded) {
                    {}
                } else {
                    {
                        Box(
                            modifier = Modifier
                                .padding(top = 8.dp, bottom = 8.dp)
                                .size(width = 36.dp, height = 4.dp)
                                .clip(RoundedCornerShape(2.dp))
                                .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
                        )
                    }
                }
            ) {
                if (profileStore.selectedProfile != null) {
                    ProfileCard(
                        profile = profileStore.selectedProfile!!,
                        modifier = Modifier.fillMaxWidth().wrapContentHeight(),
                        showBorder = false,
                        isExpanded = isProfileExpanded,
                        onExpand = { profileStore.isProfileExpanded = true }
                    )
                } else {
                    Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                        ContainedLoadingIndicator()
                    }
                }
                if (!isProfileExpanded) {
                    Spacer(Modifier.navigationBarsPadding())
                }
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
