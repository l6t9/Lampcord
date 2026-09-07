package me.lampu.lampcord.shared.ui.components.guilds.settings

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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.ui.components.RoleIcon
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.model.Role as DiscordRole

@Composable
fun ServerRoles(guild: Guild, onRoleClick: (DiscordRole) -> Unit) {
    SettingsLayout {
        Material3SettingsGroup(title = "Roles") {
            Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                guild.roles.sortedByDescending { it.position }.forEach { role ->
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
                        onClick = { onRoleClick(role) }
                    ) {
                        Row(
                            modifier = Modifier.padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(20.dp)
                                    .background(
                                        if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant,
                                        CircleShape
                                    )
                            )
                            RoleIcon(role, size = 18.dp)
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
        Spacer(modifier = Modifier.height(100.dp))
    }
}

@Composable
fun RoleEditor(role: DiscordRole, guild: Guild, guildApi: GuildApi = koinInject()) {
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
        Material3SettingsGroup(title = "Display Settings") {
            Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Role Name", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    OutlinedTextField(
                        value = draftName,
                        onValueChange = { draftName = it },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = RoundedCornerShape(8.dp)
                    )
                }
                
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Role Color", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(if (draftColor != 0) Color(draftColor.toLong() or 0xFF000000L) else Color.Gray)
                            .clickable { showColorPicker = true }
                    )
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Display separately", style = MaterialTheme.typography.bodyLarge)
                        Text("Display role members separately from online members", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    ExpressiveSwitch(checked = draftHoist, onCheckedChange = { draftHoist = it })
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Allow mention", style = MaterialTheme.typography.bodyLarge)
                        Text("Allow anyone to @mention this role", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    ExpressiveSwitch(checked = draftMentionable, onCheckedChange = { draftMentionable = it })
                }
            }
        }

        Material3SettingsGroup(title = "Permissions") {
            Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                me.lampu.lampcord.shared.utils.Permission.entries.forEach { permission ->
                    val permissionsLong = draftPermissions.toULongOrNull()?.toLong() ?: 0L
                    val isEnabled = (permissionsLong and permission.value) != 0L
                    
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
                        onClick = {
                            val newPerms = if (isEnabled) permissionsLong and permission.value.inv() else permissionsLong or permission.value
                            draftPermissions = newPerms.toULong().toString()
                        }
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(12.dp)
                        ) {
                            Text(
                                permission.name.lowercase().split("_").joinToString(" ") { it.replaceFirstChar { char -> char.uppercase() } },
                                style = MaterialTheme.typography.bodyLarge,
                                modifier = Modifier.weight(1f)
                            )
                            ExpressiveSwitch(checked = isEnabled, onCheckedChange = {
                                val newPerms = if (it) permissionsLong or permission.value else permissionsLong and permission.value.inv()
                                draftPermissions = newPerms.toULong().toString()
                            })
                        }
                    }
                }
            }
        }
        
        if (hasChanges) {
            Surface(
                modifier = Modifier.fillMaxWidth().padding(top = 16.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                shape = RoundedCornerShape(12.dp)
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End), modifier = Modifier.padding(12.dp)) {
                    TextButton(onClick = {
                        draftName = role.name
                        draftColor = role.color
                        draftHoist = role.hoist
                        draftMentionable = role.mentionable
                        draftPermissions = role.permissions
                    }) {
                        Text("Reset")
                    }
                    Button(
                        onClick = {
                            scope.launch {
                                guildApi.updateRole(guild.id, role.id, DiscordRole.Partial(
                                    name = draftName,
                                    color = draftColor,
                                    hoist = draftHoist,
                                    mentionable = draftMentionable,
                                    permissions = draftPermissions
                                ))
                            }
                        },
                        colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF43B581))
                    ) {
                        Text("Save Changes")
                    }
                }
            }
        }
        Spacer(modifier = Modifier.height(100.dp))
    }
}

@Composable
fun RoleEditorSubScreen(role: DiscordRole, guild: Guild, onBack: () -> Unit) {
    SettingsSubScreen(title = "Edit Role", onNavigateBack = onBack) {
        RoleEditor(role, guild)
    }
}
