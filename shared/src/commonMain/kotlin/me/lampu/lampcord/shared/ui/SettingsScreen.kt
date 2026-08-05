package me.lampu.lampcord.shared.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.settings.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var selectedCategory by remember { mutableStateOf<String?>(null) }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp
        
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surface
        ) {
            if (isCompact) {
                if (selectedCategory == null) {
                    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
                        TopAppBar(
                            title = { Text("Settings") },
                            navigationIcon = {
                                IconButton(onClick = onDismiss) {
                                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                                }
                            }
                        )
                        LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            item { SettingsCategoryItem("Account", Icons.Filled.AccountCircle, false) { selectedCategory = "Account" } }
                            item { SettingsCategoryItem("Profiles", Icons.Filled.Person, false) { selectedCategory = "Profiles" } }
                            item { SettingsCategoryItem("Appearance", Icons.Filled.Palette, false) { selectedCategory = "Appearance" } }
                            item { SettingsCategoryItem("Accessibility", Icons.Filled.Accessibility, false) { selectedCategory = "Accessibility" } }
                            item { SettingsCategoryItem("Voice & Video", Icons.Filled.Mic, false) { selectedCategory = "Voice & Video" } }
                            item { Spacer(Modifier.height(16.dp)) }
                            item {
                                SettingsCategoryItem("Log Out", Icons.AutoMirrored.Filled.Logout, false, color = MaterialTheme.colorScheme.error) {
                                    // TODO: Logout
                                }
                            }
                        }
                    }
                } else {
                    Column(modifier = Modifier.fillMaxSize()) {
                        TopAppBar(
                            title = { Text(selectedCategory!!) },
                            navigationIcon = {
                                IconButton(onClick = { selectedCategory = null }) {
                                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                                }
                            }
                        )
                        Box(modifier = Modifier.fillMaxSize().padding(16.dp)) {
                            when (selectedCategory) {
                                "Account" -> AccountSettings(chatState)
                                "Profiles" -> ProfileSettings(chatState)
                                "Appearance" -> AppearanceSettings(chatState)
                                "Accessibility" -> AccessibilitySettings(chatState)
                                else -> Text("Coming soon")
                            }
                        }
                    }
                }
            } else {
                // Desktop Layout
                val activeCategory = selectedCategory ?: "Account"
                Row(modifier = Modifier.fillMaxSize()) {
                    // Sidebar Category List
                    Surface(
                        modifier = Modifier.width(240.dp).fillMaxHeight(),
                        color = MaterialTheme.colorScheme.surfaceContainerLow
                    ) {
                        Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 32.dp)) {
                            androidx.compose.material3.Text(
                                "Settings",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = androidx.compose.ui.text.font.FontWeight.Bold,
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
                            )
                            
                            Spacer(modifier = Modifier.height(16.dp))
                            
                            SettingsCategoryItem("Account", Icons.Filled.AccountCircle, activeCategory == "Account") { selectedCategory = "Account" }
                            SettingsCategoryItem("Profiles", Icons.Filled.Person, activeCategory == "Profiles") { selectedCategory = "Profiles" }
                            SettingsCategoryItem("Appearance", Icons.Filled.Palette, activeCategory == "Appearance") { selectedCategory = "Appearance" }
                            SettingsCategoryItem("Accessibility", Icons.Filled.Accessibility, activeCategory == "Accessibility") { selectedCategory = "Accessibility" }
                            SettingsCategoryItem("Voice & Video", Icons.Filled.Mic, activeCategory == "Voice & Video") { selectedCategory = "Voice & Video" }
                            
                            Spacer(modifier = Modifier.weight(1f))
                            
                            SettingsCategoryItem("Log Out", Icons.AutoMirrored.Filled.Logout, false, color = MaterialTheme.colorScheme.error) {
                                // TODO: Logout logic
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
                            .padding(32.dp)
                            .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f))
                        ) {
                            when (activeCategory) {
                                "Account" -> AccountSettings(chatState)
                                "Profiles" -> ProfileSettings(chatState)
                                "Appearance" -> AppearanceSettings(chatState)
                                "Accessibility" -> AccessibilitySettings(chatState)
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
