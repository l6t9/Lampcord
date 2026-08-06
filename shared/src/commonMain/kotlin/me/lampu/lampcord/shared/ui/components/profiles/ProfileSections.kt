package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.*
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
        val bio = (guildMeta?.bio ?: userMeta?.bio ?: user.bio)
        if (!bio.isNullOrBlank()) {
            Spacer(Modifier.height(12.dp))
            Text("About Me", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
            Spacer(Modifier.height(4.dp))
            DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor, chatState = chatState)
        }

        if (isExpanded || showMemberSince) {
            val creationDate = remember(user.id) { 
                val timestamp = (user.id.toLong() shr 22) + 1420070400000L
                me.lampu.lampcord.shared.utils.DateTimeUtils.formatDiscordTimestamp(timestamp / 1000, "D")
            }
            val joinDate = profile.guild_member?.joined_at?.let { 
                if (it.isBlank()) null else me.lampu.lampcord.shared.utils.DateTimeUtils.formatTimestamp(it).substringBefore(",")
            }

            Spacer(Modifier.height(12.dp))
            Column(modifier = Modifier.fillMaxWidth()) {
                Text("Member Since", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
                Spacer(Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Brand.Discord, null, modifier = Modifier.size(16.dp), tint = theme.contentColor.copy(alpha = 0.7f))
                    Spacer(Modifier.width(6.dp))
                    Text(creationDate, style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                    
                    if (joinDate != null) {
                        Text(" • ", style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                        val guildIcon = chatState.selectedGuild?.let { guild ->
                            if (guild.icon != null) "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=32" else null
                        }
                        if (guildIcon != null) {
                            AsyncImage(model = guildIcon, contentDescription = null, modifier = Modifier.size(16.dp).clip(androidx.compose.foundation.shape.CircleShape))
                            Spacer(Modifier.width(6.dp))
                        }
                        Text(joinDate, style = MaterialTheme.typography.bodySmall, color = theme.contentColor.copy(alpha = 0.7f))
                    }
                }
            }
        }

        val presence = profile.guild_member?.presence ?: chatState.presences[user.id]
        val activities = presence?.activities ?: emptyList()
        val otherActivities = activities.filter { it.type != 4 }

        otherActivities.forEach { activity ->
            Spacer(Modifier.height(16.dp))
            UserActivity(activity, compact = false)
        }

        if (profile.guild_member?.roles?.isNotEmpty() == true) {
            Spacer(Modifier.height(16.dp))
            Text("Roles", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.9f))
            Spacer(Modifier.height(8.dp))
            androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                val guild = chatState.selectedGuild
                profile.guild_member.roles.mapNotNull { id -> guild?.roles?.find { it.id == id } }.sortedByDescending { it.position }.forEach { role ->
                    RoleTag(role.name, if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else theme.contentColor)
                }
            }
        }

        if (isExpanded && profile.connected_accounts.isNotEmpty()) {
            Spacer(Modifier.height(16.dp))
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
