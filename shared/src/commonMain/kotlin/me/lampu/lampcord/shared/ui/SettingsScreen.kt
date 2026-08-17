package me.lampu.lampcord.shared.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LargeTopAppBar
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.WideNavigationRail
import androidx.compose.material3.WideNavigationRailDefaults
import androidx.compose.material3.WideNavigationRailItem
import androidx.compose.material3.WideNavigationRailState
import androidx.compose.material3.WideNavigationRailValue
import androidx.compose.material3.rememberWideNavigationRailState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchDestination
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchEntry
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchField
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchResults
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.AccessibilitySettings
import me.lampu.lampcord.shared.ui.settings.AccountSettings
import me.lampu.lampcord.shared.ui.settings.AdvancedSettings
import me.lampu.lampcord.shared.ui.settings.AppearanceSettings
import me.lampu.lampcord.shared.ui.settings.ChatSettings
import me.lampu.lampcord.shared.ui.settings.ConnectionsSettings
import me.lampu.lampcord.shared.ui.settings.DevicesSettings
import me.lampu.lampcord.shared.ui.settings.NotificationsSettings
import me.lampu.lampcord.shared.ui.settings.PrivacySettings
import me.lampu.lampcord.shared.ui.settings.ProfileSettings
import org.koin.compose.koinInject

