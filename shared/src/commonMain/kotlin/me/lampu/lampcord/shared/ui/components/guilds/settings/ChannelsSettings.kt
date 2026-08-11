package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.coroutines.launch

@Composable
fun ServerChannels(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }
    val allChannels = chatState.channels.filter { it.guild_id == guild.id }
    
    val categories = allChannels.filter { it.type == 4 }.sortedBy { it.position }
    val uncategorized = allChannels.filter { it.parent_id == null && it.type != 4 }.sortedBy { it.position }

    var editingChannel by remember { mutableStateOf<Channel?>(null) }

    if (editingChannel != null) {
        if (isMobile) {
            SettingsSubScreen(title = "Edit Channel", onNavigateBack = { editingChannel = null }) {
                ChannelEditor(editingChannel!!, chatState, onDone = { editingChannel = null })
            }
        } else {
            // On desktop we can just show it in the same area if we had a proper router, 
            // but for now let's just replace the content.
            ChannelEditor(editingChannel!!, chatState, onDone = { editingChannel = null })
        }
    } else {
        if (isMobile) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(uncategorized) { channel ->
                    ChannelItem(channel, onClick = { editingChannel = channel })
                }
                categories.forEach { category ->
                    item {
                        Text(
                            text = category.name?.uppercase() ?: "CATEGORY",
                            style = MaterialTheme.typography.labelSmall,
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { editingChannel = category }
                                .padding(horizontal = 16.dp, vertical = 8.dp),
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    items(allChannels.filter { it.parent_id == category.id }.sortedBy { it.position }) { channel ->
                        ChannelItem(channel, onClick = { editingChannel = channel })
                    }
                }
            }
        } else {
            DesktopSettingsLayout {
                DesktopSettingsSection(title = "Channels", icon = Icons.Filled.Tag) {
                    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                        uncategorized.forEach { channel ->
                            ChannelRow(channel, onClick = { editingChannel = channel })
                        }
                        categories.forEach { category ->
                            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text(
                                    text = category.name?.uppercase() ?: "CATEGORY",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    modifier = Modifier.clickable { editingChannel = category }
                                )
                                allChannels.filter { it.parent_id == category.id }.sortedBy { it.position }.forEach { channel ->
                                    ChannelRow(channel, onClick = { editingChannel = channel })
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ChannelItem(channel: Channel, onClick: () -> Unit) {
    Material3SettingsItem(
        icon = when (channel.type) {
            4 -> Icons.Rounded.Folder
            2 -> Icons.Rounded.VolumeUp
            else -> Icons.Rounded.Tag
        },
        title = { Text(channel.name ?: "unnamed") },
        onClick = onClick
    )
}

@Composable
private fun ChannelRow(channel: Channel, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
            .clickable { onClick() }
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Icon(
            imageVector = when (channel.type) {
                4 -> Icons.Rounded.Folder
                2 -> Icons.Rounded.VolumeUp
                else -> Icons.Rounded.Tag
            },
            contentDescription = null,
            modifier = Modifier.size(18.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(channel.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(1f))
    }
}

@Composable
fun ChannelEditor(channel: Channel, chatState: ChatState, onDone: () -> Unit) {
    val scope = rememberCoroutineScope()
    var draftName by remember(channel.name) { mutableStateOf(channel.name ?: "") }
    var draftTopic by remember(channel.topic) { mutableStateOf(channel.topic ?: "") }
    var draftNsfw by remember(channel.nsfw) { mutableStateOf(channel.nsfw ?: false) }

    val hasChanges = draftName != (channel.name ?: "") || draftTopic != (channel.topic ?: "") || draftNsfw != (channel.nsfw ?: false)

    Column(modifier = Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
        Text(if (channel.type == 4) "Category Settings" else "Channel Settings", style = MaterialTheme.typography.headlineSmall)
        
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("Channel Name", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
            OutlinedTextField(
                value = draftName,
                onValueChange = { draftName = it },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                shape = RoundedCornerShape(12.dp)
            )
        }

        if (channel.type != 4 && channel.type != 2) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Channel Topic", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                OutlinedTextField(
                    value = draftTopic,
                    onValueChange = { draftTopic = it },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp)
                )
            }

            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    Text("Age-Restricted Channel", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.Bold)
                    Text("Users will need to confirm they are of over legal age to view this channel.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                ExpressiveSwitch(checked = draftNsfw, onCheckedChange = { draftNsfw = it })
            }
        }

        Row(horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.End), modifier = Modifier.fillMaxWidth()) {
            TextButton(onClick = onDone) {
                Text("Cancel")
            }
            if (hasChanges) {
                Button(onClick = {
                    scope.launch {
                        if (chatState.client.updateChannel(channel.id, draftName, draftTopic, draftNsfw)) {
                            onDone()
                        }
                    }
                }) {
                    Text("Save Changes")
                }
            }
        }

        HorizontalDivider()

        Button(
            onClick = {
                scope.launch {
                    if (chatState.client.deleteChannel(channel.id)) {
                        onDone()
                    }
                }
            },
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.errorContainer,
                contentColor = MaterialTheme.colorScheme.onErrorContainer
            ),
            modifier = Modifier.fillMaxWidth()
        ) {
            Icon(Icons.Default.Delete, null)
            Spacer(Modifier.width(8.dp))
            Text("Delete Channel")
        }
    }
}
