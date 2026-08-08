package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.DiscordGreen

@Composable
fun AutocompletePicker(
    chatState: ChatState,
    type: AutocompleteType,
    query: String,
    modifier: Modifier = Modifier,
    onItemSelected: (String, String) -> Unit // (label, replacement)
) {
    val items = when (type) {
        AutocompleteType.MENTION -> {
            val guildId = chatState.selectedGuild?.id
            val results = mutableListOf<AutocompleteItem>()
            
            if (query.isEmpty() || "everyone".contains(query, ignoreCase = true)) {
                results.add(AutocompleteItem(id = "everyone", title = "everyone", replacement = "@everyone", iconType = Icons.Filled.Group))
            }
            if (query.isEmpty() || "here".contains(query, ignoreCase = true)) {
                results.add(AutocompleteItem(id = "here", title = "here", replacement = "@here", iconType = Icons.Filled.Group))
            }

            val members = if (guildId != null) {
                 chatState.memberListItems.filterNotNull().mapNotNull { it.member }.filter {
                     val name = it.nick ?: it.user?.global_name ?: it.user?.username ?: ""
                     name.contains(query, ignoreCase = true) || it.user?.username?.contains(query, ignoreCase = true) == true
                 }.take(10)
            } else {
                chatState.relationships.filter { 
                    it.user?.global_name?.contains(query, ignoreCase = true) == true || 
                    it.user?.username?.contains(query, ignoreCase = true) == true
                }.mapNotNull { it.user?.let { u -> Member(user = u) } }.take(10)
            }
            
            results.addAll(members.map { member ->
                val user = member.user!!
                val name = member.nick ?: user.global_name ?: user.username ?: "Unknown User"
                AutocompleteItem(
                    id = user.id,
                    title = name,
                    subtitle = user.username,
                    icon = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" },
                    replacement = "<@${user.id}>"
                )
            })

            val roles = chatState.selectedGuild?.roles?.filter { 
                it.name.contains(query, ignoreCase = true) 
            }?.take(5) ?: emptyList()
            results.addAll(roles.map { role ->
                AutocompleteItem(
                    id = role.id,
                    title = role.name,
                    iconType = Icons.Filled.Group,
                    replacement = "<@&${role.id}>",
                    color = if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else null
                )
            })

            results.take(15)
        }
        AutocompleteType.CHANNEL -> {
            val channels = chatState.channels.filter { 
                it.type in listOf(0, 2, 4, 5, 13, 15, 16) && it.name?.contains(query, ignoreCase = true) == true 
            }.take(10)
            channels.map { channel ->
                AutocompleteItem(
                    id = channel.id,
                    title = channel.name ?: "unnamed",
                    iconType = when (channel.type) {
                        4 -> Icons.Filled.Folder
                        15 -> Icons.Outlined.Forum
                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                        5 -> Icons.Filled.Campaign
                        else -> Icons.Filled.Tag
                    },
                    replacement = if (channel.type == 4) channel.name ?: "" else "<#${channel.id}>"
                )
            }
        }
        AutocompleteType.ROLE -> emptyList()
        AutocompleteType.COMMAND -> {
            val commands = chatState.availableCommands.filter { 
                it.name.contains(query, ignoreCase = true) 
            }.take(10)
            commands.map { command ->
                val app = chatState.availableApplications.find { it.id == command.application_id }
                AutocompleteItem(
                    id = command.id,
                    title = "/${command.name}",
                    subtitle = command.description,
                    icon = app?.icon?.let { "https://cdn.discordapp.com/app-icons/${app.id}/$it.png?size=64" },
                    replacement = command.name,
                    isCommand = true,
                    commandObj = command
                )
            }
        }
        AutocompleteType.EMOJI -> {
            val emojis = chatState.selectedGuild?.emojis?.filter { 
                it.name?.contains(query, ignoreCase = true) == true 
            }?.take(10) ?: emptyList()
            emojis.map { emoji ->
                AutocompleteItem(
                    id = emoji.id ?: emoji.name ?: "",
                    title = ":${emoji.name}:",
                    icon = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=64" else null,
                    replacement = if (emoji.id != null) "<:${emoji.name}:${emoji.id}>" else ":${emoji.name}:"
                )
            }
        }
    }

    if (items.isEmpty()) return

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(max = 320.dp),
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        tonalElevation = 3.dp,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
    ) {
        Column {
            val headerTitle = when(type) {
                AutocompleteType.MENTION -> "Members & Roles"
                AutocompleteType.CHANNEL -> "Channels"
                AutocompleteType.COMMAND -> "Commands"
                AutocompleteType.EMOJI -> "Emojis"
                AutocompleteType.ROLE -> ""
            }
            
            Text(
                text = headerTitle,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
            )
            
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.1f))

            LazyColumn(
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(4.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                items(items) { item ->
                    Surface(
                        onClick = { 
                            if (item.isCommand && item.commandObj != null) {
                                // Specialized handling for commands might be needed in ChatInputBar
                                onItemSelected(item.title, item.replacement)
                            } else {
                                onItemSelected(item.title, item.replacement)
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        color = Color.Transparent
                    ) {
                        Row(
                            modifier = Modifier.padding(8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (item.icon != null) {
                                AsyncImage(
                                    model = item.icon,
                                    contentDescription = null,
                                    modifier = Modifier.size(24.dp).clip(if (type == AutocompleteType.MENTION) CircleShape else RoundedCornerShape(4.dp))
                                )
                            } else if (item.iconType != null) {
                                Icon(
                                    imageVector = item.iconType,
                                    contentDescription = null,
                                    modifier = Modifier.size(20.dp),
                                    tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            } else {
                                Box(
                                    modifier = Modifier.size(24.dp).background(MaterialTheme.colorScheme.surfaceVariant, CircleShape),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(item.title.take(1).uppercase(), style = MaterialTheme.typography.labelSmall)
                                }
                            }
                            
                            Spacer(Modifier.width(12.dp))
                            
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = item.title,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = item.color ?: MaterialTheme.colorScheme.onSurface,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                                if (item.subtitle != null) {
                                    Text(
                                        text = item.subtitle,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

data class AutocompleteItem(
    val id: String,
    val title: String,
    val subtitle: String? = null,
    val icon: String? = null,
    val iconType: androidx.compose.ui.graphics.vector.ImageVector? = null,
    val replacement: String,
    val isCommand: Boolean = false,
    val commandObj: ApplicationCommand? = null,
    val color: Color? = null
)
