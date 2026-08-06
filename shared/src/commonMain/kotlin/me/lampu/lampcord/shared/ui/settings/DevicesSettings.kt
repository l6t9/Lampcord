package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun DevicesSettings(chatState: ChatState) {
    val devices = emptyList<me.lampu.lampcord.shared.api.DiscordDevice>()

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Active Sessions",
            items = devices.map { device ->
                Material3SettingsItem(
                    icon = if (device.os?.contains("Android", true) == true) Icons.Filled.Smartphone else Icons.Filled.Tv,
                    title = { Text(device.model ?: "Unknown Device") },
                    description = { Text("${device.os} • ${device.location ?: "Unknown Location"}") },
                    trailingContent = {
                        TextButton(onClick = { /* TODO */ }) {
                            Text("Log Out", color = MaterialTheme.colorScheme.error)
                        }
                    }
                )
            }
        )

        if (devices.isEmpty()) {
            Material3SettingsGroup {
                Box(
                    modifier = Modifier.fillMaxWidth().padding(32.dp),
                    contentAlignment = androidx.compose.ui.Alignment.Center
                ) {
                    Text("No other active sessions found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }

        Spacer(Modifier.height(16.dp))

        Material3SettingsGroup(title = "Security") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Log Out All Other Sessions", color = MaterialTheme.colorScheme.error) },
                        description = { Text("If you see a device you don't recognize, sign out of all other sessions.") },
                        onClick = { /* TODO */ }
                    )
                ),
                horizontalPadding = 0.dp
            )
        }
    }
}
