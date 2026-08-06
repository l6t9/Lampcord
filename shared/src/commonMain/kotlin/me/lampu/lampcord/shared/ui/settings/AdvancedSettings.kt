package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*

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
                            chatState.userSettings = chatState.userSettings?.copy(developer_mode = it)
                        }
                    )
                ),
                horizontalPadding = 0.dp
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
                ),
                horizontalPadding = 0.dp
            )
        }
    }
}
