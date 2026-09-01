package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import org.koin.compose.koinInject

@Composable
fun NotificationsSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Notifications",
        onNavigateBack = onBack
    ) {
        NotificationsSettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun NotificationsSettingsContent(settingsStore: SettingsStore = koinInject()) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Push Notifications",
            items = listOf(
                switchSettingsItem(
                    title = "Enable Notifications",
                    description = "Receive push notifications on your device.",
                    checked = settingsStore.notificationsEnabled,
                    onCheckedChange = { settingsStore.notificationsEnabled = it }
                ),
                switchSettingsItem(
                    title = "Show Message Preview",
                    description = "Include message content in notifications.",
                    checked = settingsStore.showMessagePreview,
                    onCheckedChange = { settingsStore.showMessagePreview = it }
                )
            )
        )

        Material3SettingsGroup(
            title = "Sounds",
            items = listOf(
                switchSettingsItem(
                    title = "Message Sound",
                    description = "Play a sound when you receive a message.",
                    checked = settingsStore.notificationSound,
                    onCheckedChange = { settingsStore.notificationSound = it }
                ),
                switchSettingsItem(
                    title = "Incoming Call Sound",
                    description = "Play a sound when you are being called.",
                    checked = true, // TODO: persistent state
                    onCheckedChange = { }
                )
            )
        )

        Material3SettingsGroup(
            title = "In-App Notifications",
            items = listOf(
                switchSettingsItem(
                    title = "Show In-App Notifications",
                    description = "Display banners in the app when someone mentions or pings you.",
                    checked = settingsStore.showInAppNotifications,
                    onCheckedChange = { settingsStore.showInAppNotifications = it }
                )
            )
        )
    }
}
