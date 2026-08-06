package me.lampu.lampcord.shared.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.spring
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.*
import me.lampu.lampcord.shared.ui.components.settings.*

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SettingsScreen(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var selectedCategory by remember { mutableStateOf<String?>(null) }
    var searchQuery by rememberSaveable { mutableStateOf("") }
    
    val searchEntries = rememberSettingsSearchEntries()
    val searchResults = remember(searchEntries, searchQuery) {
        searchEntries.filter { it.matches(searchQuery) }
    }

    fun openSearchEntry(entry: SettingsSearchEntry) {
        selectedCategory = entry.screen
        searchQuery = ""
    }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp
        
        if (isCompact) {
            val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
            val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

            AnimatedContent(
                targetState = selectedCategory,
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
                                                onClick = { selectedCategory = "Account" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Person,
                                                title = { Text("Profiles") },
                                                description = { Text("Customize your appearance across servers") },
                                                onClick = { selectedCategory = "Profiles" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Link,
                                                title = { Text("Connections") },
                                                description = { Text("Connect your accounts from other platforms") },
                                                onClick = { selectedCategory = "Connections" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Tv,
                                                title = { Text("Devices") },
                                                description = { Text("Manage your active sessions") },
                                                onClick = { selectedCategory = "Devices" }
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
                                                onClick = { selectedCategory = "Appearance" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Accessibility,
                                                title = { Text("Accessibility") },
                                                description = { Text("Visual and interactive adjustments") },
                                                onClick = { selectedCategory = "Accessibility" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Mic,
                                                title = { Text("Voice & Video") },
                                                description = { Text("Input, output, and camera settings") },
                                                onClick = { selectedCategory = "Voice & Video" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Notifications,
                                                title = { Text("Notifications") },
                                                description = { Text("Control how you're notified") },
                                                onClick = { selectedCategory = "Notifications" }
                                            ),
                                            Material3SettingsItem(
                                                Icons.Filled.Tune,
                                                title = { Text("Advanced") },
                                                description = { Text("Developer settings and experimental features") },
                                                onClick = { selectedCategory = "Advanced" }
                                            )
                                        )
                                    )
                                }

                                item {
                                    Material3SettingsGroup(
                                        items = listOf(
                                            Material3SettingsItem(
                                                Icons.AutoMirrored.Filled.Logout,
                                                title = { Text("Log Out") },
                                                onClick = {
                                                    chatState.disconnect()
                                                    onDismiss()
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
                        title = category,
                        onNavigateBack = { selectedCategory = null }
                    ) {
                        when (category) {
                            "Account" -> AccountSettings(chatState)
                            "Profiles" -> ProfileSettings(chatState)
                            "Connections" -> ConnectionsSettings(chatState)
                            "Devices" -> DevicesSettings(chatState)
                            "Appearance" -> AppearanceSettings(chatState)
                            "Accessibility" -> AccessibilitySettings(chatState)
                            "Notifications" -> NotificationsSettings(chatState)
                            "Advanced" -> AdvancedSettings(chatState)
                            else -> {
                                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                                    Text("Feature coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Desktop Layout - Metrolist-style Overlay
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                Card(
                    modifier = Modifier
                        .padding(32.dp)
                        .widthIn(max = 1024.dp)
                        .fillMaxWidth()
                        .heightIn(max = 768.dp)
                        .fillMaxHeight(),
                    shape = RoundedCornerShape(24.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                    elevation = CardDefaults.cardElevation(defaultElevation = 12.dp)
                ) {
                    val activeCategory = selectedCategory ?: "Account"
                    Row(modifier = Modifier.fillMaxSize()) {
                        // Sidebar Category List
                        Surface(
                            modifier = Modifier.width(260.dp).fillMaxHeight(),
                            color = MaterialTheme.colorScheme.surfaceContainerLow
                        ) {
                            Column(modifier = Modifier.padding(horizontal = 12.dp, vertical = 24.dp)) {
                                Text(
                                    "Settings",
                                    style = MaterialTheme.typography.titleLarge,
                                    fontWeight = androidx.compose.ui.text.font.FontWeight.Bold,
                                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
                                )
                                
                                Spacer(modifier = Modifier.height(12.dp))
                                
                                LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                    item { SettingsCategoryItem("Account", Icons.Filled.AccountCircle, activeCategory == "Account", isFirst = true, isLast = false) { selectedCategory = "Account" } }
                                    item { SettingsCategoryItem("Profiles", Icons.Filled.Person, activeCategory == "Profiles") { selectedCategory = "Profiles" } }
                                    item { SettingsCategoryItem("Connections", Icons.Filled.Link, activeCategory == "Connections") { selectedCategory = "Connections" } }
                                    item { SettingsCategoryItem("Devices", Icons.Filled.Tv, activeCategory == "Devices", isFirst = false, isLast = true) { selectedCategory = "Devices" } }
                                    
                                    item { Spacer(modifier = Modifier.height(16.dp)) }
                                    
                                    item { SettingsCategoryItem("Appearance", Icons.Filled.Palette, activeCategory == "Appearance", isFirst = true, isLast = false) { selectedCategory = "Appearance" } }
                                    item { SettingsCategoryItem("Accessibility", Icons.Filled.Accessibility, activeCategory == "Accessibility") { selectedCategory = "Accessibility" } }
                                    item { SettingsCategoryItem("Voice & Video", Icons.Filled.Mic, activeCategory == "Voice & Video") { selectedCategory = "Voice & Video" } }
                                    item { SettingsCategoryItem("Notifications", Icons.Filled.Notifications, activeCategory == "Notifications") { selectedCategory = "Notifications" } }
                                    item { SettingsCategoryItem("Advanced", Icons.Filled.Tune, activeCategory == "Advanced", isFirst = false, isLast = true) { selectedCategory = "Advanced" } }
                                    
                                    item { Spacer(modifier = Modifier.weight(1f)) }
                                    
                                    item {
                                        SettingsCategoryItem("Log Out", Icons.AutoMirrored.Filled.Logout, false, color = MaterialTheme.colorScheme.error, isFirst = true, isLast = true) {
                                            chatState.disconnect()
                                            onDismiss()
                                        }
                                    }
                                }
                            }
                        }

                        // Main Content Area
                        Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                            TopAppBar(
                                title = { Text(activeCategory) },
                                actions = {
                                    IconButton(onClick = onDismiss) {
                                        Icon(Icons.Filled.Close, contentDescription = "Close")
                                    }
                                },
                                colors = TopAppBarDefaults.topAppBarColors(
                                    containerColor = MaterialTheme.colorScheme.surface
                                )
                            )
                            
                            Box(modifier = Modifier
                                .fillMaxSize()
                                .padding(24.dp)
                                .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f))
                            ) {
                                when (activeCategory) {
                                    "Account" -> AccountSettings(chatState)
                                    "Profiles" -> ProfileSettings(chatState)
                                    "Connections" -> ConnectionsSettings(chatState)
                                    "Devices" -> DevicesSettings(chatState)
                                    "Appearance" -> AppearanceSettings(chatState)
                                    "Accessibility" -> AccessibilitySettings(chatState)
                                    "Notifications" -> NotificationsSettings(chatState)
                                    "Advanced" -> AdvancedSettings(chatState)
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
        add(SettingsSearchEntry("root-notifications", "Notifications", "Control how you're notified", "Notifications", "App Settings", "push mentions sounds", Icons.Filled.Notifications, "rose", SettingsSearchDestination.Notifications))
        add(SettingsSearchEntry("root-advanced", "Advanced", "Developer settings and experimental features", "Advanced", "App Settings", "logs inspector debug", Icons.Filled.Tune, "neutral", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-logout", "Log Out", "Sign out of your account", "Logout", "Account", "sign out exit", Icons.AutoMirrored.Filled.Logout, "neutral", SettingsSearchDestination.Logout))
        
        // Deep search entries
        add(SettingsSearchEntry("appearance-theme", "Theme Mode", "Auto, Light, Dark, or AMOLED", "Appearance", "Theme", "dark light amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-pure-black", "Pure Black", "Use pure black backgrounds in dark mode", "Appearance", "Theme", "amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-compact", "Compact Messages", "Denser layout for chat", "Appearance", "Display", "compact message denser", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("advanced-dev", "Developer Mode", "Exposes ID copying and debug tools", "Advanced", "Developer Settings", "id debug", Icons.Filled.Tune, "neutral", SettingsSearchDestination.Advanced))
    }
}
