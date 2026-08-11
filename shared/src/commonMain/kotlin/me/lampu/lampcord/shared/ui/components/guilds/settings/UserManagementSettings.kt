package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Ban
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.components.settings.SettingsSection
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ServerMembers(guild: Guild, discordClient: DiscordClient = koinInject()) {
    var members by remember { mutableStateOf<List<Member>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    
    LaunchedEffect(guild.id) {
        isLoading = true
        members = discordClient.searchGuildMembers(guild.id)
        isLoading = false
    }

    SettingsLayout {
        SettingsSection(title = "Members", icon = Icons.Filled.Group) {
            if (isLoading) {
                ContainedLoadingIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
            } else {
                members.forEach { member ->
                    val user = member.user ?: return@forEach
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(min = 56.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                            .padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = null,
                            modifier = Modifier.size(32.dp).clip(CircleShape)
                        )
                        Column(modifier = Modifier.weight(1f)) {
                            Text(member.nick ?: user.global_name ?: user.username ?: "Unknown", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                            Text(user.username ?: "", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        
                        // Role chips
                        val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }.sortedByDescending { it.position }
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            memberRoles.take(3).forEach { role ->
                                Box(
                                    modifier = Modifier
                                        .clip(CircleShape)
                                        .background(if (role.color != 0) Color(role.color.toLong() or 0xFF000000L).copy(alpha = 0.2f) else MaterialTheme.colorScheme.surfaceVariant)
                                        .border(1.dp, if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.outline, CircleShape)
                                        .padding(horizontal = 8.dp, vertical = 2.dp)
                                ) {
                                    Text(role.name, style = MaterialTheme.typography.labelSmall, color = if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurface)
                                }
                            }
                            if (memberRoles.size > 3) {
                                Text("+${memberRoles.size - 3}", style = MaterialTheme.typography.labelSmall)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ServerInvites(guild: Guild, discordClient: DiscordClient = koinInject()) {
    var invites by remember { mutableStateOf<List<Invite>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    val scope = rememberCoroutineScope()
    
    LaunchedEffect(guild.id) {
        isLoading = true
        invites = discordClient.getGuildInvites(guild.id)
        isLoading = false
    }

    SettingsLayout {
        SettingsSection(title = "Invites", icon = Icons.Filled.Link) {
            if (isLoading) {
                ContainedLoadingIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
            } else if (invites.isEmpty()) {
                Text("No active invites", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                invites.forEach { invite ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(min = 56.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                            .padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(invite.code, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                            Text("${invite.uses ?: 0} uses • Exp: ${invite.expires_at ?: "Never"}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconButton(onClick = {
                            scope.launch {
                                if (discordClient.deleteInvite(invite.code)) {
                                    invites = invites.filter { it.code != invite.code }
                                }
                            }
                        }) {
                            Icon(Icons.Default.Delete, "Revoke", tint = MaterialTheme.colorScheme.error)
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ServerBans(guild: Guild, discordClient: DiscordClient = koinInject()) {
    var bans by remember { mutableStateOf<List<Ban>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    val scope = rememberCoroutineScope()
    
    LaunchedEffect(guild.id) {
        isLoading = true
        bans = discordClient.getGuildBans(guild.id)
        isLoading = false
    }

    SettingsLayout {
        SettingsSection(title = "Bans", icon = Icons.Filled.Block) {
            if (isLoading) {
                ContainedLoadingIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
            } else if (bans.isEmpty()) {
                Text("No banned users", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                bans.forEach { ban ->
                    val user = ban.user
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(min = 56.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                            .padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = null,
                            modifier = Modifier.size(32.dp).clip(CircleShape)
                        )
                        Column(modifier = Modifier.weight(1f)) {
                            Text(user.global_name ?: user.username ?: "Unknown", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                            Text(ban.reason ?: "No reason provided", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Button(
                            onClick = {
                                scope.launch {
                                    if (discordClient.unbanUser(guild.id, user.id)) {
                                        bans = bans.filter { it.user.id != user.id }
                                    }
                                }
                            },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.errorContainer,
                                contentColor = MaterialTheme.colorScheme.onErrorContainer
                            )
                        ) {
                            Text("Unban")
                        }
                    }
                }
            }
        }
    }
}
