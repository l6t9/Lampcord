package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun ConnectionsSettings() {
    val connections = emptyList<me.lampu.lampcord.shared.model.ConnectedAccount>()

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Connected Accounts",
            items = connections.map { connection ->
                Material3SettingsItem(
                    icon = Icons.Filled.Public,
                    title = { Text(connection.name) },
                    description = { Text(connection.type.uppercase()) },
                    trailingContent = {
                        IconButton(onClick = { /* TODO */ }) {
                            Icon(Icons.Filled.Delete, null, tint = MaterialTheme.colorScheme.error)
                        }
                    }
                )
            }
        )

        if (connections.isEmpty()) {
            Material3SettingsGroup {
                Box(
                    modifier = Modifier.fillMaxWidth().padding(32.dp),
                    contentAlignment = androidx.compose.ui.Alignment.Center
                ) {
                    Text("No connected accounts found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }

        Spacer(Modifier.height(16.dp))
        
        Material3SettingsGroup {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        icon = Icons.Filled.Add,
                        title = { Text("Add Connection") },
                        onClick = { /* TODO */ }
                    )
                )
            )
        }
    }
}
