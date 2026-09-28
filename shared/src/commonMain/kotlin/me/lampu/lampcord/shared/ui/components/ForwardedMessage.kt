package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.messagebody.MessageAttachments
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.DateTimeUtils
import org.koin.compose.koinInject

@Composable
fun ForwardedMessage(
    message: Message,
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    gatewayManager: GatewayManager = koinInject()
) {
    val snapshot = message.message_snapshots?.firstOrNull() ?: return
    val msg = snapshot.message
    
    Column(
        modifier = Modifier
            .padding(start = 4.dp, top = 4.dp)
            .drawBehind {
                drawLine(
                    color = Color.Gray.copy(alpha = 0.3f),
                    start = androidx.compose.ui.geometry.Offset(2.dp.toPx(), 4.dp.toPx()),
                    end = androidx.compose.ui.geometry.Offset(2.dp.toPx(), size.height - 4.dp.toPx()),
                    strokeWidth = 4.dp.toPx(),
                    cap = StrokeCap.Round
                )
            }
            .padding(start = 16.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.Forward,
                contentDescription = null,
                modifier = Modifier.size(12.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
            Text(
                text = "Forwarded",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontStyle = FontStyle.Italic,
                    fontWeight = FontWeight.Bold
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
        
        DiscordMarkdownText(
            content = msg.content,
            style = MaterialTheme.typography.bodyMedium.copy(fontSize = 15.sp)
        )

        if (msg.attachments.isNotEmpty() || msg.embeds.isNotEmpty() || !msg.sticker_items.isNullOrEmpty() || !msg.components.isNullOrEmpty()) {
            MessageAttachments(msg.attachments, msg.embeds, msg.sticker_items, components = msg.components)
        }

        val reference = message.message_reference
        if (reference != null) {
            val guildId = reference.guild_id
            val channelId = reference.channel_id
            val canLink = channelId != null
            
            val guilds by guildStore.guilds.collectAsState()
            val allChannels by guildStore.allGuildChannels.collectAsState()
            val privateChannels by guildStore.privateChannels.collectAsState()

            val guild = guildId?.let { id -> guilds.firstOrNull { it.id == id } }
            
            Surface(
                onClick = {
                    if (canLink) {
                        if (guildId != null) {
                            if (guild != null) {
                                navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                                val targetChannel = allChannels[channelId]
                                targetChannel?.let { navigationStore.selectChannel(it) }
                            }
                        } else {
                            val dm = privateChannels.firstOrNull { it.id == channelId }
                            dm?.let { 
                                navigationStore.selectHome()
                                navigationStore.selectChannel(it)
                            }
                        }
                    }
                },
                enabled = canLink,
                color = Color.Transparent,
                shape = MaterialTheme.shapes.extraSmall
            ) {
                Row(
                    modifier = Modifier.padding(vertical = 0.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    if (guild != null) {
                        val iconUrl = guild.icon?.let {
                            "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=48"
                        }
                        if (iconUrl != null) {
                            AsyncImage(
                                model = iconUrl,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp).clip(MaterialTheme.shapes.extraSmall)
                            )
                        } else {
                            Surface(modifier = Modifier.size(16.dp), shape = MaterialTheme.shapes.extraSmall, color = MaterialTheme.colorScheme.primaryContainer) {
                                Box(contentAlignment = Alignment.Center) {
                                    Text(guild.name?.take(1) ?: "", style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp))
                                }
                            }
                        }
                        
                        Text(
                            text = guild.name ?: "Unknown Server",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    } else {
                        // Fallback to "Direct Message" or similar if it's not a guild
                        Icon(
                            imageVector = Icons.Rounded.AlternateEmail,
                            contentDescription = null,
                            modifier = Modifier.size(14.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                        Text(
                            text = "Direct Message",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    
                    Text(
                        text = " • ",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    )
                    
                    val timeStr = remember(msg.timestamp) {
                        if (msg.timestamp.isEmpty()) "" else DateTimeUtils.formatTimestamp(msg.timestamp)
                    }
                    
                    Text(
                        text = timeStr,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    )
                    
                    if (canLink) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                            contentDescription = null,
                            modifier = Modifier.size(14.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
                        )
                    }
                }
            }
        }
    }
}
