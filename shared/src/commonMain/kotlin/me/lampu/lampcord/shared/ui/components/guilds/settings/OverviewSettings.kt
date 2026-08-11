package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.coroutines.launch

@Composable
fun ServerOverview(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }
    val scope = rememberCoroutineScope()
    
    // Draft state for unsaved changes
    var draftName by remember(guild.name) { mutableStateOf(guild.name ?: "") }
    var draftAfkChannelId by remember(guild.afk_channel_id) { mutableStateOf(guild.afk_channel_id) }
    var draftAfkTimeout by remember(guild.afk_timeout) { mutableStateOf(guild.afk_timeout ?: 300) }
    var draftSystemChannelId by remember(guild.system_channel_id) { mutableStateOf(guild.system_channel_id) }
    var draftDefaultNotifications by remember(guild.default_message_notifications) { mutableStateOf(guild.default_message_notifications ?: 0) }
    var draftVerificationLevel by remember(guild.verification_level) { mutableStateOf(guild.verification_level ?: 0) }
    var draftExplicitContentFilter by remember(guild.explicit_content_filter) { mutableStateOf(guild.explicit_content_filter ?: 0) }
    
    val hasChanges = draftName != (guild.name ?: "") || 
                     draftAfkChannelId != guild.afk_channel_id || 
                     draftAfkTimeout != (guild.afk_timeout ?: 300) ||
                     draftSystemChannelId != guild.system_channel_id ||
                     draftDefaultNotifications != (guild.default_message_notifications ?: 0) ||
                     draftVerificationLevel != (guild.verification_level ?: 0) ||
                     draftExplicitContentFilter != (guild.explicit_content_filter ?: 0)

    if (!isMobile) {
        DesktopServerOverview(
            guild = guild,
            chatState = chatState,
            draftName = draftName,
            onNameChange = { draftName = it },
            draftAfkChannelId = draftAfkChannelId,
            onAfkChannelChange = { draftAfkChannelId = it },
            draftAfkTimeout = draftAfkTimeout,
            onAfkTimeoutChange = { draftAfkTimeout = it },
            draftSystemChannelId = draftSystemChannelId,
            onSystemChannelChange = { draftSystemChannelId = it },
            draftDefaultNotifications = draftDefaultNotifications,
            onDefaultNotificationsChange = { draftDefaultNotifications = it },
            draftVerificationLevel = draftVerificationLevel,
            onVerificationLevelChange = { draftVerificationLevel = it },
            draftExplicitContentFilter = draftExplicitContentFilter,
            onExplicitContentFilterChange = { draftExplicitContentFilter = it },
            hasChanges = hasChanges,
            onSave = {
                scope.launch {
                    chatState.updateGuild(guild.id, Guild.Partial(
                        name = draftName,
                        afk_channel_id = draftAfkChannelId,
                        afk_timeout = draftAfkTimeout,
                        system_channel_id = draftSystemChannelId,
                        default_message_notifications = draftDefaultNotifications,
                        verification_level = draftVerificationLevel,
                        explicit_content_filter = draftExplicitContentFilter
                    ))
                }
            },
            onReset = {
                draftName = guild.name ?: ""
                draftAfkChannelId = guild.afk_channel_id
                draftAfkTimeout = guild.afk_timeout ?: 300
                draftSystemChannelId = guild.system_channel_id
                draftDefaultNotifications = guild.default_message_notifications ?: 0
                draftVerificationLevel = guild.verification_level ?: 0
                draftExplicitContentFilter = guild.explicit_content_filter ?: 0
            }
        )
    } else {
        MobileServerOverview(
            guild = guild,
            chatState = chatState,
            draftName = draftName,
            onNameChange = { draftName = it },
            draftAfkChannelId = draftAfkChannelId,
            onAfkChannelChange = { draftAfkChannelId = it },
            draftAfkTimeout = draftAfkTimeout,
            onAfkTimeoutChange = { draftAfkTimeout = it },
            draftSystemChannelId = draftSystemChannelId,
            onSystemChannelChange = { draftSystemChannelId = it },
            draftDefaultNotifications = draftDefaultNotifications,
            onDefaultNotificationsChange = { draftDefaultNotifications = it },
            draftVerificationLevel = draftVerificationLevel,
            onVerificationLevelChange = { draftVerificationLevel = it },
            draftExplicitContentFilter = draftExplicitContentFilter,
            onExplicitContentFilterChange = { draftExplicitContentFilter = it }
        )
    }
}

