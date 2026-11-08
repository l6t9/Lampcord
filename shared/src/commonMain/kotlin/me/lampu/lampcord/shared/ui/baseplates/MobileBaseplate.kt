package me.lampu.lampcord.shared.ui.baseplates

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back
import me.lampu.lampcord.shared.generated.resources.channels
import me.lampu.lampcord.shared.generated.resources.friends
import me.lampu.lampcord.shared.generated.resources.messages
import me.lampu.lampcord.shared.generated.resources.home
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_nav_you
import me.lampu.lampcord.shared.generated.resources.mentions
import me.lampu.lampcord.shared.generated.resources.password_manager_open_settings
import me.lampu.lampcord.shared.generated.resources.pins
import me.lampu.lampcord.shared.generated.resources.profile
import me.lampu.lampcord.shared.generated.resources.search
import me.lampu.lampcord.shared.generated.resources.settings
import org.jetbrains.compose.resources.stringResource

import me.lampu.lampcord.shared.api.CdnUrls
import androidx.compose.animation.*
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.navigation3.runtime.entryProvider
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
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.UserProfile
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.FilledTonalButton
import androidx.compose.foundation.shape.CircleShape
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.ui.components.profiles.ProfileBanner
import me.lampu.lampcord.shared.model.ProfileCollectibles
import me.lampu.lampcord.shared.model.ProfileFrameProduct
import me.lampu.lampcord.shared.ui.components.profiles.ProfileFrameOverlay
import me.lampu.lampcord.shared.ui.components.profiles.profileBannerHeight
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
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.text.style.TextAlign