enum class SettingsSection(val title: String, val icon: ImageVector, val selectedIcon: ImageVector) {
    ACCOUNT("Account", Icons.Rounded.AccountCircle, Icons.Filled.AccountCircle),
    PROFILES("Profiles", Icons.Rounded.Person, Icons.Filled.Person),
    PRIVACY("Privacy & Safety", Icons.Rounded.Security, Icons.Filled.Security),
    CONNECTIONS("Connections", Icons.Rounded.Link, Icons.Filled.Link),
    DEVICES("Devices", Icons.Rounded.Tv, Icons.Filled.Tv),
    APPEARANCE("Appearance", Icons.Rounded.Palette, Icons.Filled.Palette),
    ACCESSIBILITY("Accessibility", Icons.Rounded.Accessibility, Icons.Filled.Accessibility),
    VOICE_VIDEO("Voice & Video", Icons.Rounded.Mic, Icons.Filled.Mic),
    CHAT("Chat", Icons.Rounded.Forum, Icons.Filled.Forum),
    NOTIFICATIONS("Notifications", Icons.Rounded.Notifications, Icons.Filled.Notifications),
    ADVANCED("Advanced", Icons.Rounded.Tune, Icons.Filled.Tune),
    ABOUT("About", Icons.Rounded.Info, Icons.Filled.Info),
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    sessionManager: SessionManager = koinInject(),
    onDismiss: () -> Unit
) {

    var selectedCategory by remember { mutableStateOf<SettingsSection?>(null) }
    var searchQuery by rememberSaveable { mutableStateOf("") }
    var showLogoutConfirmation by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current

    PlatformBackHandler(enabled = selectedCategory != null || searchQuery.isNotEmpty()) {
        if (searchQuery.isNotEmpty()) {
            searchQuery = ""
        } else {
            selectedCategory = null
        }
    }

    if (showLogoutConfirmation) {
        AlertDialog(
            onDismissRequest = { showLogoutConfirmation = false },
            icon = {
                Icon(
                    Icons.Filled.Logout,
                    contentDescription = null,
                    modifier = Modifier.size(32.dp),
                    tint = MaterialTheme.colorScheme.error
                )
            },
            title = {
                Text(
                    text = "Log Out",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            text = {
                Text(
                    text = "Are you sure you want to log out of Lampcord?",
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(
                    onClick = {
                        showLogoutConfirmation = false
                        sessionManager.disconnect()
                        onDismiss()
                    },
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError
                    ),
                    shape = MaterialTheme.shapes.medium
                ) {
                    Text("Log Out")
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { showLogoutConfirmation = false },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Cancel")
                }
            },
            shape = MaterialTheme.shapes.extraLarge,
            tonalElevation = AlertDialogDefaults.TonalElevation
        )
    }

    val searchEntries = rememberSettingsSearchEntries()
    val searchResults = remember(searchEntries, searchQuery) {
        searchEntries.filter { it.matches(searchQuery) }
    }

    fun openSearchEntry(entry: SettingsSearchEntry) {
        selectedCategory = SettingsSection.entries.find { it.title == entry.screen }
        searchQuery = ""
    }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp
        
        if (isCompact) {
            val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
            val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

            AnimatedContent(
                targetState = selectedCategory,
                modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
                transitionSpec = {
                    if (targetState != null) {
                        (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                            fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                        )
                    } else {
                        (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { -it / 8 }).togetherWith(
                            fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { it / 8 }
                        )
                    }
                },
                label = "SettingsTransition"
            ) { category ->
                if (category == null) {
                    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()
                    Scaffold(
                        modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
                        topBar = {
                            LargeTopAppBar(
                                title = { Text("Settings") },
                                navigationIcon = {
                                    IconButton(onClick = onDismiss) {
                                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                                    }
                                },
                                scrollBehavior = scrollBehavior
                            )
                        }
                    ) { padding ->
                        LazyColumn(
                            modifier = Modifier.fillMaxSize().padding(padding),
                            contentPadding = PaddingValues(bottom = 24.dp),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            item {
                                SettingsSearchField(
                                    query = searchQuery,
                                    onQueryChange = { searchQuery = it },
                                    onClear = { searchQuery = "" },
                                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
                                )
                            }

                            if (searchQuery.isBlank()) {
                                item {
                                    Material3SettingsGroup(
                                        title = "User Settings",
                                        items = listOf(
                                            Material3SettingsItem(
                                                Icons.Filled.AccountCircle,
                                                title = { Text("Account") },
                                                description = { Text("Manage your account details and security") },
                                                onClick = { selectedCategory = SettingsSection.ACCOUNT }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Person,
                                                title = { Text("Profiles") },
                                                description = { Text("Customize your appearance across servers") },
                                                onClick = { selectedCategory = SettingsSection.PROFILES }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Security,
                                                title = { Text("Privacy & Safety") },
                                                description = { Text("Manage who can contact you and what you see") },
                                                onClick = { selectedCategory = SettingsSection.PRIVACY }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Link,
                                                title = { Text("Connections") },
                                                description = { Text("Connect your accounts from other platforms") },
                                                onClick = { selectedCategory = SettingsSection.CONNECTIONS }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Tv,
                                                title = { Text("Devices") },
                                                description = { Text("Manage your active sessions") },
                                                onClick = { selectedCategory = SettingsSection.DEVICES }
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        title = "App Settings",
                                        items = listOf(
                                            Material3SettingsItem(
                                                Icons.Filled.Palette,
                                                title = { Text("Appearance") },
                                                description = { Text("Theme, colors, and message display") },
                                                onClick = { selectedCategory = SettingsSection.APPEARANCE }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Accessibility,
                                                title = { Text("Accessibility") },
                                                description = { Text("Visual and interactive adjustments") },
                                                onClick = { selectedCategory = SettingsSection.ACCESSIBILITY }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Mic,
                                                title = { Text("Voice & Video") },
                                                description = { Text("Input, output, and camera settings") },
                                                onClick = { selectedCategory = SettingsSection.VOICE_VIDEO }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Rounded.Forum,
                                                title = { Text("Chat") },
                                                description = { Text("Control how you interact with chat and media") },
                                                onClick = { selectedCategory = SettingsSection.CHAT }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Notifications,
                                                title = { Text("Notifications") },
                                                description = { Text("Control how you're notified") },
                                                onClick = { selectedCategory = SettingsSection.NOTIFICATIONS }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Tune,
                                                title = { Text("Advanced") },
                                                description = { Text("Developer settings and experimental features") },
                                                onClick = { selectedCategory = SettingsSection.ADVANCED }
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        items = listOf(
                                            Material3SettingsItem(
                                                Icons.Filled.Info,
                                                title = { Text("About") },
                                                description = { Text("App information and credits") },
                                                onClick = { selectedCategory = SettingsSection.ABOUT }
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        items = listOf(
                                            Material3SettingsItem(
                                                Icons.AutoMirrored.Filled.Logout,
                                                title = { Text("Log Out", color = MaterialTheme.colorScheme.error) },
                                                iconTint = MaterialTheme.colorScheme.error,
                                                onClick = {
                                                    showLogoutConfirmation = true
                                                }
                                            )
                                        )
                                    )
                                }
                            } else {
                                item {
                                    SettingsSearchResults(
                                        query = searchQuery,
                                        results = searchResults,
                                        onEntryClick = ::openSearchEntry
                                    )
                                }
                            }
                        }
                    }
                } else {
                    SettingsSubScreen(
                        title = category.title,
                        onNavigateBack = { selectedCategory = null },
                        contentScrollable = true
                    ) {
                        when (category) {
                            SettingsSection.ACCOUNT -> AccountSettings()
                            SettingsSection.PROFILES -> ProfileSettings()
                            SettingsSection.PRIVACY -> PrivacySettings()
                            SettingsSection.CONNECTIONS -> ConnectionsSettings()
                            SettingsSection.DEVICES -> DevicesSettings()
                            SettingsSection.APPEARANCE -> AppearanceSettings()
                            SettingsSection.ACCESSIBILITY -> AccessibilitySettings()
                            SettingsSection.VOICE_VIDEO -> { /* TODO */ }
                            SettingsSection.CHAT -> ChatSettings()
                            SettingsSection.NOTIFICATIONS -> NotificationsSettings()
                            SettingsSection.ADVANCED -> AdvancedSettings()
                            SettingsSection.ABOUT -> AboutContent(version = "1.0.0", onOpenUrl = { uriHandler.openUri(it) })
                        }
                    }
                }
            }
        } else {
            // Desktop Layout
            val railState = rememberWideNavigationRailState(initialValue = WideNavigationRailValue.Expanded)
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                SettingsDesktopOverlay(
                    selectedCategory = selectedCategory,
                    onDismiss = onDismiss,
                    onCategorySelected = { selectedCategory = it },
                    showLogoutConfirmation = showLogoutConfirmation,
                    onLogoutConfirmationChanged = { showLogoutConfirmation = it },
                    railState = railState
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SettingsDesktopOverlay(
    userStore: UserStore = koinInject(),
    selectedCategory: SettingsSection?,
    onDismiss: () -> Unit,
    onCategorySelected: (SettingsSection) -> Unit,
    showLogoutConfirmation: Boolean,
    onLogoutConfirmationChanged: (Boolean) -> Unit,
    railState: WideNavigationRailState
) {
    val activeCategory = selectedCategory ?: SettingsSection.ACCOUNT
    val uriHandler = LocalUriHandler.current

    Surface(
        modifier = Modifier
            .widthIn(max = 1200.dp)
            .fillMaxWidth(0.95f)
            .heightIn(max = 850.dp)
            .fillMaxHeight(0.9f)
            .clip(MaterialTheme.shapes.large),
        color = MaterialTheme.colorScheme.surfaceContainerLowest,
        shape = MaterialTheme.shapes.large
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Titlebar
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 8.dp, start = 16.dp, end = 16.dp)
            ) {
                Text(
                    text = "Settings",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.align(Alignment.Center)
                )
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.align(Alignment.CenterEnd)
                ) {
                    Icon(Icons.Default.Close, "Close", tint = MaterialTheme.colorScheme.onSurface, modifier = Modifier.size(20.dp))
                }
            }

            Row(modifier = Modifier.fillMaxSize()) {
                // Wide Navigation Rail
                WideNavigationRail(
                    state = railState,
                    colors = WideNavigationRailDefaults.colors(
                        containerColor = MaterialTheme.colorScheme.surfaceContainerLowest
                    ),
                    windowInsets = WindowInsets(0.dp),
                    contentPadding = PaddingValues(0.dp),
                    header = {
                        val scope = rememberCoroutineScope()
                        val isExpanded = railState.currentValue == WideNavigationRailValue.Expanded
                        Column(
                            horizontalAlignment = Alignment.Start,
                            modifier = Modifier.padding(start = 20.dp, top = 12.dp, bottom = 4.dp)
                        ) {
                            IconButton(onClick = { scope.launch { railState.toggle() } }) {
                                Icon(
                                    imageVector = if (isExpanded) Icons.AutoMirrored.Filled.MenuOpen else Icons.Filled.Menu,
                                    contentDescription = "Toggle Sidebar",
                                )
                            }

                            ExtendedFloatingActionButton(
                                onClick = { onCategorySelected(SettingsSection.PROFILES) },
                                expanded = isExpanded,
                                icon = { Icon(Icons.Filled.Edit, null) },
                                text = { Text("Edit profile") },
                                containerColor = MaterialTheme.colorScheme.primaryContainer,
                                contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                                shape = RoundedCornerShape(16.dp),
                                modifier = Modifier.padding(top = 8.dp)
                            )
                        }
                    }
                ) {
                    val railSections = listOf(
                        SettingsSection.ACCOUNT,
                        SettingsSection.APPEARANCE,
                        SettingsSection.CHAT,
                        SettingsSection.NOTIFICATIONS,
                        SettingsSection.ADVANCED,
                        SettingsSection.ABOUT
                    )

                    railSections.forEach { section ->
                        val isSelected = activeCategory == section

                        WideNavigationRailItem(
                            selected = isSelected,
                            railExpanded = railState.currentValue == WideNavigationRailValue.Expanded,
                            onClick = { onCategorySelected(section) },
                            icon = {
                                Icon(
                                    if (isSelected) section.selectedIcon else section.icon,
                                    contentDescription = section.title,
                                )
                            },
                            label = {
                                Text(
                                    section.title,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        )
                    }

                    // Logout
                    WideNavigationRailItem(
                        selected = false,
                        railExpanded = railState.currentValue == WideNavigationRailValue.Expanded,
                        onClick = { onLogoutConfirmationChanged(true) },
                        icon = {
                            Icon(
                                Icons.AutoMirrored.Filled.Logout,
                                contentDescription = "Log Out",
                                tint = MaterialTheme.colorScheme.error,
                            )
                        },
                        label = {
                            Text(
                                "Log Out",
                                color = MaterialTheme.colorScheme.error,
                                maxLines = 1
                            )
                        }
                    )
                }

                // Main Content Area
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight()
                        .padding(end = 8.dp, bottom = 8.dp),
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.surfaceContainerLow
                ) {
                    AnimatedContent(
                        targetState = activeCategory,
                        transitionSpec = {
                            (fadeIn(animationSpec = tween(300)) + slideInVertically(animationSpec = tween(300)) { 20 }).togetherWith(
                                fadeOut(animationSpec = tween(200))
                            )
                        },
                        modifier = Modifier.fillMaxSize(),
                        label = "settingsContent",
                    ) { section ->
                        val scrollState = rememberScrollState()
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(scrollState)
                                .padding(24.dp),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            when (section) {
                                SettingsSection.ACCOUNT -> AccountSettings()
                                SettingsSection.PROFILES -> ProfileSettings()
                                SettingsSection.PRIVACY -> PrivacySettings()
                                SettingsSection.CONNECTIONS -> ConnectionsSettings()
                                SettingsSection.DEVICES -> DevicesSettings()
                                SettingsSection.APPEARANCE -> AppearanceSettings()
                                SettingsSection.ACCESSIBILITY -> AccessibilitySettings()
                                SettingsSection.CHAT -> ChatSettings()
                                SettingsSection.NOTIFICATIONS -> NotificationsSettings()
                                SettingsSection.ADVANCED -> AdvancedSettings()
                                SettingsSection.ABOUT -> AboutContent(version = "1.0.0", onOpenUrl = { uriHandler.openUri(it) })
                                else -> {
                                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        Text("Feature coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
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

@Composable
fun rememberSettingsSearchEntries(): List<SettingsSearchEntry> {
    return buildList {
        add(SettingsSearchEntry("root-account", "Account", "Manage your account details and security", "Account", "User Settings", "password email phone", Icons.Filled.AccountCircle, "blue", SettingsSearchDestination.Account))
        add(SettingsSearchEntry("root-profiles", "Profiles", "Customize your appearance across servers", "Profiles", "User Settings", "avatar banner bio", Icons.Filled.Person, "rose", SettingsSearchDestination.Profiles))
        add(SettingsSearchEntry("root-connections", "Connections", "Connect your accounts from other platforms", "Connections", "User Settings", "spotify steam twitch github", Icons.Filled.Link, "cyan", SettingsSearchDestination.Connections))
        add(SettingsSearchEntry("root-devices", "Devices", "Manage your active sessions", "Devices", "User Settings", "login session security", Icons.Filled.Tv, "gold", SettingsSearchDestination.Devices))
        add(SettingsSearchEntry("root-appearance", "Appearance", "Theme, colors, and message display", "Appearance", "App Settings", "dark mode light amoled color nitro compact", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("root-accessibility", "Accessibility", "Visual and interactive adjustments", "Accessibility", "App Settings", "font size saturation reduce motion", Icons.Filled.Accessibility, "green", SettingsSearchDestination.Accessibility))
        add(SettingsSearchEntry("root-voice", "Voice & Video", "Input, output, and camera settings", "Voice & Video", "App Settings", "microphone camera noise suppression", Icons.Filled.Mic, "rose", SettingsSearchDestination.VoiceVideo))
        add(SettingsSearchEntry("root-chat", "Chat", "Control how you interact with chat and media", "Chat", "App Settings", "gestures tap swipe message display", Icons.Rounded.Forum, "rose", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-notifications", "Notifications", "Control how you're notified", "Notifications", "App Settings", "push mentions sounds", Icons.Filled.Notifications, "rose", SettingsSearchDestination.Notifications))
        add(SettingsSearchEntry("root-about", "About", "App information and credits", "About", "App Settings", "version info credits developer", Icons.Filled.Info, "neutral", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-logout", "Log Out", "Sign out of your account", "Logout", "Account", "sign out exit", Icons.AutoMirrored.Filled.Logout, "neutral", SettingsSearchDestination.Logout))
        
        // Deep search entries
        add(SettingsSearchEntry("appearance-theme", "Theme Mode", "Auto, Light, Dark, or AMOLED", "Appearance", "Theme", "dark light amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-pure-black", "Pure Black", "Use pure black backgrounds in dark mode", "Appearance", "Theme", "amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-compact", "Compact Messages", "Denser layout for chat", "Appearance", "Display", "compact message denser", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-bubbles", "Chat Bubbles", "Display messages inside rounded chat bubbles", "Appearance", "Display", "chat bubbles message layout theme", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("advanced-dev", "Developer Mode", "Exposes ID copying and debug tools", "Advanced", "Developer Settings", "id debug", Icons.Filled.Tune, "neutral", SettingsSearchDestination.Advanced))
    }
}
