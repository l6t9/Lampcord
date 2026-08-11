package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import org.koin.compose.koinInject
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Instant

@Composable
fun ProfileSections(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean,
    showMemberSince: Boolean = false,
    navigationStore: NavigationStore = koinInject()
) {
    val user = profile.user
    val userMeta = profile.user_profile
    val guildMeta = profile.guild_member_profile

    Column(modifier = Modifier.padding(vertical = 12.dp)) {
        // Bio
        val bio = guildMeta?.bio ?: userMeta?.bio ?: user.bio
        if (!bio.isNullOrBlank()) {
            Text("ABOUT ME", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold, color = theme.contentColor.copy(alpha = 0.8f))
            Spacer(Modifier.height(4.dp))
            DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor)
            Spacer(Modifier.height(16.dp))
        }

        // Dates
        if (isExpanded || showMemberSince) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                // Discord Join Date
                Column {
                    Text("DISCORD MEMBER SINCE", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold, color = theme.contentColor.copy(alpha = 0.8f))
                    Spacer(Modifier.height(4.dp))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Surface(modifier = Modifier.size(16.dp), color = MaterialTheme.colorScheme.primary.copy(alpha = 0.2f), shape = CircleShape) {
                            Icon(me.lampu.lampcord.shared.ui.icons.Icons.Brand.Discord, null, modifier = Modifier.padding(2.dp), tint = MaterialTheme.colorScheme.primary)
                        }
                        Spacer(Modifier.width(8.dp))
                        Text(formatDate(user.id), style = MaterialTheme.typography.bodyMedium, color = theme.contentColor)
                    }
                }

                // Guild Join Date
                profile.guild_member?.joined_at?.let { joinedAt ->
                    Column {
                        val guild = navigationStore.selectedGuild
                        Text("${guild?.name?.uppercase() ?: "SERVER"} MEMBER SINCE", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold, color = theme.contentColor.copy(alpha = 0.8f))
                        Spacer(Modifier.height(4.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            val guildIcon = guild?.let { g ->
                                if (g.icon != null) "https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png?size=32" else null
                            }
                            if (guildIcon != null) {
                                AsyncImage(model = guildIcon, contentDescription = null, modifier = Modifier.size(16.dp).clip(CircleShape))
                            } else {
                                Surface(modifier = Modifier.size(16.dp), color = MaterialTheme.colorScheme.secondary.copy(alpha = 0.2f), shape = CircleShape) {}
                            }
                            Spacer(Modifier.width(8.dp))
                            Text(formatJoinDate(joinedAt), style = MaterialTheme.typography.bodyMedium, color = theme.contentColor)
                        }
                    }
                }
            }
            Spacer(Modifier.height(16.dp))
        }

        if (isExpanded) {
            // Roles
            val roles = profile.guild_member?.roles
            if (!roles.isNullOrEmpty()) {
                val guild = navigationStore.selectedGuild
                if (guild != null) {
                    Text("ROLES", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold, color = theme.contentColor.copy(alpha = 0.8f))
                    Spacer(Modifier.height(8.dp))
                    androidx.compose.foundation.layout.FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        roles.mapNotNull { id -> guild.roles.find { it.id == id } }
                            .sortedByDescending { it.position }
                            .forEach { role ->
                                RoleBadge(role, theme)
                            }
                    }
                }
            }
        }
    }
}

@Composable
private fun RoleBadge(role: me.lampu.lampcord.shared.model.Role, theme: ProfileTheme) {
    Surface(
        color = theme.cardColor.copy(alpha = 0.3f),
        shape = RoundedCornerShape(4.dp),
        border = BorderStroke(1.dp, theme.contentColor.copy(alpha = 0.1f))
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val roleColor = if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else theme.contentColor
            Box(modifier = Modifier.size(12.dp).background(roleColor, CircleShape))
            Spacer(Modifier.width(8.dp))
            Text(role.name, style = MaterialTheme.typography.labelMedium, color = theme.contentColor)
        }
    }
}

private fun formatDate(userId: String): String {
    val timestamp = (userId.toLong() shr 22) + 1420070400000L
    val instant = Instant.fromEpochMilliseconds(timestamp)
    val date = instant.toLocalDateTime(TimeZone.currentSystemDefault())
    return "${date.month.name.lowercase().capitalize()} ${date.day}, ${date.year}"
}

private fun formatJoinDate(iso: String): String {
    try {
        val instant = Instant.parse(iso)
        val date = instant.toLocalDateTime(TimeZone.currentSystemDefault())
        return "${date.month.name.lowercase().capitalize()} ${date.day}, ${date.year}"
    } catch (e: Exception) {
        return iso
    }
}

private fun String.capitalize() = replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
