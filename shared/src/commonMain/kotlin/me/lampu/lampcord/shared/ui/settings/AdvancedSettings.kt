package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
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
fun AdvancedSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Advanced",
        onNavigateBack = onBack
    ) {
        AdvancedSettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun AdvancedSettingsContent(settingsStore: SettingsStore = koinInject()) {
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
                ),
                switchSettingsItem(
                    title = "Disable Wayland Scaling Fix",
                    description = "Prevents the app from trying to automatically scale on Wayland. Requires restart.",
                    checked = settingsStore.disableWaylandScaling,
                    onCheckedChange = {
                        settingsStore.disableWaylandScaling = it
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Data Management",
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
