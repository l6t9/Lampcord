package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
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
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.messagebody.MessageAttachments
import me.lampu.lampcord.shared.utils.DateTimeUtils

@Composable
fun ForwardedMessage(message: Message, chatState: ChatState) {
    val snapshot = message.message_snapshots?.firstOrNull() ?: return
    val msg = snapshot.message
    
    Column(
        modifier = Modifier
            .padding(start = 4.dp, top = 4.dp, bottom = 4.dp)
            .drawBehind {
                drawLine(
                    color = Color.Gray.copy(alpha = 0.3f),
                    start = androidx.compose.ui.geometry.Offset(2.dp.toPx(), 0f),
                    end = androidx.compose.ui.geometry.Offset(2.dp.toPx(), size.height),
                    strokeWidth = 2.dp.toPx(),
                    cap = StrokeCap.Round
                )
            }
            .padding(start = 16.dp)
    ) {
        // Header
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                imageVector = Icons.Filled.Forward,
                contentDescription = null,
                modifier = Modifier.size(12.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
            Spacer(Modifier.width(6.dp))
            Text(
                text = "Forwarded",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontStyle = FontStyle.Italic,
                    fontWeight = FontWeight.Bold
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
        
        Spacer(Modifier.height(4.dp))
        
        // Content
        DiscordMarkdownText(
            content = msg.content,
            style = MaterialTheme.typography.bodyMedium.copy(fontSize = 15.sp),
            chatState = chatState
        )

        if (msg.attachments.isNotEmpty() || msg.embeds.isNotEmpty() || !msg.sticker_items.isNullOrEmpty() || !msg.components.isNullOrEmpty()) {
            MessageAttachments(msg.attachments, msg.embeds, msg.sticker_items, components = msg.components, chatState = chatState)
        }

        Spacer(Modifier.height(8.dp))

        // Footer / Link
        val author = msg.author
        if (author != null) {
            val reference = message.message_reference
            val canLink = reference?.channel_id != null
            
            Surface(
                onClick = {
                    if (canLink) {
                        val guildId = reference?.guild_id
                        val channelId = reference?.channel_id!!
                        
                        if (guildId != null) {
                            val guild = chatState.guilds.find { it.id == guildId }
                            if (guild != null) {
                                chatState.selectGuild(guild)
                                val targetChannel = chatState.guildStore.allGuildChannels[guildId]?.find { it.id == channelId }
                                targetChannel?.let { chatState.selectChannel(it) }
                            }
                        } else {
                            val dm = chatState.privateChannels.find { it.id == channelId }
                            dm?.let { 
                                chatState.selectHome()
                                chatState.selectChannel(it) 
                            }
                        }
                    }
                },
                enabled = canLink,
                color = Color.Transparent,
                shape = RoundedCornerShape(4.dp)
            ) {
                Row(
                    modifier = Modifier.padding(vertical = 2.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val avatarUrl = author.avatar?.let {
                        "https://cdn.discordapp.com/avatars/${author.id}/$it.png?size=48"
                    }
                    
                    if (avatarUrl != null) {
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = null,
                            modifier = Modifier.size(16.dp).clip(RoundedCornerShape(4.dp))
                        )
                    } else {
                        Surface(modifier = Modifier.size(16.dp), shape = RoundedCornerShape(4.dp), color = MaterialTheme.colorScheme.primaryContainer) {}
                    }
                    
                    Spacer(Modifier.width(6.dp))
                    
                    Text(
                        text = author.global_name ?: author.username,
                        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
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
