package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import org.koin.compose.koinInject

@Composable
fun AdvancedSettings(settingsStore: SettingsStore = koinInject()) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopAdvancedSettings(settingsStore)
    } else {
        MobileAdvancedSettings(settingsStore)
    }
}

@Composable
private fun DesktopAdvancedSettings(settingsStore: SettingsStore) {
    SettingsLayout {
        SettingsSection(
            title = "Developer Settings",
            icon = Icons.Filled.Tune
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                val devMode = settingsStore.userSettings?.developer_mode ?: false
                AdvancedToggle("Developer Mode", devMode, "Exposes ID copying and other advanced debug tools.") {
                    settingsStore.updateUserSettings(UserSettings.Partial(developer_mode = it))
                }
                AdvancedToggle("Show Hidden Channels", settingsStore.showHiddenChannels, "Display channels you don't have permission to view.") {
                    settingsStore.showHiddenChannels = it
                }
            }
        }

        SettingsSection(
            title = "Data Management",
            icon = Icons.Filled.Delete
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(onClick = { /* TODO */ }) {
                    Text("Clear Cache")
                }
                Text("Delete all locally cached images and messages.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                
                Spacer(Modifier.height(8.dp))
                
                Button(
                    onClick = { /* TODO */ },
                    colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error, contentColor = MaterialTheme.colorScheme.onError)
                ) {
                    Text("Reset App State")
                }
                Text("Restore all settings to their default values. This will log you out.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
private fun AdvancedToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Column(modifier = Modifier.weight(1f)) {
            Text(label, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
            if (description != null) {
                Text(description, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun MobileAdvancedSettings(settingsStore: SettingsStore) {
    Column(modifier = Modifier.fillMaxWidth()) {
        val devMode = settingsStore.userSettings?.developer_mode ?: false

        Material3SettingsGroup(
            title = "Developer Settings",
            items = listOf(
                switchSettingsItem(
                    title = "Developer Mode",
                    description = "Exposes ID copying and other advanced debug tools.",
                    checked = devMode,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(developer_mode = it))
                    }
                ),
                switchSettingsItem(
                    title = "Show Hidden Channels",
                    description = "Display channels you don't have permission to view as locked and greyed out.",
                    checked = settingsStore.showHiddenChannels,
                    onCheckedChange = {
                        settingsStore.showHiddenChannels = it
                    }
                )
            )
        )

        Material3SettingsGroup(title = "Data Management") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Clear Cache") },
                        description = { Text("Delete all locally cached images and messages.") },
                        onClick = { /* TODO */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Reset App State") },
                        description = { Text("Restore all settings to their default values.") },
                        onClick = { /* TODO */ }
                    )
                )
            )
        }
    }
}
