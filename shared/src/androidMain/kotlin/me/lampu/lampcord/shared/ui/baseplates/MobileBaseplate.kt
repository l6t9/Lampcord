package me.lampu.lampcord.shared.ui.baseplates

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberModalBottomSheetState
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
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.ui.NavDisplay
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.SettingsScreen
import me.lampu.lampcord.shared.ui.components.AttachmentViewer
import me.lampu.lampcord.shared.ui.components.AdaptiveModalBottomSheet
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.ChatArea
import me.lampu.lampcord.shared.ui.components.ChatInputBar
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.DiscordPanelValue
import me.lampu.lampcord.shared.ui.components.DiscordPanels
import me.lampu.lampcord.shared.ui.components.FriendsList
import me.lampu.lampcord.shared.ui.components.MemberList
import me.lampu.lampcord.shared.ui.components.Sidebar
import me.lampu.lampcord.shared.ui.components.VoiceArea
import me.lampu.lampcord.shared.ui.components.QuickSwitcher
import me.lampu.lampcord.shared.ui.components.EmojiPicker
import me.lampu.lampcord.shared.ui.components.chat.PinnedMessagesScreen
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.guilds.ServerBottomSheet
import me.lampu.lampcord.shared.ui.components.chat.ChannelSettingsScreen
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.rememberDiscordPanelsState
import me.lampu.lampcord.shared.ui.navigation.Navigator
import me.lampu.lampcord.shared.ui.navigation.Screen
import me.lampu.lampcord.shared.ui.navigation.rememberNavigationState
import me.lampu.lampcord.shared.ui.navigation.toEntries
import me.lampu.lampcord.shared.ui.navigation.entry
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun MobileBaseplate(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    userStore: UserStore
) {
    val presenceStore: PresenceStore = koinInject()
    val settingsStore: SettingsStore = koinInject()
    val allUsers by userStore.users.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()

    val initialStartRoute = remember { 
        if (navigationStore.isFriendsSelected) Screen.Friends else Screen.Chat
    }
    val topLevelRoutes = remember { 
        setOf(
            Screen.Chat,
            Screen.Friends, 
            Screen.Settings, 
            Screen.ServerSettings, 
            Screen.ChannelSettings,
            Screen.Search,
            Screen.Pins,
            Screen.ChannelsAndRoles
        ) 
    }
    val navigationState = rememberNavigationState(
        startRoute = initialStartRoute,
        topLevelRoutes = topLevelRoutes
    )
    val navigator = remember { Navigator(navigationState) }

    val panelState = rememberDiscordPanelsState()
    val keyboardController = LocalSoftwareKeyboardController.current

    // Sync navigationStore visibility states with navigator
    LaunchedEffect(navigationStore.isSettingsVisible) {
        if (navigationStore.isSettingsVisible) navigator.navigate(Screen.Settings)
        else if (navigationState.topLevelRoute == Screen.Settings) navigator.goBack()
    }
    LaunchedEffect(navigationStore.isServerSettingsVisible) {
        if (navigationStore.isServerSettingsVisible) navigator.navigate(Screen.ServerSettings)
        else if (navigationState.topLevelRoute == Screen.ServerSettings) navigator.goBack()
    }
    LaunchedEffect(navigationStore.channelSettingsChannel) {
        if (navigationStore.channelSettingsChannel != null) navigator.navigate(Screen.ChannelSettings)
        else if (navigationState.topLevelRoute == Screen.ChannelSettings) navigator.goBack()
    }
    LaunchedEffect(navigationStore.isSearchVisible) {
        if (navigationStore.isSearchVisible) navigator.navigate(Screen.Search)
        else if (navigationState.topLevelRoute == Screen.Search) navigator.goBack()
    }
    LaunchedEffect(navigationStore.isPinsVisible) {
        if (navigationStore.isPinsVisible) navigator.navigate(Screen.Pins)
        else if (navigationState.topLevelRoute == Screen.Pins) navigator.goBack()
    }
    LaunchedEffect(navigationStore.isChannelsAndRolesVisible) {
        if (navigationStore.isChannelsAndRolesVisible) navigator.navigate(Screen.ChannelsAndRoles)
        else if (navigationState.topLevelRoute == Screen.ChannelsAndRoles) navigator.goBack()
    }

    LaunchedEffect(navigationStore.selectedGuild?.id, navigationStore.isFriendsSelected) {
        if (navigationStore.selectedGuild != null) {
            navigator.navigate(Screen.Chat)
        } else if (navigationStore.isFriendsSelected) {
            navigator.navigate(Screen.Friends)
        }
    }

    // Sync navigator back to navigationStore
    LaunchedEffect(navigationState.topLevelRoute) {
        val route = navigationState.topLevelRoute
        if (route !is Screen.Settings && navigationStore.isSettingsVisible) navigationStore.isSettingsVisible = false
        if (route !is Screen.ServerSettings && navigationStore.isServerSettingsVisible) navigationStore.isServerSettingsVisible = false
        if (route !is Screen.ChannelSettings && navigationStore.channelSettingsChannel != null) navigationStore.closeChannelSettings()
        if (route !is Screen.Search && navigationStore.isSearchVisible) navigationStore.isSearchVisible = false
        if (route !is Screen.Pins && navigationStore.isPinsVisible) navigationStore.isPinsVisible = false
        if (route !is Screen.ChannelsAndRoles && navigationStore.isChannelsAndRolesVisible) navigationStore.isChannelsAndRolesVisible = false
    }

    LaunchedEffect(Unit) {
        navigationStore.focusChatRequest.collect {
            panelState.close()
        }
    }

    LaunchedEffect(panelState.currentValue) {
        if (panelState.currentValue != DiscordPanelValue.Center) {
            keyboardController?.hide()
        }
    }

    val entries = navigationState.toEntries(
        entryProvider {
            entry<Screen.Chat> {
                MainBaseplateContent(
                    navigationStore,
                    profileStore,
                    presenceStore,
                    settingsStore,
                    allUsers,
                    currentUser,
                    panelState,
                    navigationStore.selectedThread ?: navigationStore.selectedChannel,
                    isActive = navigationState.topLevelRoute == Screen.Chat
                )
            }
            entry<Screen.Friends> {
                MainBaseplateContent(
                    navigationStore,
                    profileStore,
                    presenceStore,
                    settingsStore,
                    allUsers,
                    currentUser,
                    panelState,
                    navigationStore.selectedThread ?: navigationStore.selectedChannel,
                    isActive = navigationState.topLevelRoute == Screen.Friends
                )
            }
            entry<Screen.Settings> {
                SettingsScreen(onDismiss = { navigationStore.isSettingsVisible = false })
            }
            entry<Screen.ServerSettings> {
                ServerSettings(onDismiss = { navigationStore.isServerSettingsVisible = false })
            }
            entry<Screen.ChannelSettings> {
                ChannelSettingsScreen(onDismiss = { navigationStore.closeChannelSettings() })
            }
            entry<Screen.Search> {
                SearchScreen(onDismiss = { navigationStore.isSearchVisible = false })
            }
            entry<Screen.Pins> {
                PinnedMessagesScreen(onDismiss = { navigationStore.isPinsVisible = false })
            }
            entry<Screen.ChannelsAndRoles> {
                val guild = navigationStore.selectedGuild
                Scaffold(
                    topBar = {
                        TopAppBar(
                            title = {
                                Column {
                                    Text("Channels & Roles", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                                    guild?.name?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                            },
                            navigationIcon = {
                                IconButton(onClick = { navigationStore.isChannelsAndRolesVisible = false }) {
                                    Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                                }
                            }
                        )
                    }
                ) { padding ->
                    Box(Modifier.padding(padding)) {
                        me.lampu.lampcord.shared.ui.components.guilds.ChannelsAndRoles()
                    }
                }
            }
        }
    )

    NavDisplay(
        entries = entries,
        onBack = {
            if (!navigator.goBack()) {
                // Exit app or go to home if not on start route
            }
        }
    )

    // Global Overlays (non-backstack)
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
        val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = false)

        LaunchedEffect(Unit) {
            sheetState.expand()
        }

        AdaptiveModalBottomSheet(
            onDismissRequest = {
                profileStore.selectedProfile = null
                profileStore.isProfileExpanded = false
                profileStore.isProfileLoading = false
            },
            sheetState = sheetState,
        ) {
            if (profileStore.selectedProfile != null) {
                ProfileCard(
                    profile = profileStore.selectedProfile!!,
                    modifier = Modifier.fillMaxWidth().wrapContentHeight(),
                    showBorder = false,
                    isExpanded = true,
                    onExpand = null,
                    onDismiss = {
                        profileStore.selectedProfile = null
                        profileStore.isProfileExpanded = false
                        profileStore.isProfileLoading = false
                    }
                )
            } else {
                Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            }
        }
    }

    // Server Menu Bottom Sheet
    if (navigationStore.isServerMenuVisible) {
        navigationStore.selectedGuild?.let { guild ->
            ServerBottomSheet(guild, onDismiss = { navigationStore.isServerMenuVisible = false })
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun MainBaseplateContent(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    presenceStore: PresenceStore,
    settingsStore: SettingsStore,
    allUsers: Map<String, me.lampu.lampcord.shared.model.User>,
    currentUser: me.lampu.lampcord.shared.model.User?,
    panelState: me.lampu.lampcord.shared.ui.components.DiscordPanelsState,
    activeChannel: me.lampu.lampcord.shared.model.Channel?,
    isActive: Boolean
) {
    // Ensure we recompose when these change
    key(navigationStore.selectedGuild?.id, navigationStore.isFriendsSelected) {
        Box(Modifier.fillMaxSize()) {
            val swipeEnabled = !navigationStore.isBubble && me.lampu.lampcord.shared.settings.Settings.shared.chatGestures == me.lampu.lampcord.shared.settings.ChatGestures.SWIPE_TO_MEMBERS
            DiscordPanels(
                state = panelState,
                modifier = Modifier
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.surfaceContainerHigh), // Guild Rail background for status bar
                swipeEnabled = swipeEnabled,
                startPanel = {
                    Sidebar(modifier = Modifier.systemBarsPadding().padding(start = 8.dp, top = 8.dp, bottom = 8.dp))
                },
                centerPanel = {
                    Surface(
                        modifier = Modifier.fillMaxSize(),
                        color = MaterialTheme.colorScheme.background, // Chat background
                        tonalElevation = 0.dp
                    ) {
                        Scaffold(
                            contentWindowInsets = WindowInsets(0, 0, 0, 0),
                            topBar = {
                                if (activeChannel != null || navigationStore.isChannelsAndRolesVisible || navigationStore.isFriendsSelected) {
                                    TopAppBar(
                                        windowInsets = TopAppBarDefaults.windowInsets.union(WindowInsets.statusBars),
                                        title = {
                                            Row(verticalAlignment = Alignment.CenterVertically) {
                                                if (navigationStore.isChannelsAndRolesVisible) {
                                                    Text(
                                                        text = "Browse Channels",
                                                        style = MaterialTheme.typography.titleMedium,
                                                        fontWeight = FontWeight.Bold
                                                    )
                                                } else if (activeChannel != null) {
                                                    if (activeChannel.type == 1) {
                                                        val recipientId = activeChannel.recipients?.firstOrNull()?.id ?: activeChannel.recipient_ids?.firstOrNull()
                                                        val recipient = recipientId?.let { allUsers[it] } ?: activeChannel.recipients?.firstOrNull()
                                                        
                                                        if (recipient != null) {
                                                            Box(modifier = Modifier.size(24.dp)) {
                                                                AvatarWithDecoration(
                                                                    avatarUrl = recipient.avatar?.let { "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png?size=64" },
                                                                    decorationData = recipient.avatar_decoration_data,
                                                                    size = 24.dp,
                                                                    status = presenceStore.getUserStatus(recipient.id, currentUser?.id, settingsStore.userSettings?.status)
                                                                )
                                                            }
                                                            Spacer(Modifier.width(12.dp))
                                                        }
                                                    }

                                                    Column {
                                                        Text(
                                                            text = if (activeChannel.type == 1) {
                                                                val recipientId = activeChannel.recipients?.firstOrNull()?.id ?: activeChannel.recipient_ids?.firstOrNull()
                                                                val recipient = recipientId?.let { allUsers[it] } ?: activeChannel.recipients?.firstOrNull()
                                                                recipient?.let { it.global_name ?: it.username } ?: "Chat"
                                                            } else activeChannel.name ?: "Chat",
                                                            style = MaterialTheme.typography.titleMedium,
                                                            fontWeight = FontWeight.Bold,
                                                            maxLines = 1,
                                                            overflow = TextOverflow.Ellipsis,
                                                            modifier = Modifier.basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp)
                                                        )
                                                        if (activeChannel.topic?.isNotBlank() == true) {
                                                            Text(
                                                                text = activeChannel.topic,
                                                                style = MaterialTheme.typography.labelSmall,
                                                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                                maxLines = 1,
                                                                overflow = TextOverflow.Ellipsis,
                                                                modifier = Modifier.basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp)
                                                            )
                                                        }
                                                    }
                                                } else if (navigationStore.isFriendsSelected) {
                                                    Text(
                                                        text = "Friends",
                                                        style = MaterialTheme.typography.titleMedium,
                                                        fontWeight = FontWeight.Bold
                                                    )
                                                }
                                            }
                                        },
                                        navigationIcon = {
                                            if (!navigationStore.isBubble) {
                                                val isThread = navigationStore.selectedThread != null
                                                val isRoles = navigationStore.isChannelsAndRolesVisible
                                                if (isThread) {
                                                    IconButton(onClick = {
                                                        navigationStore.selectedThread = null
                                                    }) {
                                                        Icon(
                                                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                                            contentDescription = "Back"
                                                        )
                                                    }
                                                } else if (isRoles) {
                                                    IconButton(onClick = {
                                                        navigationStore.isChannelsAndRolesVisible = false
                                                    }) {
                                                        Icon(
                                                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                                            contentDescription = "Back"
                                                        )
                                                    }
                                                } else if (activeChannel != null && activeChannel.type != 1 && activeChannel.type != 3 && activeChannel.guild_id != null) {
                                                    val channelIcon = when (activeChannel.type) {
                                                        15 -> Icons.Rounded.Forum
                                                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                                                        5 -> Icons.Filled.Campaign
                                                        else -> Icons.Filled.Tag
                                                    }
                                                    Box(
                                                        modifier = Modifier
                                                            .padding(start = 4.dp)
                                                            .size(40.dp)
                                                            .clickable(
                                                                interactionSource = remember { MutableInteractionSource() },
                                                                indication = null
                                                            ) { panelState.openStart() },
                                                        contentAlignment = Alignment.Center
                                                    ) {
                                                        Icon(
                                                            imageVector = channelIcon,
                                                            contentDescription = "Channels",
                                                            modifier = Modifier.size(22.dp),
                                                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                        )
                                                    }
                                                } else {
                                                    IconButton(onClick = { panelState.openStart() }) {
                                                        Icon(
                                                            imageVector = Icons.Filled.Menu,
                                                            contentDescription = "Menu"
                                                        )
                                                    }
                                                }
                                            }
                                        },
                                        actions = {
                                            if (!navigationStore.isBubble && activeChannel != null && (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3)) {
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
                                            }
                                        },
                                        colors = TopAppBarDefaults.topAppBarColors(
                                            containerColor = MaterialTheme.colorScheme.surfaceContainerLowest // Discord Dark Header
                                        )
                                    )
                                }
                            }
                        ) { padding ->
                            val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
                            val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

                            AnimatedContent(
                                targetState = activeChannel?.id
                                    ?: if (navigationStore.isChannelsAndRolesVisible) "roles" else if (navigationStore.isFriendsSelected) "friends" else "none",
                                transitionSpec = {
                                    (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                        fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                                    )
                                },
                                modifier = Modifier
                                    .padding(top = padding.calculateTopPadding())
                                    .fillMaxSize()
                                    .background(MaterialTheme.colorScheme.background), // Chat background
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
                                    } else if (target == "roles") {
                                        me.lampu.lampcord.shared.ui.components.guilds.ChannelsAndRoles()
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
                            .padding(end = 8.dp, top = 8.dp, bottom = 8.dp),
                        verticalArrangement = Arrangement.Top
                    ) {
                        Surface(
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(16.dp),
                            color = MaterialTheme.colorScheme.background,
                            tonalElevation = 0.dp
                        ) {
                            MemberList()
                        }
                    }
                }
            )

            // 3. Panels back handler integration
            BackHandler(enabled = isActive && panelState.currentValue == DiscordPanelValue.End) {
                panelState.close()
            }

            BackHandler(enabled = isActive && panelState.currentValue == DiscordPanelValue.Center) {
                if (navigationStore.selectedThread != null) {
                    navigationStore.selectedThread = null
                } else if (navigationStore.isChannelsAndRolesVisible) {
                    navigationStore.isChannelsAndRolesVisible = false
                } else {
                    panelState.openStart()
                }
            }
        }
    }
}
