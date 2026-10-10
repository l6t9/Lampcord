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
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withLink
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

            // One text flow rather than separate Texts in a FlowRow: the pieces used to wrap independently, which
            // dropped the space between the greeting and the name and left a stray leading space on the next line.
            val (prefix, suffix) = remember(message.id) {
                val templates = listOf(
                    "Welcome, " to ". We hope you brought pizza.",
                    "" to " just slid into the server!",
                    "Everyone welcome " to "!",
                    "" to " just showed up!",
                    "" to " hopped into the server.",
                )
                templates[message.id.takeLast(1).toIntOrNull()?.let { it % templates.size } ?: 0]
            }
            val name = message.author?.global_name ?: message.author?.username ?: "Unknown User"
            var profilePosition by remember { mutableStateOf(Offset.Zero) }
            val nameStyle = SpanStyle(fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
            val joinText = remember(prefix, suffix, name, nameStyle) {
                buildAnnotatedString {
                    append(prefix)
                    withLink(
                        LinkAnnotation.Clickable("profile", TextLinkStyles(style = nameStyle)) {
                            message.author?.let { profileStore.showProfile(it.id, position = profilePosition) }
                        }
                    ) { append(name) }
                    append(suffix)
                }
            }
            Text(
                text = joinText,
                style = MaterialTheme.typography.bodyMedium,
                // Softer than the name so the person who joined is what stands out in a long welcome channel.
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .weight(1f)
                    .onGloballyPositioned { profilePosition = it.positionInRoot() }
            )
            
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
    val embed = message.embeds.firstOrNull()

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
