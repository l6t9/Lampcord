package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
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
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Ban
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.RoleIcon
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ServerMembers(guild: Guild, guildApi: GuildApi = koinInject()) {
    var members by remember { mutableStateOf<List<Member>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    
    LaunchedEffect(guild.id) {
        isLoading = true
        members = guildApi.searchGuildMembers(guild.id)
        isLoading = false
    }

    SettingsLayout {
        Material3SettingsGroup(title = "Members") {
            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    members.forEach { member ->
                        val user = member.user ?: return@forEach
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f)
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                                AsyncImage(
                                    model = avatarUrl,
                                    contentDescription = null,
                                    modifier = Modifier.size(36.dp).clip(CircleShape)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(member.nick ?: user.global_name ?: user.username ?: "Unknown", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                    Text(user.username ?: "", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                
                                // Role chips
                                val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }.sortedByDescending { it.position }
                                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                    memberRoles.take(2).forEach { role ->
                                        Box(
                                            modifier = Modifier
                                                .clip(CircleShape)
                                                .background(if (role.color != 0) Color(role.color.toLong() or 0xFF000000L).copy(alpha = 0.15f) else MaterialTheme.colorScheme.surfaceVariant)
                                                .border(1.dp, if (role.color != 0) Color(role.color.toLong() or 0xFF000000L).copy(alpha = 0.5f) else MaterialTheme.colorScheme.outline.copy(alpha = 0.5f), CircleShape)
                                                .padding(horizontal = 8.dp, vertical = 2.dp)
                                        ) {
                                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                                RoleIcon(role, size = 10.dp)
                                                Text(role.name, style = MaterialTheme.typography.labelSmall, color = if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurface)
                                            }
                                        }
                                    }
                                    if (memberRoles.size > 2) {
                                        Text("+${memberRoles.size - 2}", style = MaterialTheme.typography.labelSmall, modifier = Modifier.align(Alignment.CenterVertically))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Spacer(modifier = Modifier.height(100.dp))
    }
}

@Composable
fun ServerInvites(guild: Guild, guildApi: GuildApi = koinInject()) {
    var invites by remember { mutableStateOf<List<Invite>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    val scope = rememberCoroutineScope()
    
    LaunchedEffect(guild.id) {
        isLoading = true
        invites = guildApi.getGuildInvites(guild.id)
        isLoading = false
    }

    SettingsLayout {
        Material3SettingsGroup(title = "Invites") {
            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else if (invites.isEmpty()) {
                Text("No active invites", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    invites.forEach { invite ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f)
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(invite.code, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary)
                                    Text("${invite.uses ?: 0} uses • Exp: ${invite.expires_at ?: "Never"}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                IconButton(onClick = {
                                    scope.launch {
                                        if (guildApi.deleteInvite(invite.code)) {
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
        Spacer(modifier = Modifier.height(100.dp))
    }
}

@Composable
fun ServerBans(guild: Guild, guildApi: GuildApi = koinInject()) {
    var bans by remember { mutableStateOf<List<Ban>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    val scope = rememberCoroutineScope()
    
    LaunchedEffect(guild.id) {
        isLoading = true
        bans = guildApi.getGuildBans(guild.id)
        isLoading = false
    }

    SettingsLayout {
        Material3SettingsGroup(title = "Bans") {
            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else if (bans.isEmpty()) {
                Text("No banned users", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    bans.forEach { ban ->
                        val user = ban.user
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f)
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                                AsyncImage(
                                    model = avatarUrl,
                                    contentDescription = null,
                                    modifier = Modifier.size(36.dp).clip(CircleShape)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(user.global_name ?: user.username ?: "Unknown", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                    Text(ban.reason ?: "No reason provided", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                Button(
                                    onClick = {
                                        scope.launch {
                                            if (guildApi.unbanUser(guild.id, user.id)) {
                                                bans = bans.filter { it.user.id != user.id }
                                            }
                                        }
                                    },
                                    colors = ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.errorContainer,
                                        contentColor = MaterialTheme.colorScheme.onErrorContainer
                                    ),
                                    shape = RoundedCornerShape(8.dp)
                                ) {
                                    Text("Unban")
                                }
                            }
                        }
                    }
                }
            }
        }
        Spacer(modifier = Modifier.height(100.dp))
    }
}
