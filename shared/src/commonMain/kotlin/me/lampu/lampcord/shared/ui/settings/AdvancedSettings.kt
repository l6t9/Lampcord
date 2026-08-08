package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings

@Composable
fun AdvancedSettings(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Developer Settings") {
            val devMode = chatState.userSettings?.developer_mode ?: false
            
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Developer Mode",
                        description = "Exposes ID copying and other advanced debug tools.",
                        checked = devMode,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(developer_mode = it))
                        }
                    ),
                    switchSettingsItem(
                        title = "Show Hidden Channels",
                        description = "Display channels you don't have permission to view as locked and greyed out.",
                        checked = chatState.settingsStore.showHiddenChannels,
                        onCheckedChange = { 
                            chatState.settingsStore.showHiddenChannels = it
                        }
                    )
                )
            )
        }

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
