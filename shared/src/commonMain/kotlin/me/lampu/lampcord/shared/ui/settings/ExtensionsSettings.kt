package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.settings.*
import org.koin.compose.koinInject

@Composable
fun ExtensionsSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Extensions",
        onNavigateBack = onBack
    ) {
        ExtensionsSettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun ExtensionsSettingsContent(settingsStore: SettingsStore = koinInject()) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Enhancements",
            items = listOf(
                switchSettingsItem(
                    title = "Free Nitro Emojis",
                    description = "Use emojis from any server for free.",
                    checked = Settings.shared.freeNitroEmojis,
                    onCheckedChange = { Settings.shared.freeNitroEmojis = it }
                ),
                switchSettingsItem(
                    title = "Realmojis",
                    description = "Makes free nitro emojis look like real ones.",
                    checked = Settings.shared.realmojis,
                    onCheckedChange = { Settings.shared.realmojis = it }
                ),
                switchSettingsItem(
                    title = "Bypass Upload Limit",
                    description = "Attempt to send files larger than 25MB.",
                    checked = Settings.shared.bypassUploadLimit,
                    onCheckedChange = { Settings.shared.bypassUploadLimit = it }
                ),
                switchSettingsItem(
                    title = "Silent Typing",
                    description = "Don't let others know when you're typing.",
                    checked = Settings.shared.silentTyping,
                    onCheckedChange = { Settings.shared.silentTyping = it }
                )
            )
        )

        Material3SettingsGroup(
            title = "Message Logger",
            items = buildList {
                add(switchSettingsItem(
                    title = "Enabled",
                    description = "Keep a local history of deleted and edited messages.",
                    checked = Settings.shared.messageLoggerEnabled,
                    onCheckedChange = { Settings.shared.messageLoggerEnabled = it }
                ))
                if (Settings.shared.messageLoggerEnabled) {
                    add(switchSettingsItem(
                        title = "Ignore Bots",
                        checked = Settings.shared.messageLoggerIgnoreBots,
                        onCheckedChange = { Settings.shared.messageLoggerIgnoreBots = it }
                    ))
                    add(switchSettingsItem(
                        title = "Ignore Self",
                        checked = Settings.shared.messageLoggerIgnoreSelf,
                        onCheckedChange = { Settings.shared.messageLoggerIgnoreSelf = it }
                    ))
                }
            }
        )

        Material3SettingsGroup(
            title = "Clean Channels",
            items = listOf(
                switchSettingsItem(
                    title = "Remove Emojis",
                    description = "Remove emojis from channel names.",
                    checked = Settings.shared.cleanChannelsRemoveEmojis,
                    onCheckedChange = { Settings.shared.cleanChannelsRemoveEmojis = it }
                ),
                switchSettingsItem(
                    title = "Hide Symbols",
                    description = "Hide symbols like # from channel names.",
                    checked = Settings.shared.cleanChannelsHideSymbols,
                    onCheckedChange = { Settings.shared.cleanChannelsHideSymbols = it }
                )
            )
        )
    }
}
