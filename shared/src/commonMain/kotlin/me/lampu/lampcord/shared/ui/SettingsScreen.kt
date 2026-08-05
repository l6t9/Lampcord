package me.lampu.lampcord.shared.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.*

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SettingsScreen(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var selectedCategory by remember { mutableStateOf<String?>(null) }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp
        
        if (isCompact) {
            val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()
            Scaffold(
                modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
                topBar = {
                    LargeTopAppBar(
                        title = { Text(selectedCategory ?: "Settings") },
                        navigationIcon = {
                            IconButton(onClick = {
                                if (selectedCategory == null) onDismiss() else selectedCategory = null
                            }) {
                                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                            }
                        },
                        scrollBehavior = scrollBehavior
                    )
                }
            ) { padding ->
                Box(modifier = Modifier.fillMaxSize().padding(padding)) {
                    if (selectedCategory == null) {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(16.dp),
                            verticalArrangement = Arrangement.spacedBy(2.dp)
                        ) {
                            item { SettingsGroupTitle("App Settings") }
                            item { SettingsCategoryItem("Account", Icons.Filled.AccountCircle, false, isFirst = true) { selectedCategory = "Account" } }
                            item { SettingsCategoryItem("Profiles", Icons.Filled.Person, false) { selectedCategory = "Profiles" } }
                            item { SettingsCategoryItem("Connections", Icons.Filled.Link, false) { selectedCategory = "Connections" } }
                            item { SettingsCategoryItem("Devices", Icons.Filled.Tv, false, isLast = true) { selectedCategory = "Devices" } }
                            
                            item { Spacer(Modifier.height(16.dp)) }
                            item { SettingsGroupTitle("App Settings") }
                            item { SettingsCategoryItem("Appearance", Icons.Filled.Palette, false, isFirst = true) { selectedCategory = "Appearance" } }
                            item { SettingsCategoryItem("Accessibility", Icons.Filled.Accessibility, false) { selectedCategory = "Accessibility" } }
                            item { SettingsCategoryItem("Voice & Video", Icons.Filled.Mic, false) { selectedCategory = "Voice & Video" } }
                            item { SettingsCategoryItem("Notifications", Icons.Filled.Notifications, false) { selectedCategory = "Notifications" } }
                            item { SettingsCategoryItem("Advanced", Icons.Filled.Tune, false, isLast = true) { selectedCategory = "Advanced" } }
                            
                            item { Spacer(Modifier.height(24.dp)) }
                            item {
                                SettingsCategoryItem(
                                    "Log Out", 
                                    Icons.AutoMirrored.Filled.Logout, 
                                    false, 
                                    color = MaterialTheme.colorScheme.error,
                                    isFirst = true,
                                    isLast = true
                                ) {
                                    chatState.disconnect()
                                    onDismiss()
                                }
                            }
                        }
                    } else {
                        Box(modifier = Modifier.fillMaxSize().padding(16.dp)) {
                            when (selectedCategory) {
                                "Account" -> AccountSettings(chatState)
                                "Profiles" -> ProfileSettings(chatState)
                                "Connections" -> ConnectionsSettings(chatState)
                                "Devices" -> DevicesSettings(chatState)
                                "Appearance" -> AppearanceSettings(chatState)
                                "Accessibility" -> AccessibilitySettings(chatState)
                                "Notifications" -> NotificationsSettings(chatState)
                                "Advanced" -> AdvancedSettings(chatState)
                                else -> Text("Coming soon")
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
                                    
                                    item { Spacer(Modifier.height(16.dp)) }
                                    
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
fun SettingsGroupTitle(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(start = 16.dp, bottom = 8.dp, top = 8.dp)
    )
}
