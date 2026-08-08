package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

private enum class ServerSettingsSection(val title: String, val icon: ImageVector) {
    OVERVIEW("Overview", Icons.Filled.Info),
    ROLES("Roles", Icons.Filled.Security),
    EMOJI("Emoji", Icons.Default.AddReaction),
    STICKERS("Stickers", Icons.Filled.Tune),
    MEMBERS("Members", Icons.Filled.Group),
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerSettings(chatState: ChatState, onDismiss: () -> Unit) {
    val guild = chatState.selectedGuild ?: return
    var selectedCategory by remember { mutableStateOf(ServerSettingsSection.OVERVIEW) }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp

        if (isCompact) {
            SettingsSubScreen(
                title = selectedCategory.title,
                onNavigateBack = onDismiss,
            ) {
                ServerSettingsContent(selectedCategory, guild, chatState)
            }
        } else {
            // Desktop Layout
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                Card(
                    modifier = Modifier
                        .padding(32.dp)
                        .widthIn(max = 1080.dp)
                        .fillMaxWidth()
                        .heightIn(max = 720.dp)
                        .fillMaxHeight(),
                    shape = RoundedCornerShape(24.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                    elevation = CardDefaults.cardElevation(defaultElevation = 12.dp)
                ) {
                    Row(modifier = Modifier.fillMaxSize()) {
                        // Sidebar
                        Surface(
                            modifier = Modifier.width(280.dp).fillMaxHeight(),
                            color = MaterialTheme.colorScheme.surfaceContainerLowest
                        ) {
                            Column(modifier = Modifier.padding(16.dp)) {
                                Text(
                                    guild.name ?: "Server Settings",
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold,
                                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp)
                                )

                                Spacer(modifier = Modifier.height(16.dp))

                                LazyColumn(
                                    modifier = Modifier.fillMaxSize(),
                                    verticalArrangement = Arrangement.spacedBy(4.dp)
                                ) {
                                    items(ServerSettingsSection.entries) { section ->
                                        val isSelected = selectedCategory == section
                                        Row(
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .clip(RoundedCornerShape(12.dp))
                                                .selectable(
                                                    selected = isSelected,
                                                    role = Role.Tab,
                                                    onClick = { selectedCategory = section },
                                                ).padding(horizontal = 8.dp, vertical = 8.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                                        ) {
                                            Icon(
                                                section.icon,
                                                contentDescription = section.title,
                                                tint = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                                                modifier = Modifier.size(24.dp)
                                            )
                                            Text(
                                                section.title,
                                                style = MaterialTheme.typography.bodyMedium,
                                                fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                                                color = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
                                            )
                                        }
                                    }
                                }
                            }
                        }

                        Box(modifier = Modifier.fillMaxHeight().width(1.dp).background(MaterialTheme.colorScheme.outlineVariant))

                        // Content
                        Box(modifier = Modifier.weight(1f).fillMaxHeight()) {
                            val scrollState = rememberScrollState()
                            Column(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .verticalScroll(scrollState)
                                    .padding(24.dp),
                                verticalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                ServerSettingsContent(selectedCategory, guild, chatState)
                            }

                            IconButton(
                                onClick = onDismiss,
                                modifier = Modifier.align(Alignment.TopEnd).padding(8.dp).size(36.dp),
                            ) {
                                Icon(Icons.Default.Close, contentDescription = "Close", modifier = Modifier.size(20.dp))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ServerSettingsContent(section: ServerSettingsSection, guild: Guild, chatState: ChatState) {
    when (section) {
        ServerSettingsSection.OVERVIEW -> ServerOverview(guild, chatState)
        ServerSettingsSection.ROLES -> ServerRoles(guild, chatState)
        ServerSettingsSection.EMOJI -> ServerEmoji(guild, chatState)
        ServerSettingsSection.STICKERS -> ServerStickers(guild, chatState)
        ServerSettingsSection.MEMBERS -> ServerMembers(guild, chatState)
    }
}

@Composable
private fun ServerOverview(guild: Guild, chatState: ChatState) {
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Material3SettingsGroup(title = "Server Details") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Server Name") },
                        description = { Text(guild.name ?: "") },
                        onClick = { /* TODO: edit name */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Server ID") },
                        description = { Text(guild.id) },
                        onClick = { /* TODO: copy ID */ }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Security") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Verification Level") },
                        description = { 
                            Text(when(guild.verification_level) {
                                0 -> "None"
                                1 -> "Low"
                                2 -> "Medium"
                                3 -> "High"
                                4 -> "Highest"
                                else -> "None"
                            })
                        },
                        onClick = { /* TODO: update verification level */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Explicit Content Filter") },
                        description = {
                            Text(when(guild.explicit_content_filter) {
                                0 -> "Don't scan any messages"
                                1 -> "Scan messages from members without a role"
                                2 -> "Scan messages from all members"
                                else -> "Don't scan"
                            })
                        },
                        onClick = { /* TODO: update filter */ }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "System Messages") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("System Messages Channel") },
                        description = { Text(chatState.channels.find { it.id == guild.system_channel_id }?.name ?: "No system messages channel") },
                        onClick = { /* TODO */ }
                    )
                )
            )
        }
    }
}

@Composable
private fun ServerRoles(guild: Guild, chatState: ChatState) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        guild.roles.sortedByDescending { it.position }.forEach { role ->
            Material3SettingsItemRow(
                item = Material3SettingsItem(
                    leadingContent = {
                        Box(
                            modifier = Modifier
                                .size(12.dp)
                                .background(if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant, CircleShape)
                        )
                    },
                    title = { Text(role.name) },
                    trailingContent = {
                        if (role.managed) {
                            Icon(Icons.Filled.Security, null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    },
                    onClick = { /* TODO: Edit Role */ }
                ),
                isFirst = true,
                isLast = true
            )
        }
    }
}

@Composable
private fun ServerEmoji(guild: Guild, chatState: ChatState) {
    if (guild.emojis.isEmpty()) {
        Text("No custom emojis", color = MaterialTheme.colorScheme.onSurfaceVariant)
    } else {
        FlowRow(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            guild.emojis.forEach { emoji ->
                val url = "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=96"
                me.lampu.lampcord.shared.ui.components.AsyncImage(
                    model = url,
                    contentDescription = emoji.name,
                    modifier = Modifier.size(48.dp)
                )
            }
        }
    }
}

@Composable
private fun ServerStickers(guild: Guild, chatState: ChatState) {
    Text("Sticker management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
}

@Composable
private fun ServerMembers(guild: Guild, chatState: ChatState) {
    Text("Member management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
}
