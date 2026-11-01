package me.lampu.lampcord.shared.ui

import me.lampu.lampcord.shared.generated.resources.accessibility
import me.lampu.lampcord.shared.generated.resources.account
import me.lampu.lampcord.shared.generated.resources.appearance
import me.lampu.lampcord.shared.generated.resources.connections
import me.lampu.lampcord.shared.generated.resources.guild_template_name_category_chat
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_advanced
import me.lampu.lampcord.shared.generated.resources.language
import me.lampu.lampcord.shared.generated.resources.notifications
import me.lampu.lampcord.shared.generated.resources.privacy_and_safety
import me.lampu.lampcord.shared.generated.resources.voice_and_video

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
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
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchDestination
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchEntry
import me.lampu.lampcord.shared.ui.components.settings.SettingsOverlayFrame
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchField
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchResults
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.AccessibilitySettingsContent
import me.lampu.lampcord.shared.ui.settings.AccountSettingsContent
import me.lampu.lampcord.shared.ui.settings.AdditionalSettingsContent
import me.lampu.lampcord.shared.ui.settings.AdvancedSettingsContent
import me.lampu.lampcord.shared.ui.settings.AppearanceSettingsContent
import me.lampu.lampcord.shared.ui.settings.ChatSettingsContent
import me.lampu.lampcord.shared.ui.settings.ConnectionsSettingsContent
import me.lampu.lampcord.shared.ui.settings.DevicesSettingsContent
import me.lampu.lampcord.shared.ui.settings.LanguageSettingsContent
import me.lampu.lampcord.shared.ui.settings.NotificationsSettingsContent
import me.lampu.lampcord.shared.ui.settings.PrivacySettingsContent
import me.lampu.lampcord.shared.ui.settings.ProfileSettingsContent
import me.lampu.lampcord.shared.ui.settings.ThemeEditorScreen
import me.lampu.lampcord.shared.ui.settings.ThemingSettingsContent
import me.lampu.lampcord.shared.ui.settings.UpdatesContent
import me.lampu.lampcord.shared.ui.settings.VoiceSettingsContent
import me.lampu.lampcord.shared.ui.settings.DebugLogScreen
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.app_settings
import me.lampu.lampcord.shared.generated.resources.back
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.lampcord_language_description
import me.lampu.lampcord.shared.generated.resources.logout
import me.lampu.lampcord.shared.generated.resources.more_options
import me.lampu.lampcord.shared.generated.resources.settings
import me.lampu.lampcord.shared.generated.resources.user_settings
import me.lampu.lampcord.shared.generated.resources.user_settings_confirm_logout
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

