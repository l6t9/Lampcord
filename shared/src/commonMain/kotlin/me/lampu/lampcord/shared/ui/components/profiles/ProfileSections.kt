package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.animation.core.*
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Instant

@Composable
private fun ProfileSectionHeader(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.Normal,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.padding(bottom = 8.dp)
    )
}

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
    val uriHandler = LocalUriHandler.current

    val profileTextColor = MaterialTheme.colorScheme.onSurface
    val profileSecondaryTextColor = MaterialTheme.colorScheme.onSurfaceVariant

    Column(modifier = Modifier.padding(bottom = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        // Bio Priority: Guild Member Bio -> User Profile Bio -> Base User Bio
        val bio = (guildMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: userMeta?.bio?.takeIf { it.isNotBlank() } 
            ?: user.bio?.takeIf { it.isNotBlank() })?.let { Profile3y3.strip(it) }
            
        if (!bio.isNullOrBlank()) {
            Column {
                ProfileSectionHeader("About Me")
                DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = profileTextColor)
            }
        }

        // Dates
        if (isExpanded || showMemberSince) {
            Column {
                ProfileSectionHeader("Member Since")
                Row(verticalAlignment = Alignment.CenterVertically) {
                    // Discord Join Date
                    Surface(modifier = Modifier.size(16.dp), color = MaterialTheme.colorScheme.primary.copy(alpha = 0.2f), shape = CircleShape) {
                        Icon(Icons.Brand.Discord, null, modifier = Modifier.padding(2.dp), tint = MaterialTheme.colorScheme.primary)
                    }
                    Spacer(Modifier.width(8.dp))
                    Text(formatDate(user.id), style = MaterialTheme.typography.bodyMedium, color = profileTextColor)

                    // Guild Join Date
                    profile.guild_member?.joined_at?.let { joinedAt ->
                        Text(" • ", style = MaterialTheme.typography.bodyMedium, color = profileSecondaryTextColor)
                        val guildIcon = guild?.let { g ->
                            if (g.icon != null) "https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png?size=32" else null
                        }
                        if (guildIcon != null) {
                            AsyncImage(model = guildIcon, contentDescription = null, modifier = Modifier.size(16.dp).clip(CircleShape))
                        } else {
                            Surface(modifier = Modifier.size(16.dp), color = MaterialTheme.colorScheme.secondary.copy(alpha = 0.2f), shape = CircleShape) {}
                        }
                        Spacer(Modifier.width(8.dp))
                        Text(formatJoinDate(joinedAt), style = MaterialTheme.typography.bodyMedium, color = profileTextColor)
                    }
                }
            }
        }

        val presences by presenceStore.presences.collectAsState()
        val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]
        val activities = (profile.activities.ifEmpty { presence?.activities ?: emptyList() }).filter { it.type != 4 }

        if (activities.isNotEmpty()) {
            Column {
                ProfileSectionHeader("Activity")
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    activities.forEach { activity ->
                        UserActivity(
                            activity = activity,
                            compact = false
                        )
                    }
                }
            }
        }

        // Music / Last.fm Placeholder (if not already handled by Spotify activity)
        if (isExpanded) {
            // Check if user has a connected account that looks like Last.fm or similar
            val musicAccount = profile.connected_accounts.find { it.type == "lastfm" || it.type == "spotify" }
            if (musicAccount != null && activities.none { it.type == 2 }) {
                Column {
                    ProfileSectionHeader("Music Stats")
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        color = theme.cardColor.copy(alpha = 0.3f),
                    ) {
                        Row(
                            modifier = Modifier.padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(Icons.Filled.MusicNote, null, modifier = Modifier.size(24.dp), tint = profileTextColor)
                            Spacer(Modifier.width(12.dp))
                            Column {
                                Text("Recently Played", style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold, color = profileTextColor)
                                Text("Connect Last.fm for detailed stats", style = MaterialTheme.typography.bodySmall, color = profileSecondaryTextColor)
                            }
                        }
                    }
                }
            }
        }

        if (isExpanded) {
            // Roles
            val roles = profile.guild_member?.roles
            if (!roles.isNullOrEmpty() && guild != null) {
                Column {
                    ProfileSectionHeader("Roles")
                    androidx.compose.foundation.layout.FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        roles.mapNotNull { id -> guild.roles.find { it.id == id } }
                            .sortedByDescending { it.position }
                            .forEach { role ->
                                RoleBadge(role)
                            }
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
                    var showManageRoles by remember(profile.user.id, guild.id) { mutableStateOf(false) }
                    Button(
                        onClick = { showManageRoles = true },
                        modifier = Modifier.fillMaxWidth().height(40.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = theme.buttonColor,
                            contentColor = theme.buttonTextColor
                        ),
                        shape = CircleShape,
                        contentPadding = PaddingValues(horizontal = 12.dp)
                    ) {
                        Text("Manage User", style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Medium)
                    }
                    if (showManageRoles) {
                        ManageRolesSheet(profile = profile, guild = guild, onDismiss = { showManageRoles = false })
                    }
                }
            }

            // Connections
            if (profile.connected_accounts.isNotEmpty()) {
                Column {
                    ProfileSectionHeader("Connections")
                    Material3SettingsGroup(
                        horizontalPadding = 0.dp,
                        items = profile.connected_accounts.map { account ->
                            Material3SettingsItem(
                                icon = getConnectionIcon(account.type, account.name),
                                iconTint = profileTextColor,
                                containerColor = theme.cardColor.copy(alpha = 0.3f),
                                title = { Text(account.name, color = profileTextColor) },
                                trailingContent = if (account.verified) {
                                    {
                                        Icon(
                                            Icons.AutoMirrored.Filled.OpenInNew,
                                            contentDescription = "Open Link",
                                            modifier = Modifier.size(16.dp),
                                            tint = profileSecondaryTextColor
                                        )
                                    }
                                } else null,
                                onClick = {
                                    val url = getConnectionUrl(account.type, account.name, account.id)
                                    if (url != null) {
                                        uriHandler.openUri(url)
                                    } else {
                                        setClipboardText(account.name)
                                        showToast("Copied ${account.name} to clipboard")
                                    }
                                }
                            )
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun RoleBadge(role: me.lampu.lampcord.shared.model.Role) {
    val infiniteTransition = rememberInfiniteTransition(label = "roleGradient")
    val animValue by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(4000, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "gradientOffset"
    )

    Surface(
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(8.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f))
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val roleColors = remember<List<Color>?>(role) {
                val c = role.colors
                if (c?.secondary_color != null) {
                    val p = c.primary_color ?: role.color
                    listOfNotNull(
                        Color(p or -0x1000000),
                        Color(c.secondary_color or -0x1000000),
                        c.tertiary_color?.let { Color(it or -0x1000000) }
                    )
                } else null
            }

            val pInt = role.colors?.primary_color ?: role.color
            val baseColor = if (pInt != 0) Color(pInt or -0x1000000) else MaterialTheme.colorScheme.primary
            
            val brush = remember(roleColors, animValue) {
                val rc = roleColors
                if (rc != null && rc.size > 1) {
                    val animatedColors = when {
                        rc.size >= 3 -> listOf(rc[0], rc[1], rc[2], rc[0])
                        rc.size == 2 -> listOf(rc[0], rc[1], rc[0])
                        else -> rc
                    }
                    
                    val offset = animValue * 100f
                    Brush.linearGradient(
                        colors = animatedColors,
                        start = Offset(offset - 100f, 0f),
                        end = Offset(offset, 0f),
                        tileMode = TileMode.Repeated
                    )
                } else null
            }

            Box(
                modifier = Modifier
                    .size(12.dp)
                    .clip(CircleShape)
                    .then(
                        if (brush != null) Modifier.background(brush)
                        else Modifier.background(baseColor)
                    )
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = role.name,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurface,
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
