package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.messagebody.MessageAttachments

@Composable
fun ForwardedMessage(snapshot: MessageSnapshot, chatState: ChatState) {
    val msg = snapshot.message
    
    Column(
        modifier = Modifier
            .padding(vertical = 4.dp)
            .fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.padding(bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Filled.Forward,
                contentDescription = null,
                modifier = Modifier.size(14.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
            Spacer(Modifier.width(6.dp))
            Text(
                text = "Forwarded",
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }

        Surface(
            modifier = Modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
            shape = RoundedCornerShape(8.dp),
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
        ) {
            Column(modifier = Modifier.padding(12.dp)) {
                val author = msg.author
                if (author != null) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        val avatarUrl = author.avatar?.let {
                            "https://cdn.discordapp.com/avatars/${author.id}/$it.png?size=48"
                        }
                        
                        if (avatarUrl != null) {
                            AsyncImage(
                                model = avatarUrl,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp).clip(CircleShape)
                            )
                        } else {
                            Surface(modifier = Modifier.size(18.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
                        }
                        
                        Spacer(Modifier.width(8.dp))
                        
                        Text(
                            text = author.global_name ?: author.username,
                            style = MaterialTheme.typography.titleSmall.copy(fontSize = 13.sp),
                            fontWeight = FontWeight.Bold
                        )
                    }
                    
                    Spacer(Modifier.height(4.dp))
                }
                
                DiscordMarkdownText(
                    content = msg.content,
                    style = MaterialTheme.typography.bodyMedium.copy(fontSize = 14.sp),
                    chatState = chatState
                )

                if (msg.attachments.isNotEmpty() || msg.embeds.isNotEmpty() || !msg.sticker_items.isNullOrEmpty() || !msg.components.isNullOrEmpty()) {
                    MessageAttachments(msg.attachments, msg.embeds, msg.sticker_items, components = msg.components, chatState = chatState)
                }
            }
        }
    }
}
