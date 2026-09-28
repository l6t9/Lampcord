package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.DiscordGreen
import me.lampu.lampcord.shared.ui.theme.DiscordRed
import me.lampu.lampcord.shared.ui.theme.Fuchsia
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import org.koin.compose.koinInject

@Composable
fun SystemMessage(
    message: Message,
    profileStore: ProfileStore = koinInject(),
    messageStore: MessageStore = koinInject()
) {
    val targetMessageId = message.message_reference?.message_id

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .then(
                if (targetMessageId != null) {
                    Modifier.clickableCursor {
                        messageStore.scrollToMessageId = targetMessageId
                    }
                } else Modifier
            )
            .padding(vertical = 4.dp, horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        val (icon, iconTint, text) = when (message.type) {
            1 -> Triple(Icons.Filled.PersonAdd, DiscordGreen, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} added a recipient.")
            2 -> Triple(Icons.Filled.PersonRemove, DiscordRed, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} removed a recipient.")
            3 -> Triple(Icons.AutoMirrored.Filled.VolumeUp, MaterialTheme.colorScheme.primary, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} started a call.")
            4 -> Triple(Icons.Filled.Edit, MaterialTheme.colorScheme.onSurfaceVariant, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} changed the channel name.")
            5 -> Triple(Icons.Filled.Image, MaterialTheme.colorScheme.onSurfaceVariant, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} changed the channel icon.")
            6 -> Triple(Icons.Filled.PushPin, MaterialTheme.colorScheme.primary, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} pinned a message to this channel.")
            7 -> Triple(Icons.AutoMirrored.Filled.ArrowForward, DiscordGreen, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} joined the server.")
            8, 9, 10, 11 -> Triple(Icons.Filled.RocketLaunch, Fuchsia, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} just boosted the server!")
            12 -> Triple(Icons.Filled.Campaign, MaterialTheme.colorScheme.primary, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} added a followed channel to this channel.")
            18 -> Triple(Icons.Filled.Tag, MaterialTheme.colorScheme.primary, "${message.author?.global_name ?: message.author?.username ?: "Unknown"} started a thread.")
            46 -> Triple(Icons.Filled.BarChart, MaterialTheme.colorScheme.primary, formatPollResultMessage(message))
            else -> Triple(Icons.Filled.Info, MaterialTheme.colorScheme.onSurfaceVariant, "System message (Type ${message.type})")
        }

        if (message.type == 7) {
            Box(modifier = Modifier.width(44.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                androidx.compose.foundation.layout.FlowRow(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.Center
                ) {
                    val randomMessages = listOf("pizzaPre", "slid", "everyoneWelcomePre", "showedUp", "hopped")
                    val selectedMessage = randomMessages[message.id.takeLast(1).toIntOrNull()?.let { it % randomMessages.size } ?: 0]

                    if (selectedMessage.contains("Pre")) {
                        Text(
                            text = when (selectedMessage) {
                                "pizzaPre" -> "Welcome,"
                                "everyoneWelcomePre" -> "Everyone welcome"
                                else -> ""
                            },
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                    }

                    var profilePosition by remember { mutableStateOf(Offset.Zero) }
                    Text(
                        text = message.author?.global_name ?: message.author?.username ?: "Unknown User",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier
                            .onGloballyPositioned { profilePosition = it.positionInRoot() }
                            .clickableCursor { message.author?.let { profileStore.showProfile(it.id, position = profilePosition) } }
                    )

                    Text(
                        text = when (selectedMessage) {
                            "pizzaPre" -> ". We hope you brought pizza."
                            "everyoneWelcomePre" -> "!"
                            "slid" -> " just slid into the server!"
                            "showedUp" -> " just showed up!"
                            "hopped" -> " hopped into the server."
                            else -> " just slid into the server!"
                        },
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                }
            }
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        } else {
            Box(modifier = Modifier.width(40.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Text(
                text = text,
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}

private fun formatPollResultMessage(message: Message): String {
    val authorName = message.author?.global_name ?: message.author?.username ?: "User"
    val embed = message.embeds?.firstOrNull()

    val questionText = embed?.fields?.find { it.name == "poll_question_text" }?.value
        ?: embed?.title
        ?: message.poll?.question?.text
        ?: message.content.takeIf { it.isNotBlank() }
        ?: "poll"

    val victorText = embed?.fields?.find { it.name == "victor_answer_text" }?.value
    val victorVotes = embed?.fields?.find { it.name == "victor_answer_votes" }?.value?.toIntOrNull() ?: 0
    val totalVotes = embed?.fields?.find { it.name == "total_votes" }?.value?.toIntOrNull()
        ?: message.poll?.results?.answer_counts?.sumOf { it.count }
        ?: 0

    val percent = if (totalVotes > 0) (victorVotes * 100 / totalVotes) else 0

    return when {
        totalVotes == 0 -> "$authorName's poll \"$questionText\" has closed! There were no votes."
        victorText.isNullOrBlank() -> "$authorName's poll \"$questionText\" has closed! The result was a draw ($percent%)."
        else -> "$authorName's poll \"$questionText\" has closed! The winner was $victorText ($percent%)."
    }
}
