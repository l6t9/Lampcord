package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun ConnectionsSettings(onBack: () -> Unit) {
    SettingsSubScreen(
        title = "Connections",
        onNavigateBack = onBack
    ) {
        ConnectionsSettingsContent()
    }
}

@Composable
fun ConnectionsSettingsContent() {
    val connections = emptyList<me.lampu.lampcord.shared.model.ConnectedAccount>()

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Connected Accounts",
            items = buildList {
                if (connections.isEmpty()) {
                    add(Material3SettingsItem(
                        title = { 
                            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = androidx.compose.ui.Alignment.Center) {
                                Text("No connected accounts found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    ))
                } else {
                    addAll(connections.map { connection ->
                        Material3SettingsItem(
                            icon = Icons.Filled.Public,
                            title = { Text(connection.name) },
                            description = { Text(connection.type) },
                            trailingContent = {
                                IconButton(onClick = { /* TODO */ }) {
                                    Icon(Icons.Filled.Delete, null, tint = MaterialTheme.colorScheme.error)
                                }
                            }
                        )
                    })
                }
            }
        )

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
