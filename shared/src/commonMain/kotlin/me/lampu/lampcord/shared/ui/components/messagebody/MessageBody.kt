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
        MessageAttachments(
            attachments = message.attachments,
            embeds = message.embeds,
            stickerItems = message.sticker_items,
            stickers = message.stickers,
            poll = message.poll,
            components = message.components
        )
    }
}

@Composable
fun MessageAttachments(
    attachments: List<Attachment>,
    embeds: List<Embed>,
    stickerItems: List<StickerItem>? = null,
    stickers: List<me.lampu.lampcord.shared.model.Sticker>? = null,
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

    if (!stickerItems.isNullOrEmpty()) {
        StickersView(stickerItems)
    } else if (!stickers.isNullOrEmpty()) {
        StickersView(stickers.map { StickerItem(it.id, it.name, it.format_type) })
    }

    poll?.let { PollView(it) }

    val inviteRegex = Regex("""discord(?:\.com/invite|\.gg)/([a-zA-Z0-9\-]+)""")
    val processedInvites = mutableSetOf<String>()

    embeds.forEach { embed ->
        EmbedView(embed)
        
        // Extract and show native invite preview if it's an invite link
        embed.url?.let { url ->
            inviteRegex.find(url)?.groupValues?.get(1)?.let { code ->
                if (processedInvites.add(code)) {
                    InviteEmbedView(code)
                }
            }
        }
    }

    components?.let {
        MessageComponentsRow(it)
    }
}
