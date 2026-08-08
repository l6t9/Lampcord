package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import kotlinx.coroutines.launch

@Composable
fun PrivacySettings(chatState: ChatState) {
    val userSettings = chatState.userSettings
    val scope = rememberCoroutineScope()

    fun updateSettings(partial: UserSettings.Partial) {
        scope.launch {
            // In a real app, this would call chatState.discordClient.updateUserSettings
            // and then update the local state if successful.
            // For now, we update local state directly for immediate feedback.
        }
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Safe Direct Messaging") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Keep me safe") },
                        description = { Text("Scan direct messages from everyone.") },
                        onClick = { /* TODO: set explicitContentFilter = 2 */ },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 2, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("My friends are nice") },
                        description = { Text("Scan direct messages from everyone unless they are a friend.") },
                        onClick = { /* TODO: set explicitContentFilter = 1 */ },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 1, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("I live on the edge") },
                        description = { Text("Don't scan any direct messages.") },
                        onClick = { /* TODO: set explicitContentFilter = 0 */ },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 0, onClick = null) }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Server Privacy Defaults") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Allow direct messages from server members",
                        description = "This setting is applied when you join a new server. It does not affect existing servers.",
                        checked = userSettings?.default_guilds_restricted == false,
                        onCheckedChange = { /* TODO */ }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Who can add you as a friend") {
            val flags = userSettings?.friend_source_flags
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Everyone",
                        checked = flags?.all == true,
                        onCheckedChange = { /* TODO */ }
                    ),
                    switchSettingsItem(
                        title = "Friends of Friends",
                        checked = flags?.mutual_friends == true,
                        onCheckedChange = { /* TODO */ }
                    ),
                    switchSettingsItem(
                        title = "Server Members",
                        checked = flags?.mutual_guilds == true,
                        onCheckedChange = { /* TODO */ }
                    )
                )
            )
        }
    }
}
