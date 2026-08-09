package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
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

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
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
                modifier = Modifier.size(24.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.width(12.dp))
            Text(
                text = name,
                style = MaterialTheme.typography.titleLarge,
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
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            val actions = listOf(
                HeaderButtonData(
                    icon = Icons.Filled.Search,
                    label = "Search",
                    onClick = { chatState.isSearchVisible = true }
                ),
                HeaderButtonData(
                    icon = Icons.Filled.PushPin,
                    label = "Pins",
                    onClick = { chatState.isPinsVisible = true }
                ),
                HeaderButtonData(
                    icon = Icons.Filled.Settings,
                    label = "Settings",
                    onClick = { },
                    enabled = false
                )
            )

            actions.forEachIndexed { index, action ->
                val shape = when {
                    actions.size == 1 -> RoundedCornerShape(20.dp)
                    index == 0 -> RoundedCornerShape(topStart = 20.dp, bottomStart = 20.dp, topEnd = 4.dp, bottomEnd = 4.dp)
                    index == actions.lastIndex -> RoundedCornerShape(topStart = 4.dp, bottomStart = 4.dp, topEnd = 20.dp, bottomEnd = 20.dp)
                    else -> RoundedCornerShape(4.dp)
                }

                HeaderButton(
                    action = action,
                    shape = shape,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

private data class HeaderButtonData(
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val label: String,
    val onClick: () -> Unit,
    val enabled: Boolean = true
)

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun HeaderButton(
    action: HeaderButtonData,
    shape: androidx.compose.ui.graphics.Shape,
    modifier: Modifier = Modifier
) {
    val contentAlpha = if (action.enabled) 1f else 0.4f
    Surface(
        onClick = action.onClick,
        enabled = action.enabled,
        modifier = modifier.height(64.dp),
        shape = shape,
        color = MaterialTheme.colorScheme.secondaryContainer,
        contentColor = MaterialTheme.colorScheme.onSecondaryContainer
    ) {
        Column(
            modifier = Modifier.fillMaxSize().padding(horizontal = 4.dp, vertical = 8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Icon(
                imageVector = action.icon,
                contentDescription = null,
                modifier = Modifier.size(20.dp).alpha(contentAlpha)
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = action.label,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSecondaryContainer.copy(alpha = contentAlpha),
                textAlign = TextAlign.Center,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                fontSize = 10.sp
            )
        }
    }
}
