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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@Composable
fun ServerChannels(
    guild: Guild,
    guildStore: GuildStore = koinInject(),
    channelApi: ChannelApi = koinInject()
) {
    val allGuildChannels by guildStore.allGuildChannels.collectAsState()
    val allChannels = allGuildChannels.values.filter { it.guild_id == guild.id }
    
    val categories = allChannels.filter { it.type == 4 }.sortedBy { it.position }
    val uncategorized = allChannels.filter { it.parent_id == null && it.type != 4 }.sortedBy { it.position }

    var editingChannel by remember { mutableStateOf<Channel?>(null) }

    if (editingChannel != null) {
        ChannelEditor(editingChannel!!, channelApi, onDone = { editingChannel = null })
    } else {
        SettingsLayout {
            Material3SettingsGroup(title = "Channels") {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    uncategorized.forEach { channel ->
                        ChannelRow(channel, onClick = { editingChannel = channel })
                    }
                    categories.forEach { category ->
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = (category.name ?: "Category").uppercase(),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            fontWeight = FontWeight.Black,
                            modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp).clickableCursor { editingChannel = category }
                        )
                        allChannels.filter { it.parent_id == category.id }.sortedBy { it.position }.forEach { channel ->
                            ChannelRow(channel, onClick = { editingChannel = channel })
                        }
                    }
                }
            }
            Spacer(modifier = Modifier.height(100.dp))
        }
    }
}

@Composable
private fun ChannelRow(channel: Channel, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
        onClick = onClick
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(
                imageVector = when (channel.type) {
                    4 -> Icons.Rounded.Folder
                    2, 13 -> Icons.AutoMirrored.Rounded.VolumeUp
                    else -> Icons.Rounded.Tag
                },
                contentDescription = null,
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(channel.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(1f))
            Icon(Icons.Rounded.ChevronRight, null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f))
        }
    }
}

@Composable
fun ChannelEditor(channel: Channel, channelApi: ChannelApi, onDone: () -> Unit, guildStore: GuildStore = koinInject()) {
    val scope = rememberCoroutineScope()
    var draftName by remember(channel.name) { mutableStateOf(channel.name ?: "") }
    var draftTopic by remember(channel.topic) { mutableStateOf(channel.topic ?: "") }
    var draftNsfw by remember(channel.nsfw) { mutableStateOf(channel.nsfw ?: false) }

    val hasChanges = draftName != (channel.name ?: "") || draftTopic != (channel.topic ?: "") || draftNsfw != (channel.nsfw ?: false)

    SettingsLayout {
        Material3SettingsGroup(title = if (channel.type == 4) "Category Details" else "Channel Details") {
            Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Name", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    OutlinedTextField(
                        value = draftName,
                        onValueChange = { draftName = it },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = RoundedCornerShape(8.dp)
                    )
                }

                if (channel.type != 4 && channel.type != 2 && channel.type != 13) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Topic", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        OutlinedTextField(
                            value = draftTopic,
                            onValueChange = { draftTopic = it },
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(8.dp)
                        )
                    }

                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("Age-Restricted Channel", style = MaterialTheme.typography.bodyLarge)
                            Text("Users will need to confirm they are of over legal age to view this channel.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        ExpressiveSwitch(checked = draftNsfw, onCheckedChange = { draftNsfw = it })
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
                    TextButton(onClick = onDone) {
                        Text("Cancel")
                    }
                    Button(
                        onClick = {
                            scope.launch {
                                if (channelApi.updateChannel(channel.id, draftName, draftTopic, draftNsfw)) {
                                    onDone()
                                }
                            }
                        },
                        colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF43B581))
                    ) {
                        Text("Save Changes")
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        Material3SettingsGroup(title = "Danger Zone") {
            Button(
                onClick = {
                    scope.launch {
                        if (channelApi.deleteChannel(channel.id)) {
                            guildStore.handleChannelDelete(channel)
                            onDone()
                        }
                    }
                },
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer,
                    contentColor = MaterialTheme.colorScheme.onErrorContainer
                ),
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                shape = RoundedCornerShape(8.dp)
            ) {
                Icon(Icons.Default.Delete, null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text("Delete Channel")
            }
        }
        
        Spacer(modifier = Modifier.height(100.dp))
    }
}
