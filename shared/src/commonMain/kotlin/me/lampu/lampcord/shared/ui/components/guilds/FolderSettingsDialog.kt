package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.GuildFolder
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private val folderColors = listOf(
    0xFF5865F2.toInt(), 0xFFEB459E.toInt(), 0xFFF9A825.toInt(), 0xFF3EBA8D.toInt(), 0xFFE91E63.toInt(),
    0xFF9B59B6.toInt(), 0xFFF57C00.toInt(), 0xFF00BCD4.toInt(), 0xFF8D6E63.toInt(), 0xFF4CAF50.toInt(),
    0xFF607D8B.toInt(), 0xFFC2185B.toInt(), 0xFF795548.toInt(), 0xFF3D5AFE.toInt()
)

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun FolderSettingsDialog(
    folder: GuildFolder,
    onDismiss: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    var name by remember(folder) { mutableStateOf(folder.name ?: "") }
    var selectedColor by remember(folder) { mutableStateOf<Int?>(folder.color) }

    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                imageVector = Icons.Filled.FolderOpen,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        },
        title = {
            Text(
                text = "Folder Settings",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        },
        text = {
            Column(modifier = Modifier.fillMaxWidth()) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Folder Name") },
                    placeholder = { Text("Folder") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(Modifier.height(16.dp))
                Text(
                    text = "Folder Color",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(
                        modifier = Modifier
                            .size(28.dp)
                            .clip(CircleShape)
                            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
                            .clickable { selectedColor = null }
                    ) {
                        if (selectedColor == null) {
                            Icon(
                                imageVector = Icons.Filled.Close,
                                contentDescription = "No color",
                                modifier = Modifier.padding(6.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                    folderColors.forEach { color ->
                        Box(
                            modifier = Modifier
                                .size(28.dp)
                                .clip(CircleShape)
                                .background(Color(color))
                                .clickable { selectedColor = color }
                        ) {
                            if (selectedColor == color) {
                                Icon(
                                    imageVector = Icons.Filled.Check,
                                    contentDescription = null,
                                    modifier = Modifier.padding(6.dp),
                                    tint = Color.White
                                )
                            }
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    val updated = folder.copy(
                        name = name.ifBlank { null },
                        color = selectedColor
                    )
                    val folders = (settingsStore.userSettings?.guild_folders ?: emptyList())
                        .map { if (it.guild_ids == folder.guild_ids) updated else it }
                    settingsStore.updateUserSettings(UserSettings.Partial(guild_folders = folders))
                    onDismiss()
                }
            ) {
                Text("Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        },
        shape = MaterialTheme.shapes.extraLarge
    )
}
