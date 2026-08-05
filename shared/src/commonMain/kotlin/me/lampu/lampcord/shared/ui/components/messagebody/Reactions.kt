package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.model.getDisplayUrl

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReactionsView(message: Message, chatState: ChatState) {
    val reactions = message.reactions ?: return
    if (reactions.isEmpty()) return

    CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides 0.dp) {
        androidx.compose.foundation.layout.FlowRow(
            modifier = Modifier.padding(top = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            reactions.forEach { reaction ->
                val isMe = reaction.me
                val emojiStr = if (reaction.emoji.id != null) "${reaction.emoji.name}:${reaction.emoji.id}" else reaction.emoji.name ?: ""
                
                FilterChip(
                    modifier = Modifier.height(28.dp),
                    selected = isMe,
                    onClick = {
                        if (isMe) {
                            chatState.removeReaction(message.channel_id, message.id, emojiStr)
                        } else {
                            chatState.addReaction(message.channel_id, message.id, emojiStr)
                        }
                    },
                    label = {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            val emojiUrl = reaction.emoji.getDisplayUrl()
                            var loadFailed by remember { mutableStateOf(false) }
                            if (emojiUrl != null && !loadFailed) {
                                AsyncImage(
                                    model = emojiUrl, 
                                    contentDescription = reaction.emoji.name, 
                                    modifier = Modifier.size(16.dp), 
                                    showPlaceholder = false, 
                                    onState = { state -> if (state is coil3.compose.AsyncImagePainter.State.Error) loadFailed = true }
                                )
                            } else {
                                Text(reaction.emoji.name ?: "", fontSize = 14.sp)
                            }
                            Text(
                                text = reaction.count.toString(), 
                                style = MaterialTheme.typography.labelSmall, 
                                fontWeight = FontWeight.Bold
                            )
                        }
                    },
                    shape = RoundedCornerShape(8.dp),
                    colors = FilterChipDefaults.filterChipColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                        selectedContainerColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.2f),
                        labelColor = MaterialTheme.colorScheme.onSurfaceVariant,
                        selectedLabelColor = MaterialTheme.colorScheme.primary
                    ),
                    border = FilterChipDefaults.filterChipBorder(
                        enabled = true,
                        selected = isMe,
                        borderColor = Color.Transparent,
                        selectedBorderColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.5f),
                        borderWidth = 1.dp
                    )
                )
            }
        }
    }
}
