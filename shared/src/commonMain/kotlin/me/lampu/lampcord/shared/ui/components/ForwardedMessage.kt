package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.messagebody.MessageAttachments

@Composable
fun ForwardedMessage(snapshot: MessageSnapshot, chatState: ChatState) {
    Column(
        modifier = Modifier
            .padding(start = 12.dp, top = 4.dp)
            .drawBehind {
                val strokeWidth = 2.dp.toPx()
                drawLine(
                    color = Color.Gray.copy(alpha = 0.3f),
                    start = androidx.compose.ui.geometry.Offset(0f, 0f),
                    end = androidx.compose.ui.geometry.Offset(0f, size.height),
                    strokeWidth = strokeWidth,
                    cap = androidx.compose.ui.graphics.StrokeCap.Round
                )
            }
            .padding(start = 12.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                imageVector = Icons.Rounded.Reply,
                contentDescription = null,
                modifier = Modifier.size(12.dp).graphicsLayer(scaleX = -1f),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
            Spacer(Modifier.width(4.dp))
            Text(
                text = "Forwarded",
                style = MaterialTheme.typography.labelSmall,
                fontStyle = androidx.compose.ui.text.font.FontStyle.Italic,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
        
        val msg = snapshot.message
        Row(modifier = Modifier.fillMaxWidth().padding(top = 4.dp)) {
            // Snapshot messages might not have an author in the API response
            val authorName = "Original Message" // Fallback
            
            Surface(modifier = Modifier.size(24.dp), shape = CircleShape, color = MaterialTheme.colorScheme.secondaryContainer) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Rounded.Reply,
                        contentDescription = null,
                        modifier = Modifier.size(14.dp).graphicsLayer(scaleX = -1f),
                        tint = MaterialTheme.colorScheme.onSecondaryContainer
                    )
                }
            }
            
            Spacer(Modifier.width(8.dp))
            
            Column {
                Text(
                    text = authorName,
                    style = MaterialTheme.typography.titleSmall.copy(fontSize = 13.sp),
                    fontWeight = FontWeight.Bold
                )
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
