package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageReaction
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.chat.ReactionPickerSheet
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable
fun ReactionsView(
    message: Message,
    messageStore: MessageStore = koinInject()
) {
    val reactions = message.reactions ?: return
    if (reactions.isEmpty()) return
    
    var showReactionUsers by remember { mutableStateOf<MessageReaction?>(null) }
    var showAddReactionPicker by remember { mutableStateOf(false) }

    CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides 0.dp) {
        androidx.compose.foundation.layout.FlowRow(
            modifier = Modifier.padding(top = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            reactions.forEach { reaction ->
                val isMe = reaction.me
                
                Surface(
                    modifier = Modifier
                        .height(28.dp)
                        .clip(RoundedCornerShape(8.dp)),
                    shape = RoundedCornerShape(8.dp),
                    color = if (isMe) MaterialTheme.colorScheme.primary.copy(alpha = 0.2f) else MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                    border = if (isMe) androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.5f)) else null,
                    onClick = {
                        messageStore.toggleReaction(message, reaction.emoji)
                    }
                ) {
                    Row(
                        modifier = Modifier
                            .padding(horizontal = 8.dp)
                            .pointerInput(reaction) {
                                detectTapGestures(
                                    onLongPress = {
                                        showReactionUsers = reaction
                                    }
                                )
                            },
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
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
                            fontWeight = FontWeight.Bold,
                            color = if (isMe) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            // Add Reaction Button
            Surface(
                onClick = { showAddReactionPicker = true },
                modifier = Modifier
                    .size(width = 36.dp, height = 28.dp),
                shape = RoundedCornerShape(8.dp),
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Filled.AddReaction,
                        contentDescription = "Add Reaction",
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }

    if (showReactionUsers != null) {
        ReactionUsersDialog(
            channelId = message.channel_id,
            messageId = message.id,
            reactions = reactions,
            initialEmoji = showReactionUsers!!,
            onDismiss = { showReactionUsers = null }
        )
    }

    if (showAddReactionPicker) {
        ReactionPickerSheet(
            onDismiss = { showAddReactionPicker = false },
            onEmojiSelected = { emoji ->
                messageStore.toggleReaction(message, emoji)
            }
        )
    }
}