enum class SettingsSection(val title: String, val icon: ImageVector, val selectedIcon: ImageVector) {
    ACCOUNT("Account", Icons.Rounded.AccountCircle, Icons.Filled.AccountCircle),
    PROFILES("Profiles", Icons.Rounded.Person, Icons.Filled.Person),
    PRIVACY("Privacy & Safety", Icons.Rounded.Security, Icons.Filled.Security),
    CONNECTIONS("Connections", Icons.Rounded.Link, Icons.Filled.Link),
    DEVICES("Devices", Icons.Rounded.Tv, Icons.Filled.Tv),
    APPEARANCE("Appearance", Icons.Rounded.Palette, Icons.Filled.Palette),
    ACCESSIBILITY("Accessibility", Icons.Rounded.Accessibility, Icons.Filled.Accessibility),
    LANGUAGE("Language", Icons.Rounded.Language, Icons.Filled.Language),
    VOICE_VIDEO("Voice & Video", Icons.Rounded.Mic, Icons.Filled.Mic),
    CHAT("Chat", Icons.Rounded.Forum, Icons.Filled.Forum),
    NOTIFICATIONS("Notifications", Icons.Rounded.Notifications, Icons.Filled.Notifications),
    ADVANCED("Advanced", Icons.Rounded.Tune, Icons.Filled.Tune),
    ADDITIONAL("Additional", Icons.Rounded.AutoAwesome, Icons.Filled.AutoAwesome),
    UPDATES("Updates", Icons.Rounded.Refresh, Icons.Filled.Refresh),
    ABOUT("About", Icons.Rounded.Info, Icons.Filled.Info),
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SettingsScreen(
    sessionManager: SessionManager = koinInject(),
    onNavigateToAccount: () -> Unit = {},
    onNavigateToProfiles: () -> Unit = {},
    onNavigateToPrivacy: () -> Unit = {},
    onNavigateToConnections: () -> Unit = {},
    onNavigateToDevices: () -> Unit = {},
    onNavigateToAppearance: () -> Unit = {},
    onNavigateToAccessibility: () -> Unit = {},
    onNavigateToLanguage: () -> Unit = {},
    onNavigateToVoice: () -> Unit = {},
    onNavigateToChat: () -> Unit = {},
    onNavigateToNotifications: () -> Unit = {},
    onNavigateToAdvanced: () -> Unit = {},
    onNavigateToAdditional: () -> Unit = {},
    onNavigateToAbout: () -> Unit = {},
    onNavigateToUpdates: () -> Unit = {},
    onNavigateToTheming: () -> Unit = {},
    onNavigateToNavigation: () -> Unit = {},
    onDismiss: () -> Unit,
    navigationStore: NavigationStore = koinInject()
) {

    val reduceMotion = Settings.shared.reduceMotion

    var selectedCategory by remember { mutableStateOf<SettingsSection?>(null) }

    var showThemer by remember { mutableStateOf(false) }
    var editingThemeJson by remember { mutableStateOf<String?>(null) }
    var searchQuery by rememberSaveable { mutableStateOf("") }
    var showLogoutConfirmation by remember { mutableStateOf(false) }
    var showOptionsMenu by remember { mutableStateOf(false) }
    var showDebugLogs by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current
    val scope = rememberCoroutineScope()

    PlatformBackHandler(enabled = selectedCategory != null || searchQuery.isNotEmpty() || showThemer || editingThemeJson != null || showDebugLogs) {
        if (showDebugLogs) {
            showDebugLogs = false
        } else if (editingThemeJson != null) {
            editingThemeJson = null
        } else if (showThemer) {
            showThemer = false
        } else if (searchQuery.isNotEmpty()) {
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
                    text = stringResource(Res.string.logout),
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            text = {
                Text(
                    text = stringResource(Res.string.user_settings_confirm_logout),
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
                    shape = CircleShape
                ) {
                    Text(stringResource(Res.string.logout))
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { showLogoutConfirmation = false },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(stringResource(Res.string.cancel))
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

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp

        LaunchedEffect(navigationStore.settingsCategory) {
            navigationStore.settingsCategory?.let { categoryName ->
                if (isCompact) {
                    when (categoryName) {
                        "ACCOUNT" -> onNavigateToAccount()
                        "PROFILES" -> onNavigateToProfiles()
                        "PRIVACY" -> onNavigateToPrivacy()
                        "CONNECTIONS" -> onNavigateToConnections()
                        "DEVICES" -> onNavigateToDevices()
                        "APPEARANCE" -> onNavigateToAppearance()
                        "ACCESSIBILITY" -> onNavigateToAccessibility()
                        "LANGUAGE" -> onNavigateToLanguage()
                        "VOICE_VIDEO" -> onNavigateToVoice()
                        "CHAT" -> onNavigateToChat()
                        "NOTIFICATIONS" -> onNavigateToNotifications()
                        "ADVANCED" -> onNavigateToAdvanced()
                        "ADDITIONAL" -> onNavigateToAdditional()
                        "UPDATES" -> onNavigateToUpdates()
                        "ABOUT" -> onNavigateToAbout()
                    }
                    navigationStore.settingsCategory = null
                } else {
                    SettingsSection.entries.find { it.name == categoryName }?.let {
                        selectedCategory = it
                        navigationStore.settingsCategory = null
                    }
                }
            }
        }
        
        val finalOnNavigateToTheming = {
            if (isCompact) {
                onNavigateToTheming()
            } else {
                showThemer = true
            }
        }

        val finalOnNavigateToNavigation = {
            if (isCompact) {
                onNavigateToNavigation()
            } else {
                selectedCategory = SettingsSection.APPEARANCE
            }
        }

        fun openSearchEntry(entry: SettingsSearchEntry) {
            if (entry.destination == SettingsSearchDestination.Theming) {
                finalOnNavigateToTheming()
            } else {
                selectedCategory = SettingsSection.entries.find { it.title == entry.screen }
            }
            searchQuery = ""
        }

        if (isCompact) {
            val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
            val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

            AnimatedContent(
                targetState = showDebugLogs,
                modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
                transitionSpec = {
                    if (reduceMotion) {
                        EnterTransition.None togetherWith ExitTransition.None
                    } else if (targetState) {
                        (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                            fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                        )
                    } else {
                        (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { -it / 8 }).togetherWith(
                            fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { it / 8 }
                        )
                    }.using(SizeTransform(clip = false) { _, _ -> tween(0) })
                },
                label = "SettingsTransition"
            ) { showLogs ->
                if (showLogs) {
                    DebugLogScreen(onBack = { showDebugLogs = false })
                } else {
                    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()
                    Scaffold(
                        modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
                        containerColor = MaterialTheme.colorScheme.surface,
                        topBar = {
                            LargeTopAppBar(
                                title = { Text(stringResource(Res.string.settings)) },
                                navigationIcon = {
                                    IconButton(onClick = onDismiss) {
                                        Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = stringResource(Res.string.back))
                                    }
                                },
                                colors = TopAppBarDefaults.topAppBarColors(
                                    containerColor = MaterialTheme.colorScheme.surface,
                                    scrolledContainerColor = MaterialTheme.colorScheme.surfaceContainer
                                ),
                                actions = {
                                    Box {
                                        IconButton(onClick = { showOptionsMenu = true }) {
                                            Icon(Icons.Rounded.MoreVert, contentDescription = stringResource(Res.string.more_options))
                                        }
                                        androidx.compose.material3.DropdownMenu(
                                            expanded = showOptionsMenu,
                                            onDismissRequest = { showOptionsMenu = false }
                                        ) {
                                            androidx.compose.material3.DropdownMenuItem(
                                                text = { Text("Restart") },
                                                onClick = {
                                                    showOptionsMenu = false
                                                    me.lampu.lampcord.shared.utils.restartApp()
                                                },
                                                leadingIcon = { Icon(Icons.Rounded.Refresh, null) }
                                            )
                                            androidx.compose.material3.DropdownMenuItem(
                                                text = { Text("View Debug Logs") },
                                                onClick = {
                                                    showOptionsMenu = false
                                                    showDebugLogs = true
                                                },
                                                leadingIcon = { Icon(Icons.Rounded.Info, null) }
                                            )
                                            androidx.compose.material3.DropdownMenuItem(
                                                text = { Text(stringResource(Res.string.logout)) },
                                                onClick = {
                                                    showOptionsMenu = false
                                                    showLogoutConfirmation = true
                                                },
                                                leadingIcon = { Icon(Icons.AutoMirrored.Rounded.Logout, null) }
                                            )
                                        }
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
                                val platform = me.lampu.lampcord.shared.utils.getPlatformName()
                                val useRounded = platform == "android"

                                item {
                                    Material3SettingsGroup(
                                        title = stringResource(Res.string.user_settings),
                                        items = listOf(
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.AccountCircle else Icons.Filled.AccountCircle,
                                                title = { Text(stringResource(Res.string.account)) },
                                                description = { Text("Manage your account details and security") },
                                                onClick = onNavigateToAccount
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Person else Icons.Filled.Person,
                                                title = { Text("Profiles") },
                                                description = { Text("Customize your appearance across servers") },
                                                onClick = onNavigateToProfiles
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Security else Icons.Filled.Security,
                                                title = { Text(stringResource(Res.string.privacy_and_safety)) },
                                                description = { Text("Manage who can contact you and what you see") },
                                                onClick = onNavigateToPrivacy
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Link else Icons.Filled.Link,
                                                title = { Text(stringResource(Res.string.connections)) },
                                                description = { Text("Connect your accounts from other platforms") },
                                                onClick = onNavigateToConnections
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Tv else Icons.Filled.Tv,
                                                title = { Text("Devices") },
                                                description = { Text("Manage your active sessions") },
                                                onClick = onNavigateToDevices
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        title = stringResource(Res.string.app_settings),
                                        items = listOf(
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Palette else Icons.Filled.Palette,
                                                title = { Text(stringResource(Res.string.appearance)) },
                                                description = { Text("Theme, colors, and message display") },
                                                onClick = onNavigateToAppearance
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Accessibility else Icons.Filled.Accessibility,
                                                title = { Text(stringResource(Res.string.accessibility)) },
                                                description = { Text("Visual and interactive adjustments") },
                                                onClick = onNavigateToAccessibility
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Language else Icons.Filled.Language,
                                                title = { Text(stringResource(Res.string.language)) },
                                                description = { Text(stringResource(Res.string.lampcord_language_description)) },
                                                onClick = onNavigateToLanguage
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Mic else Icons.Filled.Mic,
                                                title = { Text(stringResource(Res.string.voice_and_video)) },
                                                description = { Text("Input, output, and camera settings") },
                                                onClick = onNavigateToVoice
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Forum else Icons.Filled.Forum,
                                                title = { Text(stringResource(Res.string.guild_template_name_category_chat)) },
                                                description = { Text("Control how you interact with chat and media") },
                                                onClick = onNavigateToChat
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Notifications else Icons.Filled.Notifications,
                                                title = { Text(stringResource(Res.string.notifications)) },
                                                description = { Text("Control how you're notified") },
                                                onClick = onNavigateToNotifications
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Tune else Icons.Filled.Tune,
                                                title = { Text(stringResource(Res.string.lampcord_settings_advanced)) },
                                                description = { Text("Developer settings and experimental features") },
                                                onClick = onNavigateToAdvanced
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.AutoAwesome else Icons.Filled.AutoAwesome,
                                                title = { Text("Additional") },
                                                description = { Text("Profile enhancements, emoji tweaks, logger and text replacement") },
                                                onClick = onNavigateToAdditional
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        items = listOf(
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Refresh else Icons.Filled.Refresh,
                                                title = { Text("Updates") },
                                                description = { Text("Check for updates and read the changelog") },
                                                onClick = onNavigateToUpdates
                                            ),
                                            Material3SettingsItem(
                                                if (useRounded) Icons.Rounded.Info else Icons.Filled.Info,
                                                title = { Text("About") },
                                                description = { Text("App information and credits") },
                                                onClick = onNavigateToAbout
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        items = listOf(
                                            Material3SettingsItem(
                                                if (useRounded) Icons.AutoMirrored.Rounded.Logout else Icons.AutoMirrored.Filled.Logout,
                                                title = { Text(stringResource(Res.string.logout), color = MaterialTheme.colorScheme.error) },
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
                }
            }
        } else {
            SettingsOverlayFrame(onDismiss = onDismiss) {
                    SettingsDesktopOverlay(
                        selectedCategory = selectedCategory,
                        showThemer = showThemer,
                        editingThemeJson = editingThemeJson,
                        showDebugLogs = showDebugLogs,
                        searchQuery = searchQuery,
                        searchResults = searchResults,
                        onSearchQueryChange = { searchQuery = it },
                        onClearSearch = { searchQuery = "" },
                        onSearchEntryClick = { entry ->
                            if (entry.destination == SettingsSearchDestination.Logout) {
                                showLogoutConfirmation = true
                            } else {
                                openSearchEntry(entry)
                            }
                        },
                        onShowThemerChanged = { showThemer = it },
                        onEditingThemeJsonChanged = { editingThemeJson = it },
                        onShowDebugLogsChanged = { showDebugLogs = it },
                        onNavigateToTheming = finalOnNavigateToTheming,
                        onNavigateToNavigation = finalOnNavigateToNavigation,
                        onDismiss = onDismiss,
                        onCategorySelected = { selectedCategory = it },
                        onLogoutConfirmationChanged = { showLogoutConfirmation = it },
                        finalOnNavigateToTheming = finalOnNavigateToTheming,
                        finalOnNavigateToNavigation = finalOnNavigateToNavigation
                    )
            }
        }
    }
}

@Composable
fun SettingsDesktopOverlay(
    selectedCategory: SettingsSection?,
    showThemer: Boolean,
    editingThemeJson: String?,
    showDebugLogs: Boolean,
    searchQuery: String,
    searchResults: List<SettingsSearchEntry>,
    onSearchQueryChange: (String) -> Unit,
    onClearSearch: () -> Unit,
    onSearchEntryClick: (SettingsSearchEntry) -> Unit,
    onShowThemerChanged: (Boolean) -> Unit,
    onEditingThemeJsonChanged: (String?) -> Unit,
    onShowDebugLogsChanged: (Boolean) -> Unit,
    onNavigateToTheming: () -> Unit = {},
    onNavigateToNavigation: () -> Unit = {},
    onDismiss: () -> Unit,
    onCategorySelected: (SettingsSection) -> Unit,
    onLogoutConfirmationChanged: (Boolean) -> Unit,
    finalOnNavigateToTheming: () -> Unit = {},
    finalOnNavigateToNavigation: () -> Unit = {}
) {
    val activeCategory = selectedCategory ?: SettingsSection.ACCOUNT
    val reduceMotion = Settings.shared.reduceMotion
    val uriHandler = LocalUriHandler.current

    Row(modifier = Modifier.fillMaxSize()) {
        SettingsDesktopNavigationPane(
            selectedCategory = activeCategory,
            showThemer = showThemer,
            editingThemeJson = editingThemeJson,
            showDebugLogs = showDebugLogs,
            searchQuery = searchQuery,
            searchResults = searchResults,
            onSearchQueryChange = onSearchQueryChange,
            onClearSearch = onClearSearch,
            onSearchEntryClick = onSearchEntryClick,
            onCategorySelected = onCategorySelected,
            onShowThemerChanged = onShowThemerChanged,
            onEditingThemeJsonChanged = onEditingThemeJsonChanged,
            onShowDebugLogsChanged = onShowDebugLogsChanged,
            onLogoutConfirmationChanged = onLogoutConfirmationChanged,
            modifier = Modifier.width(360.dp).fillMaxHeight()
        )

        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .padding(end = 12.dp, bottom = 12.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(80.dp)
                    .padding(start = 32.dp, end = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val title = when {
                    editingThemeJson != null -> "Theme Editor"
                    showThemer -> "Themer"
                    showDebugLogs -> "Debug Logs"
                    else -> activeCategory.title
                }
                AnimatedContent(
                    targetState = title,
                    transitionSpec = {
                        if (reduceMotion) {
                            EnterTransition.None togetherWith ExitTransition.None
                        } else {
                            fadeIn(tween(150)) togetherWith fadeOut(tween(100))
                        }
                    },
                    contentAlignment = Alignment.CenterStart,
                    label = "desktopSettingsTitle",
                    modifier = Modifier.weight(1f)
                ) { currentTitle ->
                    Text(
                        text = currentTitle,
                        style = MaterialTheme.typography.headlineMedium,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, "Close", modifier = Modifier.size(20.dp))
                }
            }

            Surface(
                modifier = Modifier.fillMaxWidth().weight(1f),
                shape = RoundedCornerShape(28.dp),
                color = MaterialTheme.colorScheme.surfaceContainerLow
            ) {
                val navigationState = remember(editingThemeJson, showThemer, showDebugLogs, activeCategory) {
                    when {
                        editingThemeJson != null -> "editor" to editingThemeJson
                        showThemer -> "themer" to null
                        showDebugLogs -> "logs" to null
                        else -> activeCategory.name to null
                    }
                }

                AnimatedContent(
                    targetState = navigationState,
                    transitionSpec = {
                        if (reduceMotion) {
                            EnterTransition.None togetherWith ExitTransition.None
                        } else {
                            (fadeIn(animationSpec = tween(300)) + slideInVertically(animationSpec = tween(300)) { 20 }).togetherWith(
                                fadeOut(animationSpec = tween(200))
                            )
                        }
                    },
                    modifier = Modifier.fillMaxSize(),
                    label = "settingsContent",
                ) { (target, data) ->
                    if (target == "editor") {
                        ThemeEditorScreen(themeJson = data!!, onBack = { onEditingThemeJsonChanged(null) })
                    } else if (target == "themer") {
                        ThemingSettingsContent(onNavigateToEditor = { onEditingThemeJsonChanged(it) })
                    } else if (target == "logs") {
                        DebugLogScreen(onBack = { onShowDebugLogsChanged(false) })
                    } else {
                        val scrollState = rememberScrollState()
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(scrollState),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            when (activeCategory) {
                                SettingsSection.ACCOUNT -> {
                                    val userStore: UserStore = koinInject()
                                    val user by userStore.currentUser.collectAsState()
                                    user?.let { AccountSettingsContent(userVal = it) }
                                }
                                SettingsSection.PROFILES -> {
                                    val userStore: UserStore = koinInject()
                                    val user by userStore.currentUser.collectAsState()
                                    user?.let { ProfileSettingsContent(userVal = it) }
                                }
                                SettingsSection.PRIVACY -> PrivacySettingsContent()
                                SettingsSection.CONNECTIONS -> ConnectionsSettingsContent()
                                SettingsSection.DEVICES -> DevicesSettingsContent()
                                SettingsSection.APPEARANCE -> AppearanceSettingsContent(
                                    onNavigateToTheming = finalOnNavigateToTheming,
                                    onNavigateToNavigation = finalOnNavigateToNavigation
                                )
                                SettingsSection.ACCESSIBILITY -> AccessibilitySettingsContent()
                                SettingsSection.LANGUAGE -> LanguageSettingsContent()
                                SettingsSection.VOICE_VIDEO -> VoiceSettingsContent()
                                SettingsSection.CHAT -> ChatSettingsContent()
                                SettingsSection.NOTIFICATIONS -> NotificationsSettingsContent()
                                SettingsSection.ADVANCED -> AdvancedSettingsContent()
                                SettingsSection.ADDITIONAL -> AdditionalSettingsContent()
                                SettingsSection.UPDATES -> UpdatesContent()
                                SettingsSection.ABOUT -> AboutContent(
                                    version = me.lampu.lampcord.shared.update.APP_VERSION,
                                    onOpenUrl = { uriHandler.openUri(it) }
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SettingsDesktopNavigationPane(
    selectedCategory: SettingsSection,
    showThemer: Boolean,
    editingThemeJson: String?,
    showDebugLogs: Boolean,
    searchQuery: String,
    searchResults: List<SettingsSearchEntry>,
    onSearchQueryChange: (String) -> Unit,
    onClearSearch: () -> Unit,
    onSearchEntryClick: (SettingsSearchEntry) -> Unit,
    onCategorySelected: (SettingsSection) -> Unit,
    onShowThemerChanged: (Boolean) -> Unit,
    onEditingThemeJsonChanged: (String?) -> Unit,
    onShowDebugLogsChanged: (Boolean) -> Unit,
    onLogoutConfirmationChanged: (Boolean) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.padding(horizontal = 20.dp, vertical = 16.dp)) {
        SettingsSearchField(
            query = searchQuery,
            onQueryChange = onSearchQueryChange,
            onClear = onClearSearch
        )
        Spacer(Modifier.height(16.dp))
        if (searchQuery.isBlank()) {
            val active = !showThemer && editingThemeJson == null && !showDebugLogs
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(4.dp),
                contentPadding = PaddingValues(bottom = 16.dp)
            ) {
                desktopSettingsGroups.forEach { group ->
                    item(key = "group:${group.title}") {
                        Text(
                            text = group.title,
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(start = 16.dp, top = 16.dp, bottom = 8.dp)
                        )
                    }
                    itemsIndexed(group.sections, key = { _, section -> section.name }) { index, section ->
                        val selected = active && selectedCategory == section
                        DesktopSettingsNavItem(
                            title = section.title,
                            subtitle = section.subtitle(),
                            icon = if (selected) section.selectedIcon else section.icon,
                            selected = selected,
                            first = index == 0,
                            last = index == group.sections.lastIndex,
                            onClick = {
                                onCategorySelected(section)
                                onShowThemerChanged(false)
                                onEditingThemeJsonChanged(null)
                                onShowDebugLogsChanged(false)
                            }
                        )
                    }
                }
                item { Spacer(Modifier.height(8.dp)) }
                item {
                    DesktopSettingsNavItem(
                        title = stringResource(Res.string.logout),
                        subtitle = "Sign out of your account",
                        icon = Icons.AutoMirrored.Filled.Logout,
                        selected = false,
                        first = true,
                        last = true,
                        destructive = true,
                        onClick = { onLogoutConfirmationChanged(true) }
                    )
                }
            }
        } else {
            SettingsSearchResults(
                query = searchQuery,
                results = searchResults,
                onEntryClick = onSearchEntryClick,
                modifier = Modifier.fillMaxSize(),
                horizontalPadding = 0.dp,
                showHeader = false
            )
        }
    }
}

@Composable
private fun DesktopSettingsNavItem(
    title: String,
    subtitle: String,
    icon: ImageVector,
    selected: Boolean,
    first: Boolean,
    last: Boolean,
    onClick: () -> Unit,
    destructive: Boolean = false
) {
    val background by animateColorAsState(
        targetValue = if (selected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer,
        label = "settingsNavigationItem"
    )
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .clip(connectedSettingsShape(first, last))
            .background(background)
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(modifier = Modifier.size(32.dp), contentAlignment = Alignment.Center) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(24.dp)
            )
        }
        Spacer(Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                color = if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodyMedium,
                color = if (destructive) MaterialTheme.colorScheme.error.copy(alpha = 0.8f) else MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

private fun connectedSettingsShape(first: Boolean, last: Boolean) = RoundedCornerShape(
    topStart = if (first) 20.dp else 5.dp,
    topEnd = if (first) 20.dp else 5.dp,
    bottomStart = if (last) 20.dp else 5.dp,
    bottomEnd = if (last) 20.dp else 5.dp
)

private fun SettingsSection.subtitle(): String = when (this) {
    SettingsSection.ACCOUNT -> "Manage your account details and security"
    SettingsSection.PROFILES -> "Customize your appearance across servers"
    SettingsSection.PRIVACY -> "Manage who can contact you and what you see"
    SettingsSection.CONNECTIONS -> "Connect your accounts from other platforms"
    SettingsSection.DEVICES -> "Manage your active sessions"
    SettingsSection.APPEARANCE -> "Theme, colors, and message display"
    SettingsSection.ACCESSIBILITY -> "Visual and interactive adjustments"
    SettingsSection.LANGUAGE -> "Select your preferred language"
    SettingsSection.VOICE_VIDEO -> "Input, output, and camera settings"
    SettingsSection.CHAT -> "Control how you interact with chat and media"
    SettingsSection.NOTIFICATIONS -> "Control how you're notified"
    SettingsSection.ADVANCED -> "Developer settings and experimental features"
    SettingsSection.ADDITIONAL -> "Profile enhancements, emoji tweaks, logger and text replacement"
    SettingsSection.UPDATES -> "Check for updates and read the changelog"
    SettingsSection.ABOUT -> "App information and credits"
}

private data class DesktopSettingsGroup(val title: String, val sections: List<SettingsSection>)

private val desktopSettingsGroups = listOf(
    DesktopSettingsGroup(
        "User Settings",
        listOf(
            SettingsSection.ACCOUNT,
            SettingsSection.PROFILES,
            SettingsSection.PRIVACY,
            SettingsSection.CONNECTIONS,
            SettingsSection.DEVICES
        )
    ),
    DesktopSettingsGroup(
        "App Settings",
        listOf(
            SettingsSection.APPEARANCE,
            SettingsSection.ACCESSIBILITY,
            SettingsSection.LANGUAGE,
            SettingsSection.VOICE_VIDEO,
            SettingsSection.CHAT,
            SettingsSection.NOTIFICATIONS,
            SettingsSection.ADVANCED,
            SettingsSection.ADDITIONAL
        )
    ),
    DesktopSettingsGroup(
        "About",
        listOf(SettingsSection.UPDATES, SettingsSection.ABOUT)
    )
)

@Composable
fun rememberSettingsSearchEntries(): List<SettingsSearchEntry> {
    val platform = me.lampu.lampcord.shared.utils.getPlatformName()
    val useRounded = platform == "android"
    
    fun getIcon(rounded: ImageVector, filled: ImageVector): ImageVector {
        return if (useRounded) rounded else filled
    }

    return buildList {
        add(SettingsSearchEntry("root-account", "Account", "Manage your account details and security", "Account", "User Settings", "password email phone", getIcon(Icons.Rounded.AccountCircle, Icons.Filled.AccountCircle), "blue", SettingsSearchDestination.Account))
        add(SettingsSearchEntry("root-profiles", "Profiles", "Customize your appearance across servers", "Profiles", "User Settings", "avatar banner bio", getIcon(Icons.Rounded.Person, Icons.Filled.Person), "rose", SettingsSearchDestination.Profiles))
        add(SettingsSearchEntry("root-connections", "Connections", "Connect your accounts from other platforms", "Connections", "User Settings", "spotify steam twitch github", getIcon(Icons.Rounded.Link, Icons.Filled.Link), "cyan", SettingsSearchDestination.Connections))
        add(SettingsSearchEntry("root-devices", "Devices", "Manage your active sessions", "Devices", "User Settings", "login session security", getIcon(Icons.Rounded.Tv, Icons.Filled.Tv), "gold", SettingsSearchDestination.Devices))
        add(SettingsSearchEntry("root-appearance", "Appearance", "Theme, colors, and message display", "Appearance", "App Settings", "dark mode light amoled color nitro compact", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("root-accessibility", "Accessibility", "Visual and interactive adjustments", "Accessibility", "App Settings", "font size saturation reduce motion", getIcon(Icons.Rounded.Accessibility, Icons.Filled.Accessibility), "green", SettingsSearchDestination.Accessibility))
        add(SettingsSearchEntry("root-language", "Language", "Select your preferred language", "Language", "App Settings", "language locale translation region", getIcon(Icons.Rounded.Language, Icons.Filled.Language), "blue", SettingsSearchDestination.Language))
        add(SettingsSearchEntry("root-voice", "Voice & Video", "Input, output, and camera settings", "Voice & Video", "App Settings", "microphone camera noise suppression", getIcon(Icons.Rounded.Mic, Icons.Filled.Mic), "rose", SettingsSearchDestination.VoiceVideo))
        add(SettingsSearchEntry("root-chat", "Chat", "Control how you interact with chat and media", "Chat", "App Settings", "gestures tap swipe message display", getIcon(Icons.Rounded.Forum, Icons.Filled.Forum), "rose", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-notifications", "Notifications", "Control how you're notified", "Notifications", "App Settings", "push mentions sounds", getIcon(Icons.Rounded.Notifications, Icons.Filled.Notifications), "rose", SettingsSearchDestination.Notifications))
        add(SettingsSearchEntry("root-about", "About", "App information and credits", "About", "App Settings", "version info credits developer", getIcon(Icons.Rounded.Info, Icons.Filled.Info), "neutral", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-themer", "Themer", "Manage and edit themes", "Themer", "App Settings", "theming colors custom aliucord", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Theming))
        add(SettingsSearchEntry("root-logout", "Log Out", "Sign out of your account", "Logout", "Account", "sign out exit", getIcon(Icons.AutoMirrored.Rounded.Logout, Icons.AutoMirrored.Filled.Logout), "neutral", SettingsSearchDestination.Logout))
        
        add(SettingsSearchEntry("appearance-theme", "Theme Mode", "Auto, Light, Dark, or AMOLED", "Appearance", "Theme", "dark light amoled", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-pure-black", "Pure Black", "Use pure black backgrounds in dark mode", "Appearance", "Theme", "amoled", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-compact", "Compact Messages", "Denser layout for chat", "Appearance", "Display", "compact message denser", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-bubbles", "Chat Bubbles", "Display messages inside rounded chat bubbles", "Appearance", "Display", "chat bubbles message layout theme", getIcon(Icons.Rounded.Palette, Icons.Filled.Palette), "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("root-additional", "Additional", "Profile enhancements, emoji tweaks, logger and text replacement", "Additional", "App Settings", "3y3 userbg userpfp nitro emoji realmojis message logger silent typing text replacement favorites server pinned channels", getIcon(Icons.Rounded.AutoAwesome, Icons.Filled.AutoAwesome), "neutral", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("advanced-dev", "Developer Mode", "Exposes ID copying and debug tools", "Advanced", "Developer Settings", "id debug", getIcon(Icons.Rounded.Tune, Icons.Filled.Tune), "neutral", SettingsSearchDestination.Advanced))
    }
}
