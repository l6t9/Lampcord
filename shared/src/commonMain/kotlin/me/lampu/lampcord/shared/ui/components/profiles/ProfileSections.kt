package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
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
    guildStore: GuildStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val user = profile.user
    val userMeta = profile.user_profile
    val guildMeta = profile.guild_member_profile
    val guilds by guildStore.guilds.collectAsState()
    val guild = profile.guild_id?.let { gid -> guilds.find { it.id == gid } }
    val currentUser by userStore.currentUser.collectAsState()

    Column(modifier = Modifier.padding(bottom = 12.dp)) {
        // Bio Priority: Guild Member Bio -> User Profile Bio -> Base User Bio
        val bio = guildMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: userMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: user.bio?.takeIf { it.isNotBlank() }
            
        if (!bio.isNullOrBlank()) {
            Text("About Me", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.8f))
            Spacer(Modifier.height(4.dp))
            DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor)
            Spacer(Modifier.height(16.dp))
        }

        // Dates
        if (isExpanded || showMemberSince) {
            Column {
                Text("Member Since", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.8f))
                Spacer(Modifier.height(8.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    // Discord Join Date
                    Surface(modifier = Modifier.size(16.dp), color = MaterialTheme.colorScheme.primary.copy(alpha = 0.2f), shape = CircleShape) {
                        Icon(me.lampu.lampcord.shared.ui.icons.Icons.Brand.Discord, null, modifier = Modifier.padding(2.dp), tint = MaterialTheme.colorScheme.primary)
                    }
                    Spacer(Modifier.width(8.dp))
                    Text(formatDate(user.id), style = MaterialTheme.typography.bodyMedium, color = theme.contentColor)

                    // Guild Join Date
                    profile.guild_member?.joined_at?.let { joinedAt ->
                        Text(" • ", style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.5f))
                        val guilds by guildStore.guilds.collectAsState()
                        val guild = profile.guild_id?.let { gid -> guilds.find { it.id == gid } }
                        
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
            Spacer(Modifier.height(16.dp))
        }

        val presences by presenceStore.presences.collectAsState()
        val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]
        val activities = (profile.activities.ifEmpty { presence?.activities ?: emptyList() }).filter { it.type != 4 }

        if (activities.isNotEmpty()) {
            Text("Activity", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.8f))
            Spacer(Modifier.height(8.dp))
            activities.forEach { activity ->
                UserActivity(
                    activity = activity,
                    compact = false
                )
                Spacer(Modifier.height(16.dp))
            }
        }

        if (isExpanded) {
            // Roles
            val roles = profile.guild_member?.roles
            if (!roles.isNullOrEmpty() && guild != null) {
                Text("Roles", style = MaterialTheme.typography.labelSmall, color = theme.contentColor.copy(alpha = 0.8f))
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

            // Manage User (add/remove roles)
            if (guild != null && profile.guild_member != null && profile.user.id != currentUser?.id) {
                val me = currentUser
                val myMember = remember(guild.id, me) {
                    me?.id?.let { userStore.getMember(guild.id, it) }
                }
                val canManage = me != null && PermissionHelper.hasPermission(
                    myMember ?: Member(user = me),
                    guild,
                    null,
                    Permission.MANAGE_ROLES,
                    me.id
                )
                if (canManage) {
                    Spacer(Modifier.height(16.dp))
                    var showManageRoles by remember(profile.user.id, guild.id) { mutableStateOf(false) }
                    Button(
                        onClick = { showManageRoles = true },
                        modifier = Modifier.fillMaxWidth().height(36.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = theme.buttonColor,
                            contentColor = theme.buttonTextColor
                        ),
                        shape = CircleShape,
                        contentPadding = PaddingValues(horizontal = 12.dp)
                    ) {
                        Text("Manage User", style = MaterialTheme.typography.labelLarge, fontWeight = androidx.compose.ui.text.font.FontWeight.Medium)
                    }
                    if (showManageRoles) {
                        ManageRolesSheet(profile = profile, guild = guild, onDismiss = { showManageRoles = false })
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
            Text(
                text = role.name,
                style = MaterialTheme.typography.labelMedium,
                color = theme.contentColor,
                maxLines = 1,
                modifier = Modifier.basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp)
            )
        }
    }
}

private fun formatDate(userId: String): String {
    val timestamp = (userId.toLong() shr 22) + 1420070400000L
    val instant = Instant.fromEpochMilliseconds(timestamp)
    val date = instant.toLocalDateTime(TimeZone.currentSystemDefault())
    val month = date.month.name.lowercase().take(3).replaceFirstChar { it.uppercase() }
    return "$month ${date.day}, ${date.year}"
}

private fun formatJoinDate(iso: String): String {
    try {
        val instant = Instant.parse(iso)
        val date = instant.toLocalDateTime(TimeZone.currentSystemDefault())
        val month = date.month.name.lowercase().take(3).replaceFirstChar { it.uppercase() }
        return "$month ${date.day}, ${date.year}"
    } catch (e: Exception) {
        return iso
    }
}
