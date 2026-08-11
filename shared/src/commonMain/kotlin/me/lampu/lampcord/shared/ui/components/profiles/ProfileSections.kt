package me.lampu.lampcord.shared.ui.components.profiles

import kotlinx.datetime.Instant

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun ProfileSections(
    profile: UserProfile,
    chatState: ChatState,
    theme: ProfileTheme,
    isExpanded: Boolean,
    showMemberSince: Boolean = false
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile

    Column(modifier = Modifier.offset(y = if (isExpanded) (-50).dp else (-35).dp)) {
        val bio = guildMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: userMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: user.bio?.takeIf { it.isNotBlank() }
            
        if (!bio.isNullOrBlank()) {
            Column(modifier = Modifier.padding(vertical = 8.dp)) {
                Text("About Me", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                Spacer(Modifier.height(8.dp))
                DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor, chatState = chatState)
            }
        }

        if (isExpanded || showMemberSince) {
            val creationDate = remember(user.id) { 
                val timestamp = (user.id.toLong() shr 22) + 1420070400000L
                me.lampu.lampcord.shared.utils.DateTimeUtils.formatDiscordTimestamp(timestamp / 1000, "D")
            }
            val joinDate = profile.guild_member?.joined_at?.let { 
                if (it.isBlank()) null else {
                    try {
                        val instant = Instant.parse(it)
                        me.lampu.lampcord.shared.utils.DateTimeUtils.formatDiscordTimestamp(instant.epochSeconds, "D")
                    } catch (_: Exception) {
                        null
                    }
                }
            }

            Column(modifier = Modifier.padding(vertical = 8.dp)) {
                Text("Member Since", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                Spacer(Modifier.height(8.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Brand.Discord, null, modifier = Modifier.size(16.dp), tint = theme.contentColor.copy(alpha = 0.7f))
                    Spacer(Modifier.width(8.dp))
                    Text(creationDate, style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                    
                    if (joinDate != null) {
                        Text(" • ", style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                        val guildIcon = chatState.selectedGuild?.let { guild ->
                            if (guild.icon != null) "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=32" else null
                        }
                        if (guildIcon != null) {
                            AsyncImage(model = guildIcon, contentDescription = null, modifier = Modifier.size(16.dp).clip(androidx.compose.foundation.shape.CircleShape))
                            Spacer(Modifier.width(8.dp))
                        }
                        Text(joinDate, style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                    }
                }
            }
        }

        if (profile.guild_member?.roles?.isNotEmpty() == true) {
            Column(modifier = Modifier.padding(vertical = 8.dp)) {
                Text("Roles", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                Spacer(Modifier.height(8.dp))
                androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    val guild = chatState.selectedGuild
                    profile.guild_member.roles.mapNotNull { id -> guild?.roles?.find { it.id == id } }.sortedByDescending { it.position }.forEach { role ->
                        val roleColor = if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else null
                        RoleTag(role.name, theme.tagColor, theme.contentColor, roleColor)
                    }
                }
            }
        }

        if (me.lampu.lampcord.shared.settings.Settings.shared.showPermissions) {
            val guild = chatState.selectedGuild
            if (guild != null && profile.guild_member != null) {
                val perms = me.lampu.lampcord.shared.utils.PermissionHelper.computeBasePermissions(profile.guild_member, guild, profile.user.id)
                if (perms != 0L) {
                    val allowedPerms = me.lampu.lampcord.shared.utils.Permission.fromValue(perms)
                    if (allowedPerms.isNotEmpty()) {
                        Column(modifier = Modifier.padding(vertical = 8.dp)) {
                            Text("Permissions", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                            Spacer(Modifier.height(8.dp))
                            androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                allowedPerms.forEach { perm ->
                                    val label = perm.name.lowercase().replace('_', ' ').replaceFirstChar { it.uppercase() }
                                    RoleTag(label, theme.tagColor, theme.contentColor)
                                }
                            }
                        }
                    }
                }
            }
        }

        if (isExpanded && profile.connected_accounts.isNotEmpty()) {
            Column(modifier = Modifier.padding(vertical = 8.dp)) {
                Text("Connections", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                Spacer(Modifier.height(8.dp))
                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    profile.connected_accounts.forEachIndexed { index, connection ->
                        UserConnectionItem(
                            connection = connection,
                            contentColor = theme.contentColor,
                            isFirst = index == 0,
                            isLast = index == profile.connected_accounts.lastIndex
                        )
                    }
                }
            }
        }
    }
}