@Composable
private fun DesktopServerOverview(
    guild: Guild,
    chatState: ChatState,
    draftName: String,
    onNameChange: (String) -> Unit,
    draftAfkChannelId: String?,
    onAfkChannelChange: (String?) -> Unit,
    draftAfkTimeout: Int,
    onAfkTimeoutChange: (Int) -> Unit,
    draftSystemChannelId: String?,
    onSystemChannelChange: (String?) -> Unit,
    draftDefaultNotifications: Int,
    onDefaultNotificationsChange: (Int) -> Unit,
    draftVerificationLevel: Int,
    onVerificationLevelChange: (Int) -> Unit,
    draftExplicitContentFilter: Int,
    onExplicitContentFilterChange: (Int) -> Unit,
    hasChanges: Boolean,
    onSave: () -> Unit,
    onReset: () -> Unit
) {
    DesktopSettingsLayout {
        DesktopSettingsSection(
            title = "Server Details",
            icon = Icons.Filled.Info
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Server Name", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    OutlinedTextField(
                        value = draftName,
                        onValueChange = onNameChange,
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = RoundedCornerShape(12.dp)
                    )
                }
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Server ID", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    Text(guild.id, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
                }
            }
        }

        DesktopSettingsSection(
            title = "Channels",
            icon = Icons.Filled.Tag
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("AFK Channel", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    val voiceChannels = chatState.channels.filter { it.guild_id == guild.id && it.type == 2 }
                    DesktopButtonGroupSelection(
                        options = listOf(null) + voiceChannels.map { it.id },
                        selectedOption = draftAfkChannelId,
                        onOptionSelected = onAfkChannelChange,
                        labelProvider = { id -> if (id == null) "No AFK Channel" else voiceChannels.find { it.id == id }?.name ?: "Unknown" }
                    )
                }
                
                if (draftAfkChannelId != null) {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text("AFK Timeout", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                        DesktopButtonGroupSelection(
                            options = listOf(60, 300, 900, 1800, 3600),
                            selectedOption = draftAfkTimeout,
                            onOptionSelected = onAfkTimeoutChange,
                            labelProvider = {
                                when (it) {
                                    60 -> "1 minute"
                                    300 -> "5 minutes"
                                    900 -> "15 minutes"
                                    1800 -> "30 minutes"
                                    else -> "1 hour"
                                }
                            }
                        )
                    }
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("System Messages Channel", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    val textChannels = chatState.channels.filter { it.guild_id == guild.id && it.type == 0 }
                    DesktopButtonGroupSelection(
                        options = listOf(null) + textChannels.map { it.id },
                        selectedOption = draftSystemChannelId,
                        onOptionSelected = onSystemChannelChange,
                        labelProvider = { id -> if (id == null) "No System Channel" else textChannels.find { it.id == id }?.name ?: "Unknown" }
                    )
                }
            }
        }

        DesktopSettingsSection(
            title = "Default Notifications",
            icon = Icons.Filled.Notifications
        ) {
            DesktopButtonGroupSelection(
                options = listOf(0, 1),
                selectedOption = draftDefaultNotifications,
                onOptionSelected = onDefaultNotificationsChange,
                labelProvider = { if (it == 0) "All Messages" else "Only @mentions" }
            )
        }

        DesktopSettingsSection(
            title = "Safety Setup",
            icon = Icons.Filled.Security
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Verification Level", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    DesktopButtonGroupSelection(
                        options = listOf(0, 1, 2, 3, 4),
                        selectedOption = draftVerificationLevel,
                        onOptionSelected = onVerificationLevelChange,
                        iconProvider = { level: Int, isSelected ->
                            when (level) {
                                0 -> if (isSelected) Icons.Filled.Block else Icons.Rounded.Block
                                1 -> if (isSelected) Icons.Filled.ChevronRight else Icons.Rounded.ChevronRight
                                2 -> if (isSelected) Icons.Filled.BarChart else Icons.Rounded.BarChart
                                else -> if (isSelected) Icons.Filled.Security else Icons.Rounded.Security
                            }
                        },
                        labelProvider = {
                            when (it) {
                                0 -> "None"
                                1 -> "Low"
                                2 -> "Medium"
                                3 -> "High"
                                else -> "Highest"
                            }
                        }
                    )
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Explicit Content Filter", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    DesktopButtonGroupSelection(
                        options = listOf(0, 1, 2),
                        selectedOption = draftExplicitContentFilter,
                        onOptionSelected = onExplicitContentFilterChange,
                        iconProvider = { filter: Int, isSelected ->
                            when (filter) {
                                0 -> if (isSelected) Icons.Filled.Block else Icons.Rounded.Block
                                1 -> if (isSelected) Icons.Filled.Person else Icons.Rounded.Person
                                else -> if (isSelected) Icons.Filled.Groups else Icons.Rounded.Groups
                            }
                        },
                        labelProvider = {
                            when (it) {
                                0 -> "Don't scan"
                                1 -> "Members"
                                else -> "Everyone"
                            }
                        }
                    )
                }
            }
        }
        
        if (hasChanges) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(top = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End)
            ) {
                TextButton(onClick = onReset) {
                    Text("Reset")
                }
                Button(onClick = onSave) {
                    Text("Save Changes")
                }
            }
        }
    }
}

