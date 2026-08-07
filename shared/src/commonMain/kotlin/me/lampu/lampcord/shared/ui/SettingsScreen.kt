package me.lampu.lampcord.shared.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.*
import me.lampu.lampcord.shared.ui.components.settings.*

private enum class SettingsSection(val title: String, val icon: ImageVector) {
    ACCOUNT("Account", Icons.Filled.AccountCircle),
    PROFILES("Profiles", Icons.Filled.Person),
    CONNECTIONS("Connections", Icons.Filled.Link),
    DEVICES("Devices", Icons.Filled.Tv),
    APPEARANCE("Appearance", Icons.Filled.Palette),
    ACCESSIBILITY("Accessibility", Icons.Filled.Accessibility),
    VOICE_VIDEO("Voice & Video", Icons.Filled.Mic),
    NOTIFICATIONS("Notifications", Icons.Filled.Notifications),
    ADVANCED("Advanced", Icons.Filled.Tune),
    ABOUT("About", Icons.Filled.Info),
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SettingsScreen(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var selectedCategory by remember { mutableStateOf<SettingsSection?>(null) }
    var searchQuery by rememberSaveable { mutableStateOf("") }
    var showLogoutConfirmation by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current

    if (showLogoutConfirmation) {
        AlertDialog(
            onDismissRequest = { showLogoutConfirmation = false },
            title = { Text("Log Out", color = MaterialTheme.colorScheme.error) },
            text = { Text("Are you sure you want to log out of Materialcord?") },
            confirmButton = {
                TextButton(
                    onClick = {
                        showLogoutConfirmation = false
                        chatState.disconnect()
                        onDismiss()
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error)
                ) {
                    Text("Log Out")
                }
            },
            dismissButton = {
                TextButton(onClick = { showLogoutConfirmation = false }) {
                    Text("Cancel")
                }
            }
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
                        contentScrollable = category != SettingsSection.ABOUT
                    ) {
                        when (category) {
                            SettingsSection.ACCOUNT -> AccountSettings(chatState)
                            SettingsSection.PROFILES -> ProfileSettings(chatState)
                            SettingsSection.CONNECTIONS -> ConnectionsSettings(chatState)
                            SettingsSection.DEVICES -> DevicesSettings(chatState)
                            SettingsSection.APPEARANCE -> AppearanceSettings(chatState)
                            SettingsSection.ACCESSIBILITY -> AccessibilitySettings(chatState)
                            SettingsSection.VOICE_VIDEO -> { /* TODO */ }
                            SettingsSection.NOTIFICATIONS -> NotificationsSettings(chatState)
                            SettingsSection.ADVANCED -> AdvancedSettings(chatState)
                            SettingsSection.ABOUT -> AboutContent(version = "1.0.0", onOpenUrl = { uriHandler.openUri(it) })
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
                        .widthIn(max = 1080.dp)
                        .fillMaxWidth()
                        .heightIn(max = 720.dp)
                        .fillMaxHeight(),
                    shape = RoundedCornerShape(24.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                    elevation = CardDefaults.cardElevation(defaultElevation = 12.dp)
                ) {
                    val activeCategory = selectedCategory ?: SettingsSection.ACCOUNT
                    Row(modifier = Modifier.fillMaxSize()) {
                        // Sidebar Category List
                        Surface(
                            modifier = Modifier.width(280.dp).fillMaxHeight(),
                            color = MaterialTheme.colorScheme.surfaceContainerLowest
                        ) {
                            Column(modifier = Modifier.padding(16.dp)) {
                                Text(
                                    "Settings",
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp)
                                )
                                
                                Spacer(modifier = Modifier.height(12.dp))
                                
                                SettingsSearchField(
                                    query = searchQuery,
                                    onQueryChange = { searchQuery = it },
                                    onClear = { searchQuery = "" },
                                    modifier = Modifier.fillMaxWidth()
                                )
                                
                                Spacer(modifier = Modifier.height(16.dp))
                                
                                LazyColumn(
                                    modifier = Modifier.fillMaxSize(),
                                    verticalArrangement = Arrangement.spacedBy(4.dp)
                                ) {
                                    items(SettingsSection.entries, key = { it.ordinal }) { section ->
                                        val isSelected = activeCategory == section
                                        
                                        val animatedIconColor by animateColorAsState(
                                            targetValue = if (isSelected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainerHigh,
                                            animationSpec = tween(300, easing = FastOutSlowInEasing),
                                            label = "iconBgColor",
                                        )
                                        val animatedIconTint by animateColorAsState(
                                            targetValue = if (isSelected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurfaceVariant,
                                            animationSpec = tween(300, easing = FastOutSlowInEasing),
                                            label = "iconTint",
                                        )

                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .clip(RoundedCornerShape(12.dp))
                                                .selectable(
                                                    selected = isSelected,
                                                    role = Role.Tab,
                                                    onClick = {
                                                        selectedCategory = section
                                                        searchQuery = ""
                                                    },
                                                ).padding(horizontal = 8.dp, vertical = 8.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                                        ) {
                                            Surface(
                                                modifier = Modifier.size(40.dp),
                                                shape = RoundedCornerShape(10.dp),
                                                color = animatedIconColor,
                                            ) {
                                                Icon(
                                                    section.icon,
                                                    contentDescription = section.title,
                                                    tint = animatedIconTint,
                                                    modifier = Modifier.fillMaxSize().padding(8.dp),
                                                )
                                            }
                                            Text(
                                                section.title,
                                                style = MaterialTheme.typography.bodyMedium,
                                                fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                                                color = MaterialTheme.colorScheme.onSurface,
                                            )
                                        }
                                    }
                                    
                                    item { Spacer(modifier = Modifier.height(16.dp)) }
                                    
                                    item {
                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .clip(RoundedCornerShape(12.dp))
                                                .clickable { showLogoutConfirmation = true }
                                                .padding(horizontal = 8.dp, vertical = 8.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                                        ) {
                                            Surface(
                                                modifier = Modifier.size(40.dp),
                                                shape = RoundedCornerShape(10.dp),
                                                color = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.2f),
                                            ) {
                                                Icon(
                                                    Icons.AutoMirrored.Filled.Logout,
                                                    contentDescription = "Log Out",
                                                    tint = MaterialTheme.colorScheme.error,
                                                    modifier = Modifier.fillMaxSize().padding(8.dp),
                                                )
                                            }
                                            Text(
                                                "Log Out",
                                                style = MaterialTheme.typography.bodyMedium,
                                                fontWeight = FontWeight.SemiBold,
                                                color = MaterialTheme.colorScheme.error,
                                            )
                                        }
                                    }
                                }
                            }
                        }

                        // Divider
                        Box(modifier = Modifier.fillMaxHeight().width(1.dp).background(MaterialTheme.colorScheme.outlineVariant))

                        // Main Content Area
                        Box(modifier = Modifier.weight(1f).fillMaxHeight()) {
                            AnimatedContent(
                                targetState = activeCategory,
                                transitionSpec = {
                                    val isForward = targetState.ordinal > initialState.ordinal
                                    (
                                        slideInVertically(
                                            animationSpec = tween(300, easing = FastOutSlowInEasing),
                                            initialOffsetY = { if (isForward) it else -it },
                                        ) + fadeIn(animationSpec = tween(300))
                                    ) togetherWith (
                                        slideOutVertically(
                                            animationSpec = tween(300, easing = FastOutSlowInEasing),
                                            targetOffsetY = { if (isForward) -it else it },
                                        ) + fadeOut(animationSpec = tween(200))
                                    )
                                },
                                modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
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
                                        SettingsSection.ACCOUNT -> AccountSettings(chatState)
                                        SettingsSection.PROFILES -> ProfileSettings(chatState)
                                        SettingsSection.CONNECTIONS -> ConnectionsSettings(chatState)
                                        SettingsSection.DEVICES -> DevicesSettings(chatState)
                                        SettingsSection.APPEARANCE -> AppearanceSettings(chatState)
                                        SettingsSection.ACCESSIBILITY -> AccessibilitySettings(chatState)
                                        SettingsSection.NOTIFICATIONS -> NotificationsSettings(chatState)
                                        SettingsSection.ADVANCED -> AdvancedSettings(chatState)
                                        SettingsSection.ABOUT -> AboutContent(version = "1.0.0", onOpenUrl = { uriHandler.openUri(it) })
                                        else -> {
                                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                                Text("Feature coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                    }
                                }
                            }
                            
                            IconButton(
                                onClick = onDismiss,
                                modifier = Modifier.align(Alignment.TopEnd).padding(8.dp).size(36.dp),
                            ) {
                                Icon(
                                    Icons.Default.Close,
                                    contentDescription = "Close",
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                    modifier = Modifier.size(20.dp),
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
        add(SettingsSearchEntry("root-about", "About", "App information and credits", "About", "App Settings", "version info credits developer", Icons.Filled.Info, "neutral", SettingsSearchDestination.Advanced))
        add(SettingsSearchEntry("root-logout", "Log Out", "Sign out of your account", "Logout", "Account", "sign out exit", Icons.AutoMirrored.Filled.Logout, "neutral", SettingsSearchDestination.Logout))
        
        // Deep search entries
        add(SettingsSearchEntry("appearance-theme", "Theme Mode", "Auto, Light, Dark, or AMOLED", "Appearance", "Theme", "dark light amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-pure-black", "Pure Black", "Use pure black backgrounds in dark mode", "Appearance", "Theme", "amoled", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("appearance-compact", "Compact Messages", "Denser layout for chat", "Appearance", "Display", "compact message denser", Icons.Filled.Palette, "orange", SettingsSearchDestination.Appearance))
        add(SettingsSearchEntry("advanced-dev", "Developer Mode", "Exposes ID copying and debug tools", "Advanced", "Developer Settings", "id debug", Icons.Filled.Tune, "neutral", SettingsSearchDestination.Advanced))
    }
}
