package com.example.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.lampcord.shared.model.Message
import com.example.lampcord.shared.model.ReactionCountDetails
import com.example.lampcord.shared.model.MessageReaction
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.components.AsyncImage

@Composable
fun ReactionsView(message: Message, chatState: ChatState) {
    val reactions = message.reactions ?: return
    if (reactions.isEmpty()) return

    androidx.compose.foundation.layout.FlowRow(
        modifier = Modifier.padding(top = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        reactions.forEach { reaction ->
            val isMe = reaction.me
            Surface(
                onClick = {
                    val emojiStr = if (reaction.emoji.id != null) "${reaction.emoji.name}:${reaction.emoji.id}" else reaction.emoji.name ?: ""
                    if (isMe) {
                        chatState.removeReaction(message.channel_id, message.id, emojiStr)
                    } else {
                        chatState.addReaction(message.channel_id, message.id, emojiStr)
                    }
                },
                color = if (isMe) MaterialTheme.colorScheme.primary.copy(alpha = 0.2f) else MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                shape = RoundedCornerShape(8.dp),
                border = if (isMe) BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.5f)) else null
            ) {
                Row(modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    val emojiUrl = reaction.emoji.getDisplayUrl()
                    var loadFailed by remember { mutableStateOf(false) }
                    if (emojiUrl != null && !loadFailed) {
                        AsyncImage(model = emojiUrl, contentDescription = reaction.emoji.name, modifier = Modifier.size(16.dp), showPlaceholder = false, onState = { state -> if (state is coil3.compose.AsyncImagePainter.State.Error) loadFailed = true })
                    } else {
                        Text(reaction.emoji.name ?: "", fontSize = 14.sp)
                    }
                    Text(text = reaction.count.toString(), style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold, color = if (isMe) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}
