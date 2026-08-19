package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun PrivacySettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Privacy & Safety",
        onNavigateBack = onBack
    ) {
        PrivacySettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun PrivacySettingsContent(settingsStore: SettingsStore = koinInject()) {
    val userSettings = settingsStore.userSettings
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Safe Direct Messaging",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Content Filter") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
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
                )
            )
        )

        Material3SettingsGroup(
            title = "Server Privacy Defaults",
            items = listOf(
                switchSettingsItem(
                    title = "Allow direct messages from server members",
                    description = "This setting is applied when you join a new server.",
                    checked = userSettings?.default_guilds_restricted == false,
                    onCheckedChange = { 
                        settingsStore.updateUserSettings(UserSettings.Partial(default_guilds_restricted = !it))
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Friend Requests",
            items = buildList {
                val flags = userSettings?.friend_source_flags
                
                add(switchSettingsItem(
                    title = "Everyone",
                    checked = flags?.all == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(all = it)))
                    }
                ))
                add(switchSettingsItem(
                    title = "Friends of Friends",
                    checked = flags?.mutual_friends == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_friends = it)))
                    }
                ))
                add(switchSettingsItem(
                    title = "Server Members",
                    checked = flags?.mutual_guilds == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_guilds = it)))
                    }
                ))
            }
        )

        Material3SettingsGroup(
            title = "Enhancements",
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
