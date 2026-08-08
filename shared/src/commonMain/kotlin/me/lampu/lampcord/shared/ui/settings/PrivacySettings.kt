package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun PrivacySettings(chatState: ChatState) {
    val userSettings = chatState.userSettings

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Safe Direct Messaging") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Keep me safe") },
                        description = { Text("Scan direct messages from everyone.") },
                        onClick = { chatState.updateUserSettings(UserSettings.Partial(explicit_content_filter = 2)) },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 2, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("My friends are nice") },
                        description = { Text("Scan direct messages from everyone unless they are a friend.") },
                        onClick = { chatState.updateUserSettings(UserSettings.Partial(explicit_content_filter = 1)) },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 1, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("I live on the edge") },
                        description = { Text("Don't scan any direct messages.") },
                        onClick = { chatState.updateUserSettings(UserSettings.Partial(explicit_content_filter = 0)) },
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
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(default_guilds_restricted = !it))
                        }
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
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(all = it)))
                        }
                    ),
                    switchSettingsItem(
                        title = "Friends of Friends",
                        checked = flags?.mutual_friends == true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_friends = it)))
                        }
                    ),
                    switchSettingsItem(
                        title = "Server Members",
                        checked = flags?.mutual_guilds == true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_guilds = it)))
                        }
                    )
                )
            )
        }
    }
}
