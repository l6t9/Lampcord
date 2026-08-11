package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Role as DiscordRole
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.coroutines.launch

@Composable
fun ServerRoles(guild: Guild, chatState: ChatState, onRoleClick: (DiscordRole) -> Unit) {
    SettingsLayout {
        SettingsSection(
            title = "Roles",
            icon = Icons.Filled.Flag
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                guild.roles.sortedByDescending { it.position }.forEach { role ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(min = 48.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                            .clickable { onRoleClick(role) }
                            .padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(16.dp)
                                .background(
                                    if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant,
                                    CircleShape
                                )
                        )
                        Text(role.name, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(1f))
                        if (role.managed) {
                            Icon(
                                Icons.Filled.Security,
                                null,
                                modifier = Modifier.size(16.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun RoleEditor(role: DiscordRole, guild: Guild, chatState: ChatState) {
    val scope = rememberCoroutineScope()
    var draftName by remember(role.name) { mutableStateOf(role.name) }
    var draftColor by remember(role.color) { mutableStateOf(role.color) }
    var draftHoist by remember(role.hoist) { mutableStateOf(role.hoist) }
    var draftMentionable by remember(role.mentionable) { mutableStateOf(role.mentionable) }
    var draftPermissions by remember(role.permissions) { mutableStateOf(role.permissions) }
    
    var showColorPicker by remember { mutableStateOf(false) }

    val hasChanges = draftName != role.name || draftColor != role.color || draftHoist != role.hoist || draftMentionable != role.mentionable || draftPermissions != role.permissions

    if (showColorPicker) {
        androidx.compose.ui.window.Dialog(onDismissRequest = { showColorPicker = false }) {
            HsvColorPicker(
                initialColor = if (draftColor != 0) Color(draftColor.toLong() or 0xFF000000L) else Color.Gray,
                onColorSelected = { 
                    draftColor = (it.red * 255).toInt() shl 16 or ((it.green * 255).toInt() shl 8) or (it.blue * 255).toInt()
                    showColorPicker = false 
                },
                onDismiss = { showColorPicker = false }
            )
        }
    }

    SettingsLayout {
        SettingsSection(title = "Display", icon = Icons.Filled.Info) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Role Name", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    OutlinedTextField(
                        value = draftName,
                        onValueChange = { draftName = it },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = RoundedCornerShape(12.dp)
                    )
                }
                
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Role Color", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape)
                            .background(if (draftColor != 0) Color(draftColor.toLong() or 0xFF000000L) else Color.Gray)
                            .clickable { showColorPicker = true }
                    )
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Display role members separately from online members", style = MaterialTheme.typography.bodyLarge)
                    }
                    ExpressiveSwitch(checked = draftHoist, onCheckedChange = { draftHoist = it })
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Allow anyone to @mention this role", style = MaterialTheme.typography.bodyLarge)
                    }
                    ExpressiveSwitch(checked = draftMentionable, onCheckedChange = { draftMentionable = it })
                }
            }
        }

        SettingsSection(title = "Permissions", icon = Icons.Filled.Security) {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                me.lampu.lampcord.shared.utils.Permission.entries.forEach { permission ->
                    val permissionsLong = draftPermissions.toULongOrNull()?.toLong() ?: 0L
                    val isEnabled = (permissionsLong and permission.value) != 0L
                    
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .clickable {
                                val newPerms = if (isEnabled) permissionsLong and permission.value.inv() else permissionsLong or permission.value
                                draftPermissions = newPerms.toULong().toString()
                            }
                            .padding(8.dp)
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                permission.name.lowercase().split("_").joinToString(" ") { it.replaceFirstChar { char -> char.uppercase() } },
                                style = MaterialTheme.typography.bodyLarge
                            )
                        }
                        ExpressiveSwitch(checked = isEnabled, onCheckedChange = {
                            val newPerms = if (it) permissionsLong or permission.value else permissionsLong and permission.value.inv()
                            draftPermissions = newPerms.toULong().toString()
                        })
                    }
                }
            }
        }
        
        if (hasChanges) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End), modifier = Modifier.fillMaxWidth().padding(top = 16.dp)) {
                TextButton(onClick = {
                    draftName = role.name
                    draftColor = role.color
                    draftHoist = role.hoist
                    draftMentionable = role.mentionable
                    draftPermissions = role.permissions
                }) {
                    Text("Reset")
                }
                Button(onClick = {
                    scope.launch {
                        chatState.client.updateRole(guild.id, role.id, DiscordRole.Partial(
                            name = draftName,
                            color = draftColor,
                            hoist = draftHoist,
                            mentionable = draftMentionable,
                            permissions = draftPermissions
                        ))
                    }
                }) {
                    Text("Save Changes")
                }
            }
        }
    }
}

@Composable
fun RoleEditorSubScreen(role: DiscordRole, guild: Guild, chatState: ChatState, onBack: () -> Unit) {
    SettingsSubScreen(title = "Edit Role", onNavigateBack = onBack) {
        RoleEditor(role, guild, chatState)
    }
}
