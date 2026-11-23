package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.add
import me.lampu.lampcord.shared.generated.resources.connected_accounts
import me.lampu.lampcord.shared.generated.resources.connections
import me.lampu.lampcord.shared.generated.resources.delete
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_add_connection
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_no_connections
import org.jetbrains.compose.resources.stringResource

@Composable
fun ConnectionsSettings(onBack: () -> Unit) {
    SettingsSubScreen(
        title = stringResource(Res.string.connections),
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
            title = stringResource(Res.string.connected_accounts),
            items = buildList {
                if (connections.isEmpty()) {
                    add(Material3SettingsItem(
                        title = { 
                            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = androidx.compose.ui.Alignment.Center) {
                                Text(stringResource(Res.string.lampcord_settings_no_connections), color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    ))
                } else {
                    addAll(connections.map { connection ->
                        Material3SettingsItem(
                            icon = Icons.Filled.Public,
                            title = { Text(connection.name) },
                            value = connection.type,
                            trailingPreview = {
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
                    title = { Text(stringResource(Res.string.lampcord_settings_add_connection)) },
                    onClick = { /* TODO */ }
                )
            )
        )
    }
}
