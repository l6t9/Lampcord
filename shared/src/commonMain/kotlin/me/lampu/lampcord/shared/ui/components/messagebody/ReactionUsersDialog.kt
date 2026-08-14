package me.lampu.lampcord.shared.ui.components.messagebody

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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.MessageReaction
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReactionUsersDialog(
    channelId: String,
    messageId: String,
    reactions: List<MessageReaction>,
    initialEmoji: MessageReaction,
    onDismiss: () -> Unit,
    messageApi: MessageApi = koinInject()
) {
    var selectedReaction by remember { mutableStateOf(initialEmoji) }
    var users by remember { mutableStateOf<List<User>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }

    val emojiStr = if (selectedReaction.emoji.id != null) "${selectedReaction.emoji.name}:${selectedReaction.emoji.id}" else selectedReaction.emoji.name ?: ""

    LaunchedEffect(selectedReaction) {
        isLoading = true
        users = messageApi.getReactionUsers(channelId, messageId, emojiStr)
        isLoading = false
    }

    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 500.dp),
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surface
        ) {
            Column {
                SecondaryScrollableTabRow(
                    selectedTabIndex = reactions.indexOf(selectedReaction),
                    containerColor = Color.Transparent,
                    edgePadding = 16.dp,
                    divider = {}
                ) {
                    reactions.forEach { reaction ->
                        Tab(
                            selected = selectedReaction == reaction,
                            onClick = { selectedReaction = reaction }
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(4.dp)
                            ) {
                                val url = reaction.emoji.getDisplayUrl()
                                if (url != null) {
                                    AsyncImage(model = url, contentDescription = null, modifier = Modifier.size(16.dp))
                                } else {
                                    Text(reaction.emoji.name ?: "", fontSize = 14.sp)
                                }
                                Text(reaction.count.toString(), style = MaterialTheme.typography.labelMedium)
                            }
                        }
                    }
                }

                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

                Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                    if (isLoading) {
                        ContainedLoadingIndicator(modifier = Modifier.align(Alignment.Center))
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(16.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            items(users) { user ->
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                                    AsyncImage(
                                        model = avatarUrl,
                                        contentDescription = null,
                                        modifier = Modifier.size(32.dp).clip(CircleShape)
                                    )
                                    Spacer(modifier = Modifier.width(12.dp))
                                    Text(
                                        text = user.global_name ?: user.username ?: "Unknown",
                                        style = MaterialTheme.typography.bodyLarge,
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
