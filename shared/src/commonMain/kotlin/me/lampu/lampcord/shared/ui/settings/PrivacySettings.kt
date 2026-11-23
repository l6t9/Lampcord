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
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.friend_permitted_source_all
import me.lampu.lampcord.shared.generated.resources.friend_permitted_source_mutual_friends
import me.lampu.lampcord.shared.generated.resources.friend_permitted_source_mutual_guilds
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_content_controls
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_content_filter
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_content_filter_edge
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_content_filter_off_help
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_dm_server_members_desc
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_friend_requests
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_hide_blocked_messages
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_hide_blocked_messages_desc
import me.lampu.lampcord.shared.generated.resources.new_guilds_dm_allowed
import me.lampu.lampcord.shared.generated.resources.privacy_and_safety
import me.lampu.lampcord.shared.generated.resources.user_dm_settings
import me.lampu.lampcord.shared.generated.resources.user_explicit_content_filter
import me.lampu.lampcord.shared.generated.resources.user_explicit_content_filter_friends_and_non_friends
import me.lampu.lampcord.shared.generated.resources.user_explicit_content_filter_friends_and_non_friends_help
import me.lampu.lampcord.shared.generated.resources.user_explicit_content_filter_non_friends
import me.lampu.lampcord.shared.generated.resources.user_explicit_content_filter_non_friends_help
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

@Composable
fun PrivacySettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = stringResource(Res.string.privacy_and_safety),
        onNavigateBack = onBack
    ) {
        PrivacySettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun PrivacySettingsContent(settingsStore: SettingsStore = koinInject()) {
    val userSettings = settingsStore.userSettings
    val labelKeepMeSafe = stringResource(Res.string.user_explicit_content_filter_friends_and_non_friends)
    val labelMyFriendsAreNice = stringResource(Res.string.user_explicit_content_filter_non_friends)
    val labelLiveOnTheEdge = stringResource(Res.string.lampcord_settings_content_filter_edge)
    val labelEveryone = stringResource(Res.string.friend_permitted_source_all)
    val labelMutualFriends = stringResource(Res.string.friend_permitted_source_mutual_friends)
    val labelMutualGuilds = stringResource(Res.string.friend_permitted_source_mutual_guilds)
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = stringResource(Res.string.user_explicit_content_filter),
            items = listOf(
                selectSettingsItem(
                    title = stringResource(Res.string.lampcord_settings_content_filter),
                    options = listOf(2, 1, 0),
                    selectedOption = userSettings?.explicit_content_filter ?: 1,
                    onOptionSelected = { filter: Int ->
                        settingsStore.updateUserSettings(UserSettings.Partial(explicit_content_filter = filter))
                    },
                    optionLabel = { filter ->
                        when (filter) {
                            2 -> labelKeepMeSafe
                            1 -> labelMyFriendsAreNice
                            else -> labelLiveOnTheEdge
                        }
                    },
                    optionIcon = { filter, selected ->
                        val icon = when (filter) {
                            2 -> if (selected) Icons.Filled.Security else Icons.Rounded.Security
                            1 -> if (selected) Icons.Filled.Favorite else Icons.Rounded.Favorite
                            else -> if (selected) Icons.Filled.Warning else Icons.Rounded.Warning
                        }
                        Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp))
                    },
                    description = stringResource(
                        when (userSettings?.explicit_content_filter) {
                            2 -> Res.string.user_explicit_content_filter_friends_and_non_friends_help
                            1 -> Res.string.user_explicit_content_filter_non_friends_help
                            else -> Res.string.lampcord_settings_content_filter_off_help
                        }
                    ),
                )
            )
        )

        Material3SettingsGroup(
            title = stringResource(Res.string.user_dm_settings),
            items = listOf(
                switchSettingsItem(
                    title = stringResource(Res.string.new_guilds_dm_allowed),
                    description = stringResource(Res.string.lampcord_settings_dm_server_members_desc),
                    checked = userSettings?.default_guilds_restricted == false,
                    onCheckedChange = { 
                        settingsStore.updateUserSettings(UserSettings.Partial(default_guilds_restricted = !it))
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = stringResource(Res.string.lampcord_settings_friend_requests),
            items = buildList {
                val flags = userSettings?.friend_source_flags
                
                add(switchSettingsItem(
                    title = labelEveryone,
                    checked = flags?.all == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(all = it)))
                    }
                ))
                add(switchSettingsItem(
                    title = labelMutualFriends,
                    checked = flags?.mutual_friends == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_friends = it)))
                    }
                ))
                add(switchSettingsItem(
                    title = labelMutualGuilds,
                    checked = flags?.mutual_guilds == true,
                    onCheckedChange = {
                        settingsStore.updateUserSettings(UserSettings.Partial(friend_source_flags = me.lampu.lampcord.shared.model.FriendSourceFlags(mutual_guilds = it)))
                    }
                ))
            }
        )

        Material3SettingsGroup(
            title = stringResource(Res.string.lampcord_settings_content_controls),
            items = listOf(
                switchSettingsItem(
                    title = stringResource(Res.string.lampcord_settings_hide_blocked_messages),
                    description = stringResource(Res.string.lampcord_settings_hide_blocked_messages_desc),
                    checked = Settings.shared.hideBlockedMessages,
                    onCheckedChange = { Settings.shared.hideBlockedMessages = it }
                )
            )
        )

    }
}
