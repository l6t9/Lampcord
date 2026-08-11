package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import org.koin.compose.koinInject

@Composable
fun PrivacySettings(settingsStore: SettingsStore = koinInject()) {
    val userSettings = settingsStore.userSettings
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopPrivacySettings(settingsStore, userSettings)
    } else {
        MobilePrivacySettings(settingsStore, userSettings)
    }
}

@Composable
private fun DesktopPrivacySettings(settingsStore: SettingsStore, userSettings: UserSettings?) {
    SettingsLayout {
        SettingsSection(
            title = "Safe Direct Messaging",
            icon = Icons.Filled.Security
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Content Filter", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                SettingsButtonGroup(
                    options = listOf(2, 1, 0),
                    selectedOption = userSettings?.explicit_content_filter ?: 1,
                    onOptionSelected = { filter: Int -> settingsStore.updateUserSettings(UserSettings.Partial(explicit_content_filter = filter)) },
                    iconProvider = { filter: Int, isSelected ->
                        when (filter) {
                            2 -> if (isSelected) Icons.Filled.Security else Icons.Rounded.Security
                            1 -> if (isSelected) Icons.Filled.Favorite else Icons.Rounded.Favorite
                            else -> if (isSelected) Icons.Filled.Warning else Icons.Rounded.Warning
                        }
                    },
                    labelProvider = {
                        when (it) {
                            2 -> "Keep me safe"
                            1 -> "My friends are nice"
                            else -> "I live on the edge"
                        }
                    }
                )
                Text(
                    text = when (userSettings?.explicit_content_filter) {
                        2 -> "Scan direct messages from everyone."
                        1 -> "Scan direct messages from everyone unless they are a friend."
                        else -> "Don't scan any direct messages."
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        SettingsSection(
            title = "Server Privacy Defaults",
            icon = Icons.Filled.Public
        ) {
            PrivacyToggle("Allow direct messages from server members", userSettings?.default_guilds_restricted == false, "This setting is applied when you join a new server.") { 
                settingsStore.updateUserSettings(UserSettings.Partial(default_guilds_restricted = !it))
            }
        }

        SettingsSection(
            title = "Friend Requests",
            icon = Icons.Filled.PersonAdd
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                val flags = userSettings?.friend_source_flags
                
                PrivacyToggle("Everyone", flags?.all == true) {
                    settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(all = it)))
                }
                PrivacyToggle("Friends of Friends", flags?.mutual_friends == true) {
                    settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_friends = it)))
                }
                PrivacyToggle("Server Members", flags?.mutual_guilds == true) {
                    settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_guilds = it)))
                }
            }
        }

        SettingsSection(
            title = "Enhancements",
            icon = Icons.Filled.RocketLaunch
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                PrivacyToggle("Silent Typing", Settings.shared.silentTyping, "Don't let others know when you are typing.") {
                    Settings.shared.silentTyping = it
                }
                PrivacyToggle("Hide Blocked Messages", Settings.shared.hideBlockedMessages, "Completely remove messages from blocked users.") {
                    Settings.shared.hideBlockedMessages = it
                }
            }
        }
    }
}

@Composable
private fun PrivacyToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
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
private fun MobilePrivacySettings(settingsStore: SettingsStore, userSettings: UserSettings?) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Safe Direct Messaging") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Keep me safe") },
                        description = { Text("Scan direct messages from everyone.") },
                        onClick = { settingsStore.updateUserSettings(UserSettings.Partial(explicit_content_filter = 2)) },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 2, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("My friends are nice") },
                        description = { Text("Scan direct messages from everyone unless they are a friend.") },
                        onClick = { settingsStore.updateUserSettings(UserSettings.Partial(explicit_content_filter = 1)) },
                        trailingContent = { RadioButton(selected = userSettings?.explicit_content_filter == 1, onClick = null) }
                    ),
                    Material3SettingsItem(
                        title = { Text("I live on the edge") },
                        description = { Text("Don't scan any direct messages.") },
                        onClick = { settingsStore.updateUserSettings(UserSettings.Partial(explicit_content_filter = 0)) },
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
                            settingsStore.updateUserSettings(UserSettings.Partial(default_guilds_restricted = !it))
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
                            settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(all = it)))
                        }
                    ),
                    switchSettingsItem(
                        title = "Friends of Friends",
                        checked = flags?.mutual_friends == true,
                        onCheckedChange = { 
                            settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_friends = it)))
                        }
                    ),
                    switchSettingsItem(
                        title = "Server Members",
                        checked = flags?.mutual_guilds == true,
                        onCheckedChange = { 
                            settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_guilds = it)))
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Enhancements") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Silent Typing",
                        description = "Don't let others know when you are typing.",
                        checked = Settings.shared.silentTyping,
                        onCheckedChange = { Settings.shared.silentTyping = it }
                    ),
                    switchSettingsItem(
                        title = "Hide Blocked Messages",
                        description = "Completely remove messages from blocked users.",
                        checked = Settings.shared.hideBlockedMessages,
                        onCheckedChange = { Settings.shared.hideBlockedMessages = it }
                    )
                )
            )
        }
    }
}
