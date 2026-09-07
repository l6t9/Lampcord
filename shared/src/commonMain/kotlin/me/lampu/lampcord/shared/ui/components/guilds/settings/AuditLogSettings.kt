package me.lampu.lampcord.shared.ui.components.guilds.settings

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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.AuditLog
import me.lampu.lampcord.shared.model.AuditLogEntry
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ServerAuditLog(guild: Guild, guildApi: GuildApi = koinInject()) {
    var auditLog by remember { mutableStateOf<AuditLog?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    
    LaunchedEffect(guild.id) {
        isLoading = true
        auditLog = guildApi.getGuildAuditLog(guild.id)
        isLoading = false
    }

    SettingsLayout {
        Material3SettingsGroup(title = "Audit Log") {
            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else if (auditLog == null || auditLog!!.audit_log_entries.isEmpty()) {
                Text("No audit log entries", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    auditLog!!.audit_log_entries.forEach { entry ->
                        val user = auditLog!!.users.find { it.id == entry.user_id }
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
                                val avatarUrl = user?.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                                AsyncImage(
                                    model = avatarUrl,
                                    contentDescription = null,
                                    modifier = Modifier.size(36.dp).clip(CircleShape)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(user?.global_name ?: user?.username ?: "Unknown User", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                    Text(formatAuditLogAction(entry), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
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

private fun formatAuditLogAction(entry: AuditLogEntry): String {
    return when (entry.action_type) {
        1 -> "Created Guild"
        2 -> "Updated Guild"
        10 -> "Created Channel"
        11 -> "Updated Channel"
        12 -> "Deleted Channel"
        13 -> "Created Channel Overwrite"
        14 -> "Updated Channel Overwrite"
        15 -> "Deleted Channel Overwrite"
        20 -> "Kicked Member"
        21 -> "Pruned Members"
        22 -> "Banned Member"
        23 -> "Unbanned Member"
        24 -> "Updated Member"
        25 -> "Updated Member Roles"
        26 -> "Moved Member"
        27 -> "Disconnected Member"
        28 -> "Added Bot"
        30 -> "Created Role"
        31 -> "Updated Role"
        32 -> "Deleted Role"
        40 -> "Created Invite"
        41 -> "Updated Invite"
        42 -> "Deleted Invite"
        50 -> "Created Webhook"
        51 -> "Updated Webhook"
        52 -> "Deleted Webhook"
        60 -> "Created Emoji"
        61 -> "Updated Emoji"
        62 -> "Deleted Emoji"
        72 -> "Deleted Messages"
        73 -> "Bulk Deleted Messages"
        74 -> "Pinned Message"
        75 -> "Unpinned Message"
        80 -> "Created Integration"
        81 -> "Updated Integration"
        82 -> "Deleted Integration"
        else -> "Unknown Action (${entry.action_type})"
    }
}
