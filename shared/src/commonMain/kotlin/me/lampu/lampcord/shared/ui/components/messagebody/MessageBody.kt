package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun MessageBody(message: Message, chatState: ChatState) {
    if (message.attachments.isNotEmpty() || message.embeds.isNotEmpty() || !message.sticker_items.isNullOrEmpty() || message.poll != null || !message.components.isNullOrEmpty()) {
        Spacer(modifier = Modifier.height(8.dp))
        MessageAttachments(message.attachments, message.embeds, message.sticker_items, message.poll, message.components, chatState)
    }
}

@Composable
fun MessageAttachments(
    attachments: List<Attachment>, 
    embeds: List<Embed>, 
    stickers: List<StickerItem>? = null, 
    poll: Poll? = null, 
    components: List<MessageComponent>? = null,
    chatState: ChatState
) {
    val images = attachments.filter { it.content_type?.startsWith("image/") == true }
    val videos = attachments.filter { it.content_type?.startsWith("video/") == true }
    val otherFiles = attachments.filter { it.content_type?.startsWith("image/") != true && it.content_type?.startsWith("video/") != true }
    
    if (images.isNotEmpty()) {
        MessageMosaic(images)
    }
    
    if (videos.isNotEmpty()) {
        Spacer(modifier = Modifier.height(8.dp))
        videos.forEach { video ->
            VideoAttachment(video)
            Spacer(modifier = Modifier.height(4.dp))
        }
    }

    if (otherFiles.isNotEmpty()) {
        Spacer(modifier = Modifier.height(8.dp))
        otherFiles.forEach { file ->
            FileAttachmentView(file)
        }
    }

    stickers?.let {
        StickersView(it)
    }

    poll?.let {
        PollView(it)
    }

    embeds.forEach { embed ->
        EmbedView(embed)
    }

    components?.let {
        MessageComponentsRow(it, chatState)
    }
}

@Composable
fun PollView(poll: Poll) {
    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 425.dp).fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Filled.BarChart, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.primary)
                Spacer(Modifier.width(8.dp))
                Text(text = "Poll", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
            }
            Spacer(Modifier.height(8.dp))
            Text(text = poll.question.text ?: "Untitled Poll", style = MaterialTheme.typography.titleMedium, fontWeight = androidx.compose.ui.text.font.FontWeight.Bold)
            Spacer(Modifier.height(12.dp))
            val totalVotes = poll.results?.answer_counts?.sumOf { it.count } ?: 0
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                poll.answers.forEach { answer ->
                    val result = poll.results?.answer_counts?.find { it.id == answer.answer_id }
                    val count = result?.count ?: 0
                    val percentage = if (totalVotes > 0) count.toFloat() / totalVotes.toFloat() else 0f
                    Column {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Text(text = answer.poll_media.text ?: "", style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
                            if (totalVotes > 0) {
                                Text(text = "$count (${(percentage * 100).toInt()}%)", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                        Spacer(Modifier.height(4.dp))
                        LinearProgressIndicator(
                            progress = { percentage },
                            modifier = Modifier.fillMaxWidth().height(8.dp).clip(CircleShape),
                            color = if (result?.me_voted == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.primary.copy(alpha = 0.4f),
                            trackColor = MaterialTheme.colorScheme.surfaceVariant
                        )
                    }
                }
            }
            if (totalVotes > 0) {
                Spacer(Modifier.height(12.dp))
                Text(text = "$totalVotes votes", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
fun SelectMenuView(component: MessageComponent) {
    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 425.dp).fillMaxWidth().height(40.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shape = RoundedCornerShape(4.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
        onClick = { /* TODO */ },
        enabled = component.disabled != true
    ) {
        Row(modifier = Modifier.padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(text = component.placeholder ?: "Select an option...", style = MaterialTheme.typography.bodyMedium, color = if (component.disabled == true) MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f) else MaterialTheme.colorScheme.onSurfaceVariant)
            Icon(imageVector = Icons.Filled.ExpandMore, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun MessageComponentsRow(components: List<MessageComponent>, chatState: ChatState) {
    androidx.compose.foundation.layout.FlowRow(
        modifier = Modifier.padding(top = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        components.forEach { component ->
            when (component.type) {
                1 -> { component.components?.let { MessageComponentsRow(it, chatState) } }
                2 -> {
                    val isLink = component.style == 5
                    val buttonColor = when (component.style) {
                        1 -> ButtonDefaults.buttonColors(containerColor = Color(0xFF5865F2))
                        2 -> ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondaryContainer, contentColor = MaterialTheme.colorScheme.onSecondaryContainer)
                        3 -> ButtonDefaults.buttonColors(containerColor = Color(0xFF23A559))
                        4 -> ButtonDefaults.buttonColors(containerColor = Color(0xFFDA373C))
                        else -> ButtonDefaults.buttonColors()
                    }
                    val uriHandler = LocalUriHandler.current
                    Button(
                        onClick = { if (isLink && component.url != null) uriHandler.openUri(component.url) },
                        colors = buttonColor,
                        shape = RoundedCornerShape(4.dp),
                        modifier = Modifier.height(32.dp),
                        contentPadding = PaddingValues(horizontal = 12.dp),
                        enabled = component.disabled != true
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            component.emoji?.let { emoji ->
                                val emojiUrl = emoji.id?.let { "https://cdn.discordapp.com/emojis/$it.webp?size=48&animated=${emoji.animated == true}" }
                                if (emojiUrl != null) { AsyncImage(model = emojiUrl, contentDescription = null, modifier = Modifier.size(16.dp)) }
                                else { Text(emoji.name ?: "", fontSize = 14.sp) }
                            }
                            component.label?.let { Text(it, style = MaterialTheme.typography.labelMedium) }
                            if (isLink) { Icon(Icons.AutoMirrored.Filled.OpenInNew, null, modifier = Modifier.size(14.dp)) }
                        }
                    }
                }
                3, 5, 6, 7, 8 -> { SelectMenuView(component) }
            }
        }
    }
}