private data class BottomNavItem(val key: String, val screen: Screen)

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class, ExperimentalFoundationApi::class)
@Composable
fun MobileBaseplate(
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    userStore: UserStore = koinInject(),
    voiceStore: VoiceStore = koinInject()
) {
    val presenceStore: PresenceStore = koinInject()
    val settingsStore: SettingsStore = koinInject()
    val sessionManager: SessionManager = koinInject()
    val guildStore: GuildStore = koinInject()
    val readStateStore: ReadStateStore = koinInject()
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
            Screen.You,
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
            Screen.You to 1,
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
            Screen.VoiceSettings to 12,
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
            if (navigationState.topLevelRoute != Screen.Chat) navigator.navigate(Screen.Chat)
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

    LaunchedEffect(currentUser?.id) {
        profileStore.preloadOwnProfile(currentUser?.id)
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
        currentRoute == Screen.LanguageSettings ||
        currentRoute == Screen.PrivacySettings ||
        currentRoute == Screen.ConnectionsSettings ||
        currentRoute == Screen.DevicesSettings ||
        currentRoute == Screen.VoiceSettings ||
        currentRoute == Screen.ChatSettings ||
        currentRoute == Screen.NotificationsSettings ||
        currentRoute == Screen.NavigationSettings ||
        currentRoute == Screen.AdvancedSettings ||
        currentRoute == Screen.AdditionalSettings ||
        currentRoute == Screen.AboutSettings ||
        currentRoute == Screen.UpdatesSettings
    val isTabRoute = currentRoute in setOf(
        Screen.Chat, Screen.Mentions, Screen.GlobalSearch
    )

    val targetNavBarVisibleAmount = remember(panelState.progress, panelState.currentValue, currentRoute, isTabRoute) {
        val progress = panelState.progress
        if (isTabRoute) {
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
            entry<Screen.You> {
                val me = currentUser
                val profile = profileStore.ownProfile?.takeIf { it.user.id == me?.id }

                LaunchedEffect(me?.id, profile) {
                    if (me?.id != null && profile == null) {
                        profileStore.preloadOwnProfile(me.id)
                    }
                }

                Box(modifier = Modifier.fillMaxSize()) {
                    val contentLayout = Modifier
                        .fillMaxSize()

                    if (profile != null) {
                        ProfileCard(
                            profile = profile,
                            modifier = contentLayout,
                            isExpanded = true,
                            fillAvailableHeight = true,
                            showBorder = false,
                            showFrameArt = false,
                            showMemberSince = true,
                            contentBottomPadding = 112.dp,
                            onOpenFriends = { navigator.navigate(Screen.Friends) }
                        )
                        val frameSku = remember(profile) {
                            ProfileCollectibles.frameSku(profile.user_profile, profile.guild_member_profile)
                        }
                        val frameProduct = frameSku?.let { profileStore.getFrame(it) }
                        frameProduct?.let { product ->
                            Box(modifier = contentLayout) {
                                ProfileFrameOverlay(
                                    product = product,
                                    modifier = Modifier.matchParentSize(),
                                    railTop = profileBannerHeight(0.dp, true, null, 0f),
                                    showBottom = false
                                )
                            }
                        }
                    } else if (me != null) {
                        Box(contentLayout, contentAlignment = Alignment.Center) {
                            ContainedLoadingIndicator(modifier = Modifier.size(40.dp))
                        }
                    }

                    HorizontalFloatingToolbar(
                        expanded = true,
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .padding(bottom = 16.dp)
                            .navigationBarsPadding(),
                        colors = FloatingToolbarDefaults.standardFloatingToolbarColors(
                            toolbarContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh
                        )
                    ) {
                        FilledTonalButton(onClick = { navigator.navigate(Screen.ProfilesSettings) }) {
                            Icon(Icons.Rounded.Edit, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(Res.string.profile))
                        }
                        Spacer(Modifier.width(8.dp))
                        FilledTonalButton(onClick = { navigator.navigate(Screen.Settings) }) {
                            Icon(Icons.Rounded.Settings, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(Res.string.settings))
                        }
                    }
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
                        onNavigateToLanguage = { navigator.navigate(Screen.LanguageSettings) },
                        onNavigateToVoice = { navigator.navigate(Screen.VoiceSettings) },
                        onNavigateToChat = { navigator.navigate(Screen.ChatSettings) },
                        onNavigateToNotifications = { navigator.navigate(Screen.NotificationsSettings) },
                        onNavigateToAdvanced = { navigator.navigate(Screen.AdvancedSettings) },
                        onNavigateToAdditional = { navigator.navigate(Screen.AdditionalSettings) },
                        onNavigateToAbout = { navigator.navigate(Screen.AboutSettings) },
                        onNavigateToUpdates = { navigator.navigate(Screen.UpdatesSettings) },
                        onNavigateToTheming = { navigator.navigate(Screen.Theming) },
                        onNavigateToNavigation = { navigator.navigate(Screen.NavigationSettings) },
                        onDismiss = { navigationStore.isSettingsVisible = false }
                    )
                }
            }
            entry<Screen.AccountSettings> {
                val userStoreForAccount: UserStore = koinInject()
                AccountSettings(onBack = { navigator.goBack() }, userStore = userStoreForAccount)
            }
            entry<Screen.ProfilesSettings> {
                val userStoreForProfiles: UserStore = koinInject()
                ProfilesSettings(onBack = { navigator.goBack() }, userStore = userStoreForProfiles)
            }
            entry<Screen.AppearanceSettings> {
                AppearanceSettings(onNavigateToTheming = { navigator.navigate(Screen.Theming) }, onNavigateToNavigation = { navigator.navigate(Screen.NavigationSettings) }, onBack = { navigator.goBack() })
            }
            entry<Screen.AccessibilitySettings> {
                AccessibilitySettings(onBack = { navigator.goBack() })
            }
            entry<Screen.LanguageSettings> {
                me.lampu.lampcord.shared.ui.settings.LanguageSettings(onBack = { navigator.goBack() })
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
            entry<Screen.VoiceSettings> {
                me.lampu.lampcord.shared.ui.settings.VoiceSettings(onBack = { navigator.goBack() })
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
            entry<Screen.AdditionalSettings> {
                AdditionalSettings(onBack = { navigator.goBack() })
            }
            entry<Screen.UpdatesSettings> {
                SettingsSubScreen(title = "Updates", onNavigateBack = { navigator.goBack() }) {
                    me.lampu.lampcord.shared.ui.settings.UpdatesContent()
                }
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
                            Text(stringResource(Res.string.password_manager_open_settings))
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

    val lightPanelColor = MaterialTheme.colorScheme.surfaceContainer
    val darkPanelColor = MaterialTheme.colorScheme.surfaceContainer
    val chatBackground = MaterialTheme.colorScheme.surfaceContainer
    val memberHeaderColor = MaterialTheme.colorScheme.surface

    val settingsContainerColor = MaterialTheme.colorScheme.surfaceContainer

    val activePanelColor = remember(currentRoute, panelState.progress, lightPanelColor, darkPanelColor, chatBackground, memberHeaderColor, settingsContainerColor, isSettingsRoute, Settings.shared.panelType) {
        if (currentRoute == Screen.Chat) {
            val isOverlapping = Settings.shared.panelType == PanelType.OVERLAPPING
            val progress = panelState.progress
            val absProgress = kotlin.math.abs(progress).coerceIn(0f, 1f)

            if (progress > 0) {
                androidx.compose.ui.graphics.lerp(chatBackground, lightPanelColor, absProgress)
            } else if (progress < 0) {
                if (isOverlapping) {
                    androidx.compose.ui.graphics.lerp(chatBackground, memberHeaderColor, absProgress)
                } else {
                    androidx.compose.ui.graphics.lerp(chatBackground, lightPanelColor, absProgress)
                }
            } else {
                chatBackground
            }
        } else if (isSettingsRoute) {
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
            swipeToEndEnabled = isChat && !navigationStore.isBubble && !Settings.shared.swipeToReplyEnabled,
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
                    val overlapping = Settings.shared.panelType == PanelType.OVERLAPPING
                    MemberList(
                        overlapping = overlapping,
                        onBack = if (overlapping) ({ panelState.close() }) else null
                    )
                }
            },
            centerPanel = {
                Box(Modifier.fillMaxSize()) {
                    val currentEntry = entries.lastOrNull()
                    if (currentEntry != null) {
                        AnimatedContent(
                            targetState = currentEntry,
                            transitionSpec = {
                                if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
                                    EnterTransition.None togetherWith ExitTransition.None
                                } else {
                                    val targetKey = targetState.contentKey
                                    val initialKey = initialState.contentKey

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
                            },
                            contentKey = { it.contentKey },
                            label = "MainContentTransition"
                        ) { entry ->
                            entry.Content()
                        }
                    }

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

        Box(
            modifier = Modifier
                .align(Alignment.TopCenter)
                .fillMaxWidth()
                .height(WindowInsets.statusBars.asPaddingValues().calculateTopPadding())
                .background(MaterialTheme.colorScheme.surfaceContainer)
        )

        val navItems = remember(
            settingsStore.navTabsOrderJson,
            settingsStore.showNavHome,
            settingsStore.showNavDm,
            settingsStore.showNavSearch,
            settingsStore.showNavMentions,
            settingsStore.showNavSettings
        ) {
            val defaultOrder = listOf("home", "dm", "search", "mentions", "you")
            val order = runCatching {
                settingsStore.navTabsOrderJson.removeSurrounding("[", "]").split(",").map { it.trim().removeSurrounding("\"") }
            }.getOrDefault(defaultOrder)

            val normalizedOrder = order
                .map { if (it == "settings") "you" else it }
                .filter { it != "friends" }
                .let { list -> defaultOrder.filter { it in list } + defaultOrder.filter { it !in list } }

            val items = normalizedOrder.mapNotNull { key ->
                when (key) {
                    "home" -> if (settingsStore.showNavHome) BottomNavItem("home", Screen.Chat) else null
                    "dm" -> if (settingsStore.showNavDm) BottomNavItem("dm", Screen.Chat) else null
                    "search" -> if (settingsStore.showNavSearch) BottomNavItem("search", Screen.GlobalSearch) else null
                    "mentions" -> if (settingsStore.showNavMentions) BottomNavItem("mentions", Screen.Mentions) else null
                    "you" -> if (settingsStore.showNavSettings) BottomNavItem("you", Screen.You) else null
                    else -> null
                }
            }

            if (items.isEmpty()) listOf(BottomNavItem("easter", Screen.EasterEgg)) else items
        }

        if (isTabRoute && navBarVisibleAmount > 0.001f) {
            if (settingsStore.useYouBar) {
                val privateChannels by guildStore.privateChannels.collectAsState()
                val readStates by readStateStore.readStates.collectAsState()
                val dmMentionCount = remember(privateChannels, readStates) {
                    privateChannels.sumOf { readStates[it.id]?.mention_count ?: 0 }
                }
                var swipeTotal by remember { mutableFloatStateOf(0f) }
                Surface(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .padding(start = 12.dp, end = 12.dp, bottom = 8.dp)
                        .windowInsetsPadding(WindowInsets.navigationBars)
                        .graphicsLayer {
                            translationY = (1f - navBarVisibleAmount) * navBarHeight.toPx()
                            alpha = navBarVisibleAmount
                        }
                        .pointerInput(navigationStore.selectedGuild?.id) {
                            detectHorizontalDragGestures(
                                onDragStart = { swipeTotal = 0f },
                                onDragCancel = { swipeTotal = 0f },
                                onDragEnd = {
                                    val threshold = 72f
                                    if (swipeTotal > threshold) {
                                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                        navigationStore.selectHome()
                                        navigator.navigate(Screen.Chat)
                                    } else if (swipeTotal < -threshold) {
                                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                        navigator.navigate(Screen.Chat)
                                        if (panelState.currentValue != DiscordPanelValue.Start) {
                                            panelState.openStart()
                                        }
                                    }
                                    swipeTotal = 0f
                                },
                                onHorizontalDrag = { change, dragAmount ->
                                    change.consume()
                                    swipeTotal += dragAmount
                                }
                            )
                        },
                    shape = MaterialTheme.shapes.extraLarge,
                    color = MaterialTheme.colorScheme.surfaceContainer,
                    tonalElevation = 3.dp,
                    shadowElevation = 2.dp
                ) {
                    val nameplate = currentUser?.collectibles?.nameplate
                    Box(modifier = Modifier.fillMaxWidth()) {
                        if (nameplate != null) {
                            val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                            AsyncImage(
                                model = decoUrl,
                                contentDescription = null,
                                modifier = Modifier.matchParentSize(),
                                contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                alpha = 0.4f
                            )
                        }
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 72.dp)
                                .padding(horizontal = 12.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                        val user = currentUser
                        Row(
                            modifier = Modifier
                                .weight(1f)
                                .clip(MaterialTheme.shapes.large)
                                .combinedClickable(
                                    onClick = { navigator.navigate(Screen.You) },
                                    onLongClick = {
                                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                        showUserStatusSheet = true
                                    }
                                )
                                .padding(horizontal = 8.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(modifier = Modifier.size(40.dp)) {
                                AvatarWithDecoration(
                                    avatarUrl = user?.avatar?.let { CdnUrls.getUserAvatarUrl(user.id, it, 128) },
                                    decorationData = user?.avatar_decoration_data,
                                    size = 40.dp,
                                    status = presenceStore.getUserStatus(
                                        user?.id ?: "",
                                        user?.id,
                                        settingsStore.userSettings?.status
                                    )
                                )
                            }
                            Spacer(Modifier.width(12.dp))
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = user?.global_name ?: user?.username ?: stringResource(Res.string.lampcord_settings_nav_you),
                                    style = MaterialTheme.typography.titleSmall,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                                Text(
                                    text = "@${user?.username ?: ""}",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }

                        val favoritesVisible = Settings.shared.favoritesServerEnabled
                        val dmSelected = currentRoute == Screen.Chat &&
                            navigationStore.selectedGuild == null &&
                            !(navigationStore.isFavoritesSelected && favoritesVisible)
                        val profileActions = buildList {
                            if (settingsStore.showNavDm) add("dm")
                            if (settingsStore.showNavSearch) add("search")
                            add("mentions")
                        }

                        ButtonGroup(
                            modifier = Modifier
                                .height(40.dp)
                                .animateContentSize(
                                    animationSpec = if (Settings.shared.reduceMotion) snap() else spring(dampingRatio = 0.8f, stiffness = 500f)
                                ),
                            overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                            horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                        ) {
                            profileActions.forEachIndexed { index, action ->
                                val isFirst = index == 0
                                val isLast = index == profileActions.lastIndex
                                customItem(
                                    buttonGroupContent = {
                                        val shapes = when {
                                            profileActions.size == 1 -> ButtonGroupDefaults.connectedMiddleButtonShapes()
                                            isFirst -> ButtonGroupDefaults.connectedLeadingButtonShapes()
                                            isLast -> ButtonGroupDefaults.connectedTrailingButtonShapes()
                                            else -> ButtonGroupDefaults.connectedMiddleButtonShapes()
                                        }
                                        when (action) {
                                            "dm" -> ToggleButton(
                                                checked = dmSelected,
                                                onCheckedChange = {
                                                    navigationStore.isSettingsVisible = false
                                                    navigationStore.isSearchVisible = false
                                                    navigationStore.selectHome()
                                                    navigator.navigate(Screen.Chat)
                                                },
                                                shapes = shapes,
                                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                                contentPadding = PaddingValues(0.dp),
                                                modifier = Modifier.fillMaxSize().widthIn(min = 40.dp)
                                            ) {
                                                BadgedBox(
                                                    badge = {
                                                        if (dmMentionCount > 0) {
                                                            Badge { Text(if (dmMentionCount > 99) "99+" else dmMentionCount.toString()) }
                                                        }
                                                    }
                                                ) {
                                                    Icon(
                                                        imageVector = if (dmSelected) Icons.Filled.ChatBubble else Icons.Rounded.ChatBubble,
                                                        contentDescription = stringResource(Res.string.messages),
                                                        modifier = Modifier.size(20.dp)
                                                    )
                                                }
                                            }
                                            "search" -> ToggleButton(
                                                checked = currentRoute == Screen.GlobalSearch,
                                                onCheckedChange = { navigator.navigate(Screen.GlobalSearch) },
                                                shapes = shapes,
                                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                                contentPadding = PaddingValues(0.dp),
                                                modifier = Modifier.fillMaxSize().widthIn(min = 40.dp)
                                            ) {
                                                Icon(
                                                    imageVector = Icons.Filled.Search,
                                                    contentDescription = stringResource(Res.string.search),
                                                    modifier = Modifier.size(20.dp)
                                                )
                                            }
                                            else -> ToggleButton(
                                                checked = currentRoute == Screen.Mentions,
                                                onCheckedChange = {
                                                    navigationStore.isFriendsSelected = false
                                                    navigationStore.isMentionsSelected = true
                                                    navigator.navigate(Screen.Mentions)
                                                },
                                                shapes = shapes,
                                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                                contentPadding = PaddingValues(0.dp),
                                                modifier = Modifier.fillMaxSize().widthIn(min = 40.dp)
                                            ) {
                                                Icon(
                                                    imageVector = Icons.Rounded.AlternateEmail,
                                                    contentDescription = stringResource(Res.string.mentions),
                                                    modifier = Modifier.size(20.dp)
                                                )
                                            }
                                        }
                                    },
                                    menuContent = {
                                        DropdownMenuItem(
                                            text = {
                                                Text(
                                                    when (action) {
                                                        "dm" -> stringResource(Res.string.messages)
                                                        "search" -> stringResource(Res.string.search)
                                                        else -> stringResource(Res.string.mentions)
                                                    }
                                                )
                                            },
                                            onClick = {
                                                when (action) {
                                                    "dm" -> {
                                                        navigationStore.selectHome()
                                                        navigator.navigate(Screen.Chat)
                                                    }
                                                    "search" -> navigator.navigate(Screen.GlobalSearch)
                                                    else -> navigator.navigate(Screen.Mentions)
                                                }
                                            },
                                            leadingIcon = {
                                                Icon(
                                                    when (action) {
                                                        "dm" -> Icons.Rounded.ChatBubble
                                                        "search" -> Icons.Filled.Search
                                                        else -> Icons.Rounded.AlternateEmail
                                                    },
                                                    null
                                                )
                                            }
                                        )
                                    }
                                )
                            }
                        }
                    }
                    }
                }
            } else {
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
                    navItems.forEach { item ->
                        when (item.key) {
                            "home" -> {
                                val favoritesVisible = Settings.shared.favoritesServerEnabled
                                val dmSelected = currentRoute == Screen.Chat &&
                                    navigationStore.selectedGuild == null &&
                                    !(navigationStore.isFavoritesSelected && favoritesVisible)
                                val homeSelected = currentRoute == Screen.Chat && (!settingsStore.showNavDm || !dmSelected)
                                NavigationBarItem(
                                    selected = homeSelected,
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
                                    icon = { Icon(if (homeSelected) Icons.Filled.Home else Icons.Rounded.Home, "Home") },
                                    label = { Text(stringResource(Res.string.home)) },
                                    alwaysShowLabel = !settingsStore.hideNavLabels
                                )
                            }
                            "dm" -> {
                                val favoritesVisible = Settings.shared.favoritesServerEnabled
                                val dmSelected = currentRoute == Screen.Chat &&
                                    navigationStore.selectedGuild == null &&
                                    !(navigationStore.isFavoritesSelected && favoritesVisible)
                                NavigationBarItem(
                                    selected = dmSelected,
                                    onClick = {
                                        navigationStore.isSettingsVisible = false
                                        navigationStore.isSearchVisible = false
                                        navigationStore.selectHome()
                                        if (currentRoute != Screen.Chat) {
                                            if (lastPanelValue == DiscordPanelValue.Center) {
                                                lastPanelValue = DiscordPanelValue.Start
                                            }
                                            panelState.currentValue = lastPanelValue
                                        }
                                        navigator.navigate(Screen.Chat)
                                    },
                                    icon = { Icon(if (dmSelected) Icons.Filled.ChatBubble else Icons.Rounded.ChatBubble, "Direct Messages") },
                                    label = { Text(stringResource(Res.string.messages)) },
                                    alwaysShowLabel = !settingsStore.hideNavLabels
                                )
                            }

                            "search" -> {
                                NavigationBarItem(
                                    selected = currentRoute == Screen.GlobalSearch,
                                    onClick = {
                                        navigator.navigate(Screen.GlobalSearch)
                                    },
                                    icon = { Icon(Icons.Filled.Search, "Search") },
                                    label = { Text(stringResource(Res.string.search)) },
                                    alwaysShowLabel = !settingsStore.hideNavLabels
                                )
                            }
                            "mentions" -> {
                                NavigationBarItem(
                                    selected = currentRoute == Screen.Mentions,
                                    onClick = {
                                        navigationStore.isFriendsSelected = false
                                        navigationStore.isMentionsSelected = true
                                        navigator.navigate(Screen.Mentions)
                                    },
                                    icon = { Icon(Icons.Rounded.AlternateEmail, "Mentions") },
                                    label = { Text(stringResource(Res.string.mentions)) },
                                    alwaysShowLabel = !settingsStore.hideNavLabels
                                )
                            }
                            "you" -> {
                                NavigationBarItem(
                                    selected = currentRoute == Screen.You,
                                    onClick = {
                                        navigator.navigate(Screen.You)
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
                                        val avatarUrl = user?.avatar?.let { CdnUrls.getUserAvatarUrl(user.id, user.avatar, 64) }
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
                                            Icon(Icons.Rounded.AccountCircle, stringResource(Res.string.lampcord_settings_nav_you))
                                        }
                                    },
                                    label = { Text(stringResource(Res.string.lampcord_settings_nav_you)) },
                                    alwaysShowLabel = !settingsStore.hideNavLabels
                                )
                            }
                            "easter" -> {
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
        }

        PlatformBackHandler(enabled = navigationStore.isNotificationsSettingsVisible) {
            navigationStore.isNotificationsSettingsVisible = false
        }

        PlatformBackHandler(enabled = panelState.currentValue == DiscordPanelValue.End) {
            panelState.close()
        }

        PlatformBackHandler(enabled = panelState.currentValue == DiscordPanelValue.Center && !navigationStore.isSettingsVisible && !navigationStore.isServerSettingsVisible && navigationStore.channelSettingsChannel == null && !navigationStore.isSearchVisible && !navigationStore.isPinsVisible) {
            if (navigationStore.selectedThread != null) {
                navigationStore.selectedThread = null
            } else if (navigationStore.isChannelsAndRolesVisible) {
                navigationStore.isChannelsAndRolesVisible = false
            } else if (isChat) {
                panelState.openStart()
            } else if (navigationState.topLevelRoute != navigationState.startRoute) {
                navigator.goBack()
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
            fillHeight = true,
            fullBleed = true,
            dragHandle = {},
            onDismissRequest = {
                profileStore.selectedProfile = null
                profileStore.isProfileExpanded = false
                profileStore.isProfileLoading = false
            },
            sheetState = sheetState,
        ) {
            val p = profileStore.selectedProfile
            val cardShape = RoundedCornerShape(
                topStart = DiscordSheetCornerRadius,
                topEnd = DiscordSheetCornerRadius
            )

            Box(Modifier.fillMaxSize()) {
                val cardLayout = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxHeight(0.9f)
                    .widthIn(max = 560.dp)
                    .fillMaxWidth()
                Box(
                    modifier = cardLayout
                        .clip(cardShape)
                        .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                ) {
                    if (p != null) {
                        ProfileCard(
                            profile = p,
                            modifier = Modifier.fillMaxSize(),
                            showBorder = false,
                            isExpanded = true,
                            fillAvailableHeight = true,
                            showBoardTab = true,
                            showFrameArt = false,
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

                if (p != null) {
                    val frameSku = remember(p) {
                        ProfileCollectibles.frameSku(p.user_profile, p.guild_member_profile)
                    }
                    val frameProduct = frameSku?.let { profileStore.getFrame(it) }
                    frameProduct?.let { product ->
                        Box(modifier = cardLayout) {
                            ProfileFrameOverlay(
                                product = product,
                                modifier = Modifier.matchParentSize(),
                                railTop = profileBannerHeight(0.dp, true, null, 0f),
                                showBottom = false
                            )
                        }
                    }
                }

                Box(modifier = cardLayout) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.TopCenter)
                            .padding(top = 10.dp)
                            .size(width = 36.dp, height = 4.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(Color.White.copy(alpha = 0.65f))
                    )
                }
            }
        }
    }

    if (showUserStatusSheet) {
        UserStatusBottomSheet(
            onDismiss = { showUserStatusSheet = false },
            onSwitchAccount = { account ->
                sessionManager.switchAccount(account.token)
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
                                                        avatarUrl = recipient.avatar?.let { CdnUrls.getUserAvatarUrl(recipient.id, recipient.avatar, 64) },
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
                                                        activeChannel.name
                                                    } else {
                                                        val recipients = activeChannel.recipients?.mapNotNull { allUsers[it.id] ?: it }
                                                            ?: activeChannel.recipient_ids?.mapNotNull { allUsers[it] }
                                                            ?: emptyList()
                                                        val currentUserId = currentUser?.id
                                                        val otherRecipients = if (currentUserId != null) recipients.filter { it.id != currentUserId } else recipients
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
                                                modifier = if (!me.lampu.lampcord.shared.settings.Settings.shared.marqueeEnabled) {
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
                                                contentDescription = stringResource(Res.string.back)
                                            )
                                        }
                                    } else if (isRoles) {
                                        IconButton(onClick = {
                                            navigationStore.isChannelsAndRolesVisible = false
                                        }) {
                                            Icon(
                                                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                                contentDescription = stringResource(Res.string.back)
                                            )
                                        }
                                    } else if (Settings.shared.panelType != PanelType.OVERLAPPING && activeChannel != null && activeChannel.type != 1 && activeChannel.type != 3 && activeChannel.guild_id != null) {
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
                                                .size(44.dp)
                                                .clickableCursor(
                                                    interactionSource = remember { MutableInteractionSource() },
                                                    indication = null
                                                ) { panelState.openStart() },
                                            contentAlignment = Alignment.Center
                                        ) {
                                            Box(modifier = Modifier.size(28.dp)) {
                                                Icon(
                                                    imageVector = channelIcon,
                                                    contentDescription = stringResource(Res.string.channels),
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
                                    } else if (Settings.shared.panelType == PanelType.OVERLAPPING) {
                                        IconButton(onClick = { panelState.openStart() }) {
                                            Icon(
                                                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                                contentDescription = stringResource(Res.string.back)
                                            )
                                        }
                                    } else if (activeChannel?.type != 1 && activeChannel?.type != 3) {
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
                                val canOpenMemberListAction = settingsStore.mobileShowMemberListButton &&
                                        activeChannel != null &&
                                        !navigationStore.isChannelsAndRolesVisible
                                if (settingsStore.showCallButton && activeChannel != null && (activeChannel.type == 1 || activeChannel.type == 3)) VoiceCallButton(activeChannel)
                                if (!navigationStore.isBubble && activeChannel != null && (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3)) {
                                    if (settingsStore.showChatSearch) {
                                        IconButton(onClick = { navigationStore.isSearchVisible = true }) {
                                            Icon(
                                                imageVector = Icons.Filled.Search,
                                                contentDescription = stringResource(Res.string.search),
                                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                        }
                                    }
                                    if (activeChannel.type != 2 && activeChannel.type != 13 && settingsStore.showChatPins) {
                                        IconButton(onClick = { navigationStore.isPinsVisible = true }) {
                                            Icon(
                                                imageVector = Icons.Filled.PushPin,
                                                contentDescription = stringResource(Res.string.pins),
                                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                        }
                                    }
                                    if (canOpenMemberListAction && activeChannel.type != 15) {
                                        val memberListVisible = panelState.currentValue == DiscordPanelValue.End
                                        IconButton(
                                            onClick = {
                                                if (memberListVisible) panelState.close() else panelState.openEnd()
                                            }
                                        ) {
                                            Icon(
                                                imageVector = Icons.Filled.Group,
                                                contentDescription = "Toggle Member List",
                                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                        }
                                    }
                                }
                            },
                            colors = TopAppBarDefaults.topAppBarColors(
                                containerColor = MaterialTheme.colorScheme.surfaceContainer,
                                scrolledContainerColor = MaterialTheme.colorScheme.surfaceContainer
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
                                    DmCallStage(activeChannel)
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
                                    Icon(Icons.Rounded.Home, null, modifier = Modifier.size(80.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.2f))
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
