package me.lampu.lampcord.shared.ui.components.guilds.settings

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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun ServerAuditLog(guild: Guild, chatState: ChatState) {
    var auditLog by remember { mutableStateOf<AuditLog?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    
    LaunchedEffect(guild.id) {
        isLoading = true
        auditLog = chatState.client.getGuildAuditLog(guild.id)
        isLoading = false
    }

    SettingsLayout {
        SettingsSection(title = "Audit Log", icon = Icons.Filled.Article) {
            if (isLoading) {
                ContainedLoadingIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
            } else if (auditLog == null || auditLog!!.audit_log_entries.isEmpty()) {
                Text("No audit log entries", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    auditLog!!.audit_log_entries.forEach { entry ->
                        val user = auditLog!!.users.find { it.id == entry.user_id }
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
                            val avatarUrl = user?.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                            AsyncImage(
                                model = avatarUrl,
                                contentDescription = null,
                                modifier = Modifier.size(32.dp).clip(CircleShape)
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
