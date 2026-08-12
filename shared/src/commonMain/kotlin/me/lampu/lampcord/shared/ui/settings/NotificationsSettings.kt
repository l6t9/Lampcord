package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import org.koin.compose.koinInject

@Composable
fun NotificationsSettings() {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopNotificationsSettings()
    } else {
        MobileNotificationsSettings()
    }
}

@Composable
private fun DesktopNotificationsSettings(settingsStore: SettingsStore = koinInject()) {
    SettingsLayout {
        SettingsSection(
            title = "Push Notifications",
            icon = Icons.Filled.Notifications
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                NotificationToggle("Enable Notifications", settingsStore.notificationsEnabled, "Receive push notifications on your device.") {
                    settingsStore.notificationsEnabled = it
                }
                NotificationToggle("Show Message Preview", settingsStore.showMessagePreview, "Include message content in notifications.") {
                    settingsStore.showMessagePreview = it
                }
            }
        }

        SettingsSection(
            title = "Sounds",
            icon = Icons.Filled.VolumeUp
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                var callSound by remember { mutableStateOf(true) }

                NotificationToggle("Message Sound", settingsStore.notificationSound, "Play a sound when you receive a message.") {
                    settingsStore.notificationSound = it
                }
                NotificationToggle("Incoming Call Sound", callSound, "Play a sound when you are being called.") {
                    callSound = it
                }
            }
        }

        SettingsSection(
            title = "In-App Notifications",
            icon = Icons.Filled.Notifications
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                NotificationToggle("Show In-App Notifications", settingsStore.showInAppNotifications, "Display banners for new messages while using the app.") {
                    settingsStore.showInAppNotifications = it
                }
            }
        }
    }
}

@Composable
private fun NotificationToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
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
private fun MobileNotificationsSettings(settingsStore: SettingsStore = koinInject()) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Push Notifications") {
            Material3SettingsGroup(
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
        }

        Material3SettingsGroup(title = "Sounds") {
            var callSound by remember { mutableStateOf(true) }

            Material3SettingsGroup(
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
                        checked = callSound,
                        onCheckedChange = { callSound = it }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "In-App Notifications") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Show In-App Notifications",
                        description = "Display banners for new messages while using the app.",
                        checked = settingsStore.showInAppNotifications,
                        onCheckedChange = { settingsStore.showInAppNotifications = it }
                    )
                )
            )
        }
    }
}
