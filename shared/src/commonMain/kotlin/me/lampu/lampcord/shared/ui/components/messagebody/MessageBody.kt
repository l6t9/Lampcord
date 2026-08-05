package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState

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
    val viewableItems: List<DiscordMedia> = images + videos
    
    if (images.isNotEmpty()) {
        MessageMosaic(images, onOpenItem = { imageIndex ->
            chatState.openAttachmentViewer(viewableItems, imageIndex)
        })
    }
    
    if (videos.isNotEmpty()) {
        Spacer(modifier = Modifier.height(8.dp))
        videos.forEachIndexed { videoIndex, video ->
            VideoAttachment(
                video = video,
                onClick = { chatState.openAttachmentViewer(viewableItems, images.size + videoIndex) }
            )
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
        EmbedView(embed, chatState)
    }

    components?.let {
        MessageComponentsRow(it, chatState)
    }
}