@Composable
private fun MobileServerOverview(
    guild: Guild,
    chatState: ChatState,
    draftName: String,
    onNameChange: (String) -> Unit,
    draftAfkChannelId: String?,
    onAfkChannelChange: (String?) -> Unit,
    draftAfkTimeout: Int,
    onAfkTimeoutChange: (Int) -> Unit,
    draftSystemChannelId: String?,
    onSystemChannelChange: (String?) -> Unit,
    draftDefaultNotifications: Int,
    onDefaultNotificationsChange: (Int) -> Unit,
    draftVerificationLevel: Int,
    onVerificationLevelChange: (Int) -> Unit,
    draftExplicitContentFilter: Int,
    onExplicitContentFilterChange: (Int) -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Material3SettingsGroup(
            title = "Server Details",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Server Name") },
                    description = { Text(draftName) },
                    onClick = { /* TODO: edit name dialog/screen */ }
                ),
                Material3SettingsItem(
                    title = { Text("Server ID") },
                    description = { Text(guild.id) },
                    onClick = { /* TODO: copy ID */ }
                )
            )
        )

        Material3SettingsGroup(
            title = "Channels",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("AFK Channel") },
                    description = { 
                        Text(chatState.channels.find { it.id == draftAfkChannelId }?.name ?: "No AFK Channel") 
                    },
                    onClick = { /* TODO: selector */ }
                ),
                Material3SettingsItem(
                    title = { Text("System Messages Channel") },
                    description = {
                        Text(chatState.channels.find { it.id == draftSystemChannelId }?.name ?: "No System Channel")
                    },
                    onClick = { /* TODO: selector */ }
                )
            )
        )

        Material3SettingsGroup(
            title = "Security",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Verification Level") },
                    description = {
                        Text(
                            when (draftVerificationLevel) {
                                0 -> "None"
                                1 -> "Low"
                                2 -> "Medium"
                                3 -> "High"
                                4 -> "Highest"
                                else -> "None"
                            }
                        )
                    },
                    onClick = { 
                        // Cycle for mobile simplicity or we could use a proper picker
                        onVerificationLevelChange((draftVerificationLevel + 1) % 5)
                    }
                ),
                Material3SettingsItem(
                    title = { Text("Explicit Content Filter") },
                    description = {
                        Text(
                            when (draftExplicitContentFilter) {
                                0 -> "Don't scan any messages"
                                1 -> "Scan messages from members without a role"
                                2 -> "Scan messages from all members"
                                else -> "Don't scan"
                            }
                        )
                    },
                    onClick = { 
                        onExplicitContentFilterChange((draftExplicitContentFilter + 1) % 3)
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Notifications",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Default Notification Settings") },
                    description = {
                        Text(if (draftDefaultNotifications == 0) "All Messages" else "Only @mentions")
                    },
                    onClick = { /* TODO: picker */ }
                )
            )
        )
    }
}
