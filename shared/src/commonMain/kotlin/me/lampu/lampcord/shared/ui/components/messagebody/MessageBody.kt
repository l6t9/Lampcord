package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Attachment
import me.lampu.lampcord.shared.model.DiscordMedia
import me.lampu.lampcord.shared.model.Embed
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageComponent
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.StickerItem
import me.lampu.lampcord.shared.state.NavigationStore
import org.koin.compose.koinInject

@Composable
fun MessageBody(
    message: Message
) {
    Column(modifier = Modifier.padding(top = 0.dp)) {
        // Content is rendered by MessageItem to handle edits and highlights properly
        MessageAttachments(message.attachments, message.embeds, message.sticker_items, message.poll, message.components)
    }
}

@Composable
fun MessageAttachments(
    attachments: List<Attachment>,
    embeds: List<Embed>,
    stickerItems: List<StickerItem>? = null,
    poll: Poll? = null,
    components: List<MessageComponent>? = null,
    navigationStore: NavigationStore = koinInject()
) {
    val images = attachments.filter { it.content_type?.startsWith("image/") == true }
    val videos = attachments.filter { it.content_type?.startsWith("video/") == true }
    val otherFiles = attachments.filter { it.content_type?.startsWith("image/") != true && it.content_type?.startsWith("video/") != true }
    
    val viewableItems: List<DiscordMedia> = images + videos

    if (viewableItems.isNotEmpty()) {
        MessageMosaic(viewableItems, onOpenItem = { index ->
            navigationStore.openAttachmentViewer(viewableItems, index)
        })
    }

    otherFiles.forEach { file ->
        FileAttachmentView(file)
    }

    stickerItems?.let { 
        StickersView(it)
    }

    poll?.let { PollView(it) }

    embeds.forEach { embed ->
        EmbedView(embed)
    }

    components?.let {
        MessageComponentsRow(it)
    }
}
