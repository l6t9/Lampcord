package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun NotificationsSettings(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Push Notifications") {
            var enablePush by remember { mutableStateOf(true) }
            var showPreview by remember { mutableStateOf(true) }

            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Enable Notifications",
                        description = "Receive push notifications on your device.",
                        checked = enablePush,
                        onCheckedChange = { enablePush = it }
                    ),
                    switchSettingsItem(
                        title = "Show Message Preview",
                        description = "Include message content in notifications.",
                        checked = showPreview,
                        onCheckedChange = { showPreview = it }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Sounds") {
            var messageSound by remember { mutableStateOf(true) }
            var callSound by remember { mutableStateOf(true) }

            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Message Sound",
                        description = "Play a sound when you receive a message.",
                        checked = messageSound,
                        onCheckedChange = { messageSound = it }
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
    }
}
