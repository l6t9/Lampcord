package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun MemberHeader(channel: Channel, chatState: ChatState) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 16.dp, bottom = 8.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
            val icon = if (isDm) {
                Icons.Outlined.AlternateEmail
            } else {
                when (channel.type) {
                    15 -> Icons.Outlined.Forum
                    2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                    5 -> Icons.Filled.Campaign
                    else -> Icons.Filled.Tag
                }
            }
            
            val name = if (isDm) {
                val recipient = channel.recipients?.firstOrNull()
                recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
            } else {
                me.lampu.lampcord.shared.utils.CleanUtils.cleanChannelName(channel.name ?: "unnamed")
            }

            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(28.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.width(12.dp))
            Text(
                text = name,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        if (channel.topic?.isNotBlank() == true) {
            Spacer(Modifier.height(4.dp))
            Text(
                text = channel.topic,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(horizontal = 16.dp)
            )
        }

        Spacer(Modifier.height(16.dp))
        
        HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
        
        Spacer(Modifier.height(16.dp))

        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            HeaderButton(
                icon = Icons.Filled.Search,
                label = "Search",
                onClick = { chatState.isSearchVisible = true },
                modifier = Modifier.weight(1f)
            )
            HeaderButton(
                icon = Icons.Filled.PushPin,
                label = "Pins",
                onClick = { chatState.isPinsVisible = true },
                modifier = Modifier.weight(1f)
            )
            HeaderButton(
                icon = Icons.Filled.Notifications,
                label = "Notifications",
                onClick = { },
                enabled = false,
                modifier = Modifier.weight(1f)
            )
            HeaderButton(
                icon = Icons.Filled.Settings,
                label = "Settings",
                onClick = { },
                enabled = false,
                modifier = Modifier.weight(1f)
            )
        }
        
        Spacer(Modifier.height(16.dp))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))
    }
}

@Composable
private fun HeaderButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true
) {
    val contentAlpha = if (enabled) 1f else 0.4f
    Column(
        modifier = modifier
            .clip(MaterialTheme.shapes.medium)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(vertical = 8.dp)
            .alpha(contentAlpha),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint = MaterialTheme.colorScheme.onSurface
        )
        Spacer(Modifier.height(4.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface,
            textAlign = TextAlign.Center
        )
    }
}
