package me.lampu.lampcord.shared.ui.baseplates

import me.lampu.lampcord.shared.api.CdnUrls
import androidx.activity.compose.BackHandler
import androidx.compose.animation.*
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.Spring
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.ui.NavDisplay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.SettingsScreen
import me.lampu.lampcord.shared.ui.AboutContent
import me.lampu.lampcord.shared.ui.settings.*
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.components.ForumPostList
import me.lampu.lampcord.shared.ui.components.GlobalSnackbarHost
import me.lampu.lampcord.shared.ui.components.chat.ChannelSettingsScreen
import me.lampu.lampcord.shared.ui.components.chat.PinnedMessagesScreen
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.guilds.ServerBottomSheet
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import androidx.compose.foundation.shape.CircleShape
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCardSkeleton
import me.lampu.lampcord.shared.settings.PanelType
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.navigation.Navigator
import me.lampu.lampcord.shared.ui.navigation.Screen
import me.lampu.lampcord.shared.ui.navigation.rememberNavigationState
import me.lampu.lampcord.shared.ui.navigation.toEntries
import me.lampu.lampcord.shared.ui.navigation.entry
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun MobileBaseplate(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    userStore: UserStore,
    voiceStore: VoiceStore
) {
    val presenceStore: PresenceStore = koinInject()
    val settingsStore: SettingsStore = koinInject()
    val tokenStore: TokenStore = koinInject()
    val allUsers by userStore.users.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()

    val scope = rememberCoroutineScope()
    val haptic = LocalHapticFeedback.current
    var showUserStatusSheet by remember { mutableStateOf(false) }
    var showCustomStatusDialog by remember { mutableStateOf(false) }
    var showAddAccountDialog by remember { mutableStateOf(false) }

    val initialStartRoute = remember { 
        if (navigationStore.isFriendsSelected) Screen.Friends else Screen.Chat
    }
    val topLevelRoutes = remember { 
        setOf(
            Screen.Chat,
            Screen.Friends, 
            Screen.Mentions,
            Screen.Search,
            Screen.GlobalSearch,
            Screen.Settings, 
            Screen.ServerSettings, 
            Screen.ChannelSettings,
            Screen.Pins,
            Screen.ChannelsAndRoles,
            Screen.EasterEgg
        ) 
    }
    val navigationState = rememberNavigationState(
        startRoute = initialStartRoute,
        topLevelRoutes = topLevelRoutes
    )
    val navigator = remember { Navigator(navigationState) }

    val panelState = rememberDiscordPanelsState()
    var lastPanelValue by remember { mutableStateOf(DiscordPanelValue.Center) }
    val keyboardController = LocalSoftwareKeyboardController.current

    val routeIndexMap = remember {
        mapOf<Any, Int>(
            Screen.Chat to 0,
            Screen.Friends to 1,
            Screen.GlobalSearch to 2,
            Screen.Mentions to 3,
            Screen.Settings to 4,
            Screen.Search to 5,
            Screen.Pins to 6,
            Screen.Threads::class to 7,
            Screen.ChannelsAndRoles to 8,
            Screen.ServerSettings to 9,
            Screen.ChannelSettings to 10,
            Screen.Theming to 11,
            Screen.ThemeEditor::class to 12,
        )
    }
    val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
    val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

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
    LaunchedEffect(navigationStore.isNotificationsSettingsVisible) {
        if (navigationStore.isNotificationsSettingsVisible) navigator.navigate(Screen.NotificationsSettings)
        else if (navigationState.topLevelRoute == Screen.NotificationsSettings) navigator.goBack()
    }
    LaunchedEffect(navigationStore.isMentionsSelected) {
        if (navigationStore.isMentionsSelected) navigator.navigate(Screen.Mentions)
        else if (navigationState.topLevelRoute == Screen.Mentions) navigator.goBack()
    }

    LaunchedEffect(navigationStore.selectedGuild?.id, navigationStore.isFriendsSelected, navigationStore.isMentionsSelected) {
        if (navigationStore.isFriendsSelected) {
            navigator.navigate(Screen.Friends)
        } else if (navigationStore.isMentionsSelected) {
            navigator.navigate(Screen.Mentions)
        } else if (navigationStore.selectedGuild != null) {
            navigator.navigate(Screen.Chat)
        }
    }

    LaunchedEffect(navigationStore.isThreadPanelVisible) {
        if (navigationStore.isThreadPanelVisible) {
            val channel = navigationStore.selectedChannel
            if (channel != null) {
                navigator.navigate(Screen.Threads(channel.id))
            }
        } else if (navigationState.topLevelRoute is Screen.Threads) {
            navigator.goBack()
        }
    }

    LaunchedEffect(navigationState.topLevelRoute) {
        val route = navigationState.topLevelRoute
        
        if (route == Screen.Chat) {
            panelState.currentValue = lastPanelValue
        } else {
            panelState.close()
        }

        if (route !is Screen.Settings && navigationStore.isSettingsVisible) navigationStore.isSettingsVisible = false
        if (route !is Screen.ServerSettings && navigationStore.isServerSettingsVisible) navigationStore.isServerSettingsVisible = false
        if (route !is Screen.ChannelSettings && navigationStore.channelSettingsChannel != null) navigationStore.closeChannelSettings()
        if (route !is Screen.Search && navigationStore.isSearchVisible) navigationStore.isSearchVisible = false
        if (route !is Screen.Pins && navigationStore.isPinsVisible) navigationStore.isPinsVisible = false
        if (route !is Screen.Threads && navigationStore.isThreadPanelVisible) navigationStore.isThreadPanelVisible = false
        if (route !is Screen.ChannelsAndRoles && navigationStore.isChannelsAndRolesVisible) navigationStore.isChannelsAndRolesVisible = false
        if (route !is Screen.ChannelsAndRoles && navigationStore.isChannelsAndRolesVisible) navigationStore.isChannelsAndRolesVisible = false
        if (route !is Screen.NotificationsSettings && navigationStore.isNotificationsSettingsVisible) navigationStore.isNotificationsSettingsVisible = false

        when (route) {
            is Screen.Friends -> {
                navigationStore.isFriendsSelected = true
                navigationStore.isMentionsSelected = false
            }
            is Screen.Mentions -> {
                navigationStore.isFriendsSelected = false
                navigationStore.isMentionsSelected = true
            }
            is Screen.Chat -> {
                navigationStore.isFriendsSelected = false
                navigationStore.isMentionsSelected = false
            }
            else -> {}
        }
    }

    LaunchedEffect(Unit) {
        navigationStore.focusChatRequest.collect {
            panelState.close()
        }
    }

    LaunchedEffect(panelState.currentValue, navigationState.topLevelRoute) {
        if (navigationState.topLevelRoute == Screen.Chat) {
            lastPanelValue = panelState.currentValue
        }
        if (panelState.currentValue != DiscordPanelValue.Center) {
            keyboardController?.hide()
        }
    }

    val currentRoute = navigationState.topLevelRoute
    val isChat = currentRoute == Screen.Chat
    val isSettingsRoot = currentRoute == Screen.Settings &&
        navigationState.backStacks[Screen.Settings]?.lastOrNull() == Screen.Settings
    val isSettingsRoute = currentRoute == Screen.Settings ||
        currentRoute == Screen.AccountSettings ||
        currentRoute == Screen.ProfilesSettings ||
        currentRoute == Screen.AppearanceSettings ||
        currentRoute == Screen.AccessibilitySettings ||
        currentRoute == Screen.PrivacySettings ||
        currentRoute == Screen.ConnectionsSettings ||
        currentRoute == Screen.DevicesSettings ||
        currentRoute == Screen.ChatSettings ||
        currentRoute == Screen.NotificationsSettings ||
        currentRoute == Screen.NavigationSettings ||
        currentRoute == Screen.AdvancedSettings ||
        currentRoute == Screen.AboutSettings
    val isTabRoute = currentRoute in setOf(
        Screen.Chat, Screen.Friends, Screen.Mentions, Screen.GlobalSearch
    )
    val showSettingsTabBar = currentRoute == Screen.Settings && isSettingsRoot

    val targetNavBarVisibleAmount = remember(panelState.progress, panelState.currentValue, currentRoute, isTabRoute, showSettingsTabBar) {
        val progress = panelState.progress
        if (isTabRoute || showSettingsTabBar) {
            if (currentRoute == Screen.Chat) {
                if (panelState.currentValue == DiscordPanelValue.Start) {
                    1f
                } else {
                    progress.coerceIn(0f, 1f)
                }
            } else {
                (1f + progress).coerceIn(0f, 1f)
            }
        } else {
            0f
        }
    }

    val navBarVisibleAmount by animateFloatAsState(
        targetValue = targetNavBarVisibleAmount,
        animationSpec = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
            androidx.compose.animation.core.snap()
        } else {
            MaterialTheme.motionScheme.defaultSpatialSpec<Float>()
        },
        label = "NavBarVisibleAmount"
    )

    val bottomInset = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    val navBarHeight = 80.dp + bottomInset

    val entries = navigationState.toEntries(
        entryProvider {
            entry<Screen.Chat> {
                Box(Modifier.fillMaxSize()) {
                    MainBaseplateContent(
                        navigationStore,
                        profileStore,
                        presenceStore,
                        settingsStore,
                        voiceStore,
                        allUsers,
                        currentUser,
                        panelState,
                        navigationStore.selectedThread ?: navigationStore.selectedChannel
                    )
                }
            }
            entry<Screen.Friends> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    FriendsList()
                }
            }
            entry<Screen.Mentions> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    me.lampu.lampcord.shared.ui.components.chat.MentionsScreen()
                }
            }
            entry<Screen.Search> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    SearchScreen(onDismiss = { navigationStore.isSearchVisible = false })
                }
            }
            entry<Screen.GlobalSearch> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    GlobalSearchScreen(onDismiss = { navigator.goBack() })
                }
            }
            entry<Screen.Settings> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    SettingsScreen(
                        onNavigateToAccount = { navigator.navigate(Screen.AccountSettings) },
                        onNavigateToProfiles = { navigator.navigate(Screen.ProfilesSettings) },
                        onNavigateToPrivacy = { navigator.navigate(Screen.PrivacySettings) },
                        onNavigateToConnections = { navigator.navigate(Screen.ConnectionsSettings) },
                        onNavigateToDevices = { navigator.navigate(Screen.DevicesSettings) },
                        onNavigateToAppearance = { navigator.navigate(Screen.AppearanceSettings) },
                        onNavigateToAccessibility = { navigator.navigate(Screen.AccessibilitySettings) },
                        onNavigateToChat = { navigator.navigate(Screen.ChatSettings) },
                        onNavigateToNotifications = { navigator.navigate(Screen.NotificationsSettings) },
                        onNavigateToAdvanced = { navigator.navigate(Screen.AdvancedSettings) },
                        onNavigateToAbout = { navigator.navigate(Screen.AboutSettings) },
                        onNavigateToTheming = { navigator.navigate(Screen.Theming) },
                        onNavigateToNavigation = { navigator.navigate(Screen.NavigationSettings) },
                        onDismiss = { navigationStore.isSettingsVisible = false }
                    )
                }
            }
            entry<Screen.AccountSettings> {
                val userStore: UserStore = koinInject()
                AccountSettings(onBack = { navigator.goBack() }, userStore = userStore)
            }
            entry<Screen.ProfilesSettings> {
                val userStore: UserStore = koinInject()
                ProfilesSettings(onBack = { navigator.goBack() }, userStore = userStore)
            }
            entry<Screen.AppearanceSettings> {
                AppearanceSettings(onNavigateToTheming = { navigator.navigate(Screen.Theming) }, onNavigateToNavigation = { navigator.navigate(Screen.NavigationSettings) }, onBack = { navigator.goBack() })
            }
            entry<Screen.AccessibilitySettings> {
                AccessibilitySettings(onBack = { navigator.goBack() })
            }
            entry<Screen.PrivacySettings> {
                PrivacySettings(onBack = { navigator.goBack() })
            }
            entry<Screen.ConnectionsSettings> {
                ConnectionsSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.DevicesSettings> {
                DevicesSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.ChatSettings> {
                ChatSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.NotificationsSettings> {
                NotificationsSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.NavigationSettings> {
                NavigationSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.AdvancedSettings> {
                AdvancedSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.AboutSettings> {
                val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
                SettingsSubScreen(title = "About", onNavigateBack = { navigator.goBack() }) {
                    AboutContent(version = me.lampu.lampcord.shared.update.APP_VERSION, onOpenUrl = { uriHandler.openUri(it) })
                }
            }
            entry<Screen.EasterEgg> {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("?", style = MaterialTheme.typography.displayLarge)
                        Text("The mysterious tab.", style = MaterialTheme.typography.bodyLarge)
                        Spacer(Modifier.height(24.dp))
                        Button(onClick = {
                            navigationStore.isSettingsVisible = true
                            navigator.navigate(Screen.Settings)
                        }) {
                            Text("Open Settings")
                        }
                    }
                }
            }
            entry<Screen.ServerSettings> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    ServerSettings(onDismiss = { navigationStore.isServerSettingsVisible = false })
                }
            }
            entry<Screen.ChannelSettings> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    ChannelSettingsScreen(onDismiss = { navigationStore.closeChannelSettings() })
                }
            }
            entry<Screen.Pins> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    PinnedMessagesScreen(onDismiss = { navigationStore.isPinsVisible = false })
                }
            }
            entry<Screen.Threads> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                    ThreadPanel(onDismiss = { navigationStore.isThreadPanelVisible = false })
                }
            }
            entry<Screen.ChannelsAndRoles> {
                val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                val guild = navigationStore.selectedGuild
                Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
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
            entry<Screen.Theming> {
                Box(Modifier.fillMaxSize().padding(bottom = navBarHeight)) {
                    ThemingSettings(onNavigateToEditor = { navigator.navigate(Screen.ThemeEditor(it)) }, onBack = { navigator.goBack() })
                }
            }
            entry<Screen.ThemeEditor> { screen: Screen.ThemeEditor ->
                Box(Modifier.fillMaxSize()) {
                    ThemeEditorScreen(themeJson = screen.themeJson, onBack = { navigator.goBack() })
                }
            }
        }
    )


    val lightPanelColor = MaterialTheme.colorScheme.surfaceContainerHigh
    val darkPanelColor = MaterialTheme.colorScheme.surfaceContainerLowest
    val chatBackground = MaterialTheme.colorScheme.background
    val memberHeaderColor = MaterialTheme.colorScheme.surface

    val settingsContainerColor = MaterialTheme.colorScheme.surfaceContainerLow

    val activePanelColor = remember(currentRoute, panelState.progress, lightPanelColor, darkPanelColor, chatBackground, memberHeaderColor, settingsContainerColor, isSettingsRoute, Settings.shared.panelType) {
        if (currentRoute == Screen.Chat) {
            val isOverlapping = Settings.shared.panelType == PanelType.OVERLAPPING
            val progress = panelState.progress
            val absProgress = kotlin.math.abs(progress).coerceIn(0f, 1f)
            
            if (progress > 0) { // Sliding to Sidebar (Start)
                androidx.compose.ui.graphics.lerp(chatBackground, lightPanelColor, absProgress)
            } else if (progress < 0) { // Sliding to Member List (End)
                if (isOverlapping) {
                    androidx.compose.ui.graphics.lerp(chatBackground, memberHeaderColor, absProgress)
                } else {
                    androidx.compose.ui.graphics.lerp(chatBackground, lightPanelColor, absProgress)
                }
            } else {
                chatBackground
            }
        } else if (isSettingsRoute || showSettingsTabBar) {
            settingsContainerColor
        } else {
            darkPanelColor
        }
    }

    Box(
        Modifier
            .fillMaxSize()
            .background(activePanelColor)
    ) {
        DiscordPanels(
            state = panelState,
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding(),
            swipeToStartEnabled = isChat && !navigationStore.isBubble,
            swipeToEndEnabled = isChat && !navigationStore.isBubble && me.lampu.lampcord.shared.settings.Settings.shared.chatGestures == me.lampu.lampcord.shared.settings.ChatGestures.SWIPE_TO_MEMBERS,
            startPanel = { 
                if (isChat) {
                    val currentBottomPadding = (navBarHeight * navBarVisibleAmount).coerceAtLeast(0.dp)
                    Box(Modifier.fillMaxSize().padding(bottom = currentBottomPadding)) {
                        Sidebar()
                    }
                }
            },
            endPanel = {
                if (isChat) {
                    MemberList()
                }
            },
            centerPanel = {
                Box(Modifier.fillMaxSize()) {
                    NavDisplay(
                        entries = entries,
                        onBack = {
                            navigator.goBack()
                        },
                        transitionSpec = {
                            if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
                                EnterTransition.None togetherWith ExitTransition.None
                            } else {
                                val targetKey = targetState.key
                                val initialKey = initialState.key

                                val targetIndex = routeIndexMap[targetKey] ?: routeIndexMap[targetKey::class] ?: -1
                                val initialIndex = routeIndexMap[initialKey] ?: routeIndexMap[initialKey::class] ?: -1

                                val enterTransition =
                                    if (targetIndex == -1 || targetIndex > initialIndex) {
                                        slideInHorizontally(animationSpec = quickSpatialSpec) { it / 8 } +
                                            fadeIn(quickEffectsSpec)
                                    } else {
                                        slideInHorizontally(animationSpec = quickSpatialSpec) { -it / 8 } +
                                            fadeIn(quickEffectsSpec)
                                    }

                                val exitTransition =
                                    if (targetIndex == -1 || targetIndex > initialIndex) {
                                        slideOutHorizontally(animationSpec = quickSpatialSpec) { -it / 8 } +
                                            fadeOut(quickEffectsSpec)
                                    } else {
                                        slideOutHorizontally(animationSpec = quickSpatialSpec) { it / 8 } +
                                            fadeOut(quickEffectsSpec)
                                    }

                                enterTransition togetherWith exitTransition
                            }
                        }
                    )

                    if (panelState.currentValue != DiscordPanelValue.Center) {
                        Box(
                            modifier = Modifier
                                .zIndex(1f)
                                .fillMaxSize()
                                .clickableCursor(
                                    interactionSource = remember { MutableInteractionSource() },
                                    indication = null,
                                    onClick = { panelState.close() },
                                )
                        )
                    }
                }
            }
        )

    val navItems = remember(settingsStore.navTabsOrderJson, settingsStore.showNavHome, settingsStore.showNavFriends, settingsStore.showNavSearch, settingsStore.showNavMentions, settingsStore.showNavSettings) {
        val order = runCatching { 
            settingsStore.navTabsOrderJson.removeSurrounding("[", "]").split(",").map { it.trim().removeSurrounding("\"") }
        }.getOrDefault(listOf("home", "friends", "search", "mentions", "settings"))
        
        val items = order.mapNotNull { key ->
            when (key) {
                "home" -> if (settingsStore.showNavHome) Screen.Chat else null
                "friends" -> if (settingsStore.showNavFriends) Screen.Friends else null
                "search" -> if (settingsStore.showNavSearch) Screen.GlobalSearch else null
                "mentions" -> if (settingsStore.showNavMentions) Screen.Mentions else null
                "settings" -> if (settingsStore.showNavSettings) Screen.Settings else null
                else -> null
            }
        }
        
        if (items.isEmpty()) listOf(Screen.EasterEgg) else items
    }

    if ((isTabRoute || showSettingsTabBar) && navBarVisibleAmount > 0.001f) {
        NavigationBar(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .graphicsLayer {
                    translationY = (1f - navBarVisibleAmount) * navBarHeight.toPx()
                    alpha = navBarVisibleAmount
                },
            containerColor = MaterialTheme.colorScheme.surface,
            tonalElevation = 0.dp,
            windowInsets = WindowInsets.navigationBars
        ) {
            navItems.forEach { screen ->
                when (screen) {
                    Screen.Chat -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.Chat,
                            onClick = {
                                navigationStore.isFriendsSelected = false
                                navigationStore.isSettingsVisible = false
                                navigationStore.isSearchVisible = false
                                navigationStore.isMentionsSelected = false
                                if (currentRoute != Screen.Chat) {
                                    if (lastPanelValue == DiscordPanelValue.Center) {
                                        lastPanelValue = DiscordPanelValue.Start
                                    }
                                    panelState.currentValue = lastPanelValue
                                } else {
                                    if (panelState.currentValue == DiscordPanelValue.Center) {
                                        panelState.openStart()
                                    } else {
                                        panelState.close()
                                    }
                                }
                                navigator.navigate(Screen.Chat)
                            },
                            icon = { Icon(if (currentRoute == Screen.Chat) Icons.Brand.DiscordRounded else Icons.Brand.DiscordRoundedOutline, "Home") },
                            label = { Text("Home") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    Screen.Friends -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.Friends,
                            onClick = {
                                navigationStore.isFriendsSelected = true
                                navigator.navigate(Screen.Friends)
                            },
                            icon = { Icon(if (currentRoute == Screen.Friends) Icons.Filled.Person else Icons.Rounded.Person, "Friends") },
                            label = { Text("Friends") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    Screen.GlobalSearch -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.GlobalSearch,
                            onClick = {
                                navigator.navigate(Screen.GlobalSearch)
                            },
                            icon = { Icon(Icons.Filled.Search, "Search") },
                            label = { Text("Search") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    Screen.Mentions -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.Mentions,
                            onClick = {
                                navigationStore.isMentionsSelected = true
                                navigator.navigate(Screen.Mentions)
                            },
                            icon = { Icon(Icons.Rounded.AlternateEmail, "Mentions") },
                            label = { Text("Mentions") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    Screen.Settings -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.Settings,
                            onClick = {
                                navigationStore.isSettingsVisible = true
                                navigator.navigate(Screen.Settings)
                            },
                            modifier = Modifier.pointerInput(Unit) {
                                awaitPointerEventScope {
                                    while (true) {
                                        val down = awaitFirstDown(
                                            pass = PointerEventPass.Initial,
                                            requireUnconsumed = false
                                        )
                                        val timedOut = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                                            while (true) {
                                                val event = awaitPointerEvent(PointerEventPass.Initial)
                                                if (event.changes.none { it.pressed }) {
                                                    return@withTimeoutOrNull false
                                                }
                                            }
                                            false
                                        } ?: true

                                        if (timedOut) {
                                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                            showUserStatusSheet = true
                                            down.consume()
                                            while (true) {
                                                val event = awaitPointerEvent(PointerEventPass.Initial)
                                                event.changes.forEach { it.consume() }
                                                if (event.changes.all { !it.pressed }) break
                                            }
                                        }
                                    }
                                }
                            },
                            icon = {
                                val user = currentUser
                                val avatarUrl = user?.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                                if (avatarUrl != null) {
                                    Box(modifier = Modifier.size(24.dp)) {
                                        AvatarWithDecoration(
                                            avatarUrl = avatarUrl,
                                            decorationData = currentUser?.avatar_decoration_data,
                                            size = 24.dp,
                                            status = presenceStore.getUserStatus(
                                                currentUser?.id ?: "",
                                                currentUser?.id,
                                                settingsStore.userSettings?.status
                                            )
                                        )
                                    }
                                } else {
                                    Icon(Icons.Filled.Settings, "You")
                                }
                            },
                            label = { Text("You") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    Screen.EasterEgg -> {
                        NavigationBarItem(
                            selected = currentRoute == Screen.EasterEgg,
                            onClick = { navigator.navigate(Screen.EasterEgg) },
                            icon = { Icon(Icons.Filled.QuestionMark, null) },
                            label = { Text("?") },
                            alwaysShowLabel = !settingsStore.hideNavLabels
                        )
                    }
                    else -> {}
                }
            }
        }
    }

        BackHandler(enabled = panelState.currentValue == DiscordPanelValue.End) {
            panelState.close()
        }

        BackHandler(enabled = panelState.currentValue == DiscordPanelValue.Center && isChat) {
            if (navigationStore.selectedThread != null) {
                navigationStore.selectedThread = null
            } else if (navigationStore.isChannelsAndRolesVisible) {
                navigationStore.isChannelsAndRolesVisible = false
            } else {
                panelState.openStart()
            }
        }
    }

    if (navigationStore.isAttachmentViewerVisible) {
        AttachmentViewer(
            items = navigationStore.attachmentViewerItems,
            selectedIndex = navigationStore.attachmentViewerIndex,
            onIndexChange = { navigationStore.attachmentViewerIndex = it },
            onDismiss = { navigationStore.closeAttachmentViewer() }
        )
    }

    if (profileStore.isProfileLoading || profileStore.selectedProfile != null) {
        val sheetState = rememberDiscordSheetState()

        DiscordBottomSheet(
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
                    modifier = Modifier.fillMaxWidth(),
                    showBorder = false,
                    isExpanded = true,
                    fillAvailableHeight = true,
                    showBoardTab = true,
                    onExpand = null,
                    onDismiss = {
                        profileStore.selectedProfile = null
                        profileStore.isProfileExpanded = false
                        profileStore.isProfileLoading = false
                    }
                )
            } else {
                ProfileCardSkeleton(
                    modifier = Modifier.fillMaxWidth(),
                    isExpanded = true
                )
            }
        }
    }

    if (showUserStatusSheet) {
        UserStatusBottomSheet(
            onDismiss = { showUserStatusSheet = false },
            onSwitchAccount = { account ->
                tokenStore.switchAccount(account.token)
                me.lampu.lampcord.shared.utils.restartApp()
            },
            onSetCustomStatus = { showCustomStatusDialog = true },
            onAddAccount = { showAddAccountDialog = true }
        )
    }

    if (showCustomStatusDialog) {
        CustomStatusDialog(
            initialText = settingsStore.userSettings?.custom_status?.text ?: "",
            onDismiss = { showCustomStatusDialog = false },
            onSave = { text ->
                scope.launch {
                    presenceStore.updateCustomStatus(text.ifBlank { null })
                    showCustomStatusDialog = false
                }
            }
        )
    }

    if (showAddAccountDialog) {
        androidx.compose.ui.window.Dialog(
            onDismissRequest = { showAddAccountDialog = false },
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
        ) {
            Surface(
                modifier = Modifier.fillMaxSize(),
                color = MaterialTheme.colorScheme.background
            ) {
                me.lampu.lampcord.shared.ui.LoginScreen(onLoginSuccess = { showAddAccountDialog = false })
            }
        }
    }

    if (navigationStore.isServerMenuVisible) {
        navigationStore.selectedGuild?.let { guild ->
            val sheetState = rememberDiscordSheetState()
            ServerBottomSheet(
                guild = guild,
                onDismiss = { navigationStore.isServerMenuVisible = false },
                sheetState = sheetState
            )
        }
    }

    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.BottomCenter
    ) {
        GlobalSnackbarHost(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(bottom = 12.dp)
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun MainBaseplateContent(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    presenceStore: PresenceStore,
    settingsStore: SettingsStore,
    voiceStore: VoiceStore,
    allUsers: Map<String, me.lampu.lampcord.shared.model.User>,
    currentUser: me.lampu.lampcord.shared.model.User?,
    panelState: DiscordPanelsState,
    activeChannel: me.lampu.lampcord.shared.model.Channel?
) {
    key(navigationStore.selectedGuild?.id ?: "home") {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
            tonalElevation = 0.dp
        ) {
            Scaffold(
                contentWindowInsets = WindowInsets(0, 0, 0, 0),
                topBar = {
                    if (activeChannel != null || navigationStore.isChannelsAndRolesVisible) {
                        TopAppBar(
                            windowInsets = TopAppBarDefaults.windowInsets,
                            title = {
                                val canOpenMemberList = settingsStore.mobileShowMemberListButton &&
                                        activeChannel != null &&
                                        !navigationStore.isChannelsAndRolesVisible
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = if (canOpenMemberList) {
                                        Modifier.clickableCursor(
                                            interactionSource = remember { MutableInteractionSource() },
                                            indication = null
                                        ) {
                                            if (panelState.currentValue == DiscordPanelValue.End) panelState.close()
                                            else panelState.openEnd()
                                        }
                                    } else {
                                        Modifier
                                    }
                                ) {
                                    if (navigationStore.isChannelsAndRolesVisible) {
                                        Text(
                                            text = "Browse Channels",
                                            style = MaterialTheme.typography.titleMedium,
                                            fontWeight = FontWeight.Bold
                                        )
                                    } else if (activeChannel != null) {
                                        if (activeChannel.type == 3) {
                                            val groupIconUrl = activeChannel.icon?.let { CdnUrls.getChannelIconUrl(activeChannel.id, it, 64) }
                                            if (groupIconUrl != null) {
                                                Box(modifier = Modifier.size(32.dp)) {
                                                    AvatarWithDecoration(
                                                        avatarUrl = groupIconUrl,
                                                        decorationData = null,
                                                        size = 32.dp
                                                    )
                                                }
                                            } else {
                                                Surface(
                                                    modifier = Modifier.size(32.dp),
                                                    shape = CircleShape,
                                                    color = MaterialTheme.colorScheme.primaryContainer
                                                ) {
                                                    Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                                                        Icon(
                                                            imageVector = Icons.Rounded.Groups,
                                                            contentDescription = null,
                                                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                                            modifier = Modifier.size(20.dp)
                                                        )
                                                    }
                                                }
                                            }
                                            Spacer(Modifier.width(12.dp))
                                        } else if (activeChannel.type == 1) {
                                            val recipientId = activeChannel.recipients?.firstOrNull()?.id ?: activeChannel.recipient_ids?.firstOrNull()
                                            val recipient = recipientId?.let { allUsers[it] } ?: activeChannel.recipients?.firstOrNull()

                                            if (recipient != null) {
                                                Box(modifier = Modifier.size(32.dp)) {
                                                    AvatarWithDecoration(
                                                        avatarUrl = recipient.avatar?.let { "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png?size=64" },
                                                        decorationData = recipient.avatar_decoration_data,
                                                        size = 32.dp,
                                                        status = presenceStore.getUserStatus(recipient.id, currentUser?.id, settingsStore.userSettings?.status)
                                                    )
                                                }
                                                Spacer(Modifier.width(12.dp))
                                            }
                                        }

                                        Column {
                                            Text(
                                                text = if (activeChannel.type == 3) {
                                                    if (activeChannel.name?.isNotBlank() == true) {
                                                        activeChannel.name!!
                                                    } else {
                                                        val recipients = activeChannel.recipients?.mapNotNull { allUsers[it.id] ?: it }
                                                            ?: activeChannel.recipient_ids?.mapNotNull { allUsers[it] }
                                                            ?: emptyList()
                                                        val otherRecipients = if (currentUser != null) recipients.filter { it.id != currentUser?.id } else recipients
                                                        val displayList = if (otherRecipients.isNotEmpty()) otherRecipients else recipients
                                                        displayList.mapNotNull { it.global_name ?: it.username }
                                                            .joinToString(", ")
                                                            .ifEmpty { "Unnamed Group DM" }
                                                    }
                                                } else if (activeChannel.type == 1) {
                                                    val recipientId = activeChannel.recipients?.firstOrNull()?.id ?: activeChannel.recipient_ids?.firstOrNull()
                                                    val recipient = recipientId?.let { allUsers[it] } ?: activeChannel.recipients?.firstOrNull()
                                                    recipient?.let { it.global_name ?: it.username } ?: "Chat"
                                                } else activeChannel.name ?: "Chat",
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = FontWeight.Bold,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                                modifier = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
                                                    Modifier
                                                } else {
                                                    Modifier.basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp)
                                                }
                                            )
                                        }
                                    }
                                }
                            },
                            navigationIcon = {
                                if (!navigationStore.isBubble && settingsStore.mobileShowChannelListButton) {
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
                                        val isChannelPrivate = remember(activeChannel, navigationStore.selectedGuild) {
                                            val g = navigationStore.selectedGuild
                                            if (g == null) false
                                            else PermissionHelper.isChannelPrivate(g, activeChannel)
                                        }
                                        Box(
                                            modifier = Modifier
                                                .padding(start = 4.dp)
                                                .size(40.dp)
                                                .clickableCursor(
                                                    interactionSource = remember { MutableInteractionSource() },
                                                    indication = null
                                                ) { panelState.openStart() },
                                            contentAlignment = Alignment.Center
                                        ) {
                                            Box(modifier = Modifier.size(22.dp)) {
                                                Icon(
                                                    imageVector = channelIcon,
                                                    contentDescription = "Channels",
                                                    modifier = Modifier.fillMaxSize(),
                                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                                if (isChannelPrivate && activeChannel.type != 4) {
                                                    Surface(
                                                        modifier = Modifier
                                                            .align(Alignment.TopEnd)
                                                            .offset(x = 2.dp, y = (-2).dp)
                                                            .size(11.dp),
                                                        shape = CircleShape,
                                                        color = MaterialTheme.colorScheme.surfaceContainerLowest,
                                                    ) {
                                                        Icon(
                                                            imageVector = Icons.Filled.Lock,
                                                            contentDescription = null,
                                                            modifier = Modifier.padding(1.dp).fillMaxSize(),
                                                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                        )
                                                    }
                                                }
                                            }
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
                                if (settingsStore.showCallButton && activeChannel != null && (activeChannel.type == 1 || activeChannel.type == 3)) VoiceCallButton(activeChannel)
                                if (!navigationStore.isBubble && activeChannel != null && (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3)) {
                                    if (settingsStore.showChatSearch) {
                                        IconButton(onClick = { navigationStore.isSearchVisible = true }) {
                                            Icon(
                                                imageVector = Icons.Filled.Search,
                                                contentDescription = "Search",
                                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                        }
                                    }
                                    if (activeChannel.type != 2 && activeChannel.type != 13 && settingsStore.showChatPins) {
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
                                containerColor = MaterialTheme.colorScheme.background,
                                scrolledContainerColor = MaterialTheme.colorScheme.background
                            )
                        )
                    }
                }
            ) { padding ->
                val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
                val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

                AnimatedContent(
                    targetState = activeChannel?.id
                        ?: if (navigationStore.isChannelsAndRolesVisible) "roles" else "none",
                    transitionSpec = {
                        if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
                            EnterTransition.None togetherWith ExitTransition.None
                        } else {
                            (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                            )
                        }
                    },
                    modifier = Modifier
                        .padding(top = padding.calculateTopPadding(), bottom = 0.dp)
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.background),
                    label = "MainContentTransition"
                ) { target ->
                    Box(Modifier.fillMaxSize()) {
                        if (activeChannel != null && target == activeChannel.id) {
                            val isVoice = activeChannel.type == 2 || activeChannel.type == 13
                            if (isVoice && !voiceStore.isVoiceChatTextVisible) {
                                VoiceArea(activeChannel)
                            } else {
                                Column(
                                    modifier = Modifier
                                        .fillMaxSize()
                                ) {
                                    Box(modifier = Modifier.weight(1f)) {
                                            if (activeChannel.type == 15 && navigationStore.selectedThread == null) {
                                                ForumPostList()
                                            } else {
                                                ChatArea(modifier = Modifier.fillMaxSize())
                                            }
                                        }
                                        if (!(activeChannel.type == 15 && navigationStore.selectedThread == null)) {
                                            ChatInputBar(activeChannel)
                                        }
                                }
                            }
                        } else if (target == "roles") {
                            me.lampu.lampcord.shared.ui.components.guilds.ChannelsAndRoles()
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
        }
    }
}
