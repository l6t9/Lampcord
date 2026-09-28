package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsButtonGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ServerOverview(guild: Guild, guildStore: GuildStore = koinInject()) {
    val scope = rememberCoroutineScope()
    
    var draftName by remember(guild.id, guild.name) { mutableStateOf(guild.name ?: "") }
    var draftAfkChannelId by remember(guild.id, guild.afk_channel_id) { mutableStateOf(guild.afk_channel_id) }
    var draftAfkTimeout by remember(guild.id, guild.afk_timeout) { mutableStateOf(guild.afk_timeout ?: 300) }
    var draftSystemChannelId by remember(guild.id, guild.system_channel_id) { mutableStateOf(guild.system_channel_id) }
    var draftDefaultNotifications by remember(guild.id, guild.default_message_notifications) { mutableStateOf(guild.default_message_notifications ?: 0) }
    var draftVerificationLevel by remember(guild.id, guild.verification_level) { mutableStateOf(guild.verification_level ?: 0) }
    var draftExplicitContentFilter by remember(guild.id, guild.explicit_content_filter) { mutableStateOf(guild.explicit_content_filter ?: 0) }
    
    val hasChanges = draftName != (guild.name ?: "") || 
                     draftAfkChannelId != guild.afk_channel_id || 
                     draftAfkTimeout != (guild.afk_timeout ?: 300) ||
                     draftSystemChannelId != guild.system_channel_id ||
                     draftDefaultNotifications != (guild.default_message_notifications ?: 0) ||
                     draftVerificationLevel != (guild.verification_level ?: 0) ||
                     draftExplicitContentFilter != (guild.explicit_content_filter ?: 0)

    Box(modifier = Modifier.fillMaxSize()) {
        ServerOverviewContent(
            guild = guild,
            guildStore = guildStore,
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

        if (hasChanges) {
            Surface(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .padding(16.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                shape = RoundedCornerShape(12.dp),
                tonalElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier.padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        "You have unsaved changes!",
                        modifier = Modifier.weight(1f),
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.Bold
                    )
                    TextButton(onClick = {
                        draftName = guild.name ?: ""
                        draftAfkChannelId = guild.afk_channel_id
                        draftAfkTimeout = guild.afk_timeout ?: 300
                        draftSystemChannelId = guild.system_channel_id
                        draftDefaultNotifications = guild.default_message_notifications ?: 0
                        draftVerificationLevel = guild.verification_level ?: 0
                        draftExplicitContentFilter = guild.explicit_content_filter ?: 0
                    }) {
                        Text("Reset")
                    }
                    Button(
                        onClick = {
                            scope.launch {
                                guildStore.updateGuild(guild.id, Guild.Partial(
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
                        colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF43B581))
                    ) {
                        Text("Save Changes")
                    }
                }
            }
        }
    }
}

@Composable
private fun ServerOverviewContent(
    guild: Guild,
    guildStore: GuildStore,
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
    SettingsLayout {
        Material3SettingsGroup(title = "Server Details") {
            Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Server Name", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    OutlinedTextField(
                        value = draftName,
                        onValueChange = onNameChange,
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = RoundedCornerShape(8.dp)
                    )
                }
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text("Server ID", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(guild.id, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurface)
                }
            }
        }

        val allChannels by guildStore.allGuildChannels.collectAsState()
        
        Material3SettingsGroup(title = "Channel Settings") {
            Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("AFK Channel", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    val voiceChannels = allChannels.values.filter { it.guild_id == guild.id && it.type == 2 }
                    SettingsButtonGroup(
                        options = listOf(null) + voiceChannels.map { it.id },
                        selectedOption = draftAfkChannelId,
                        onOptionSelected = onAfkChannelChange,
                        labelProvider = { id -> if (id == null) "No AFK Channel" else voiceChannels.find { it.id == id }?.name ?: "Unknown" }
                    )
                }
                
                if (draftAfkChannelId != null) {
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text("AFK Timeout", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        SettingsButtonGroup(
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
                    Text("System Messages Channel", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    val textChannels = allChannels.values.filter { it.guild_id == guild.id && it.type == 0 }
                    SettingsButtonGroup(
                        options = listOf(null) + textChannels.map { it.id },
                        selectedOption = draftSystemChannelId,
                        onOptionSelected = onSystemChannelChange,
                        labelProvider = { id -> if (id == null) "No System Channel" else textChannels.find { it.id == id }?.name ?: "Unknown" }
                    )
                }
            }
        }

        Material3SettingsGroup(title = "Notification Settings") {
            Column(modifier = Modifier.padding(16.dp)) {
                Text("Default Notification Level", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, modifier = Modifier.padding(bottom = 12.dp))
                SettingsButtonGroup(
                    options = listOf(0, 1),
                    selectedOption = draftDefaultNotifications,
                    onOptionSelected = onDefaultNotificationsChange,
                    labelProvider = { if (it == 0) "All Messages" else "Only @mentions" }
                )
            }
        }

        Material3SettingsGroup(title = "Safety Setup") {
            Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Verification Level", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    SettingsButtonGroup(
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
                    Text("Explicit Content Filter", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    SettingsButtonGroup(
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
        
        Spacer(modifier = Modifier.height(100.dp))
    }
}
