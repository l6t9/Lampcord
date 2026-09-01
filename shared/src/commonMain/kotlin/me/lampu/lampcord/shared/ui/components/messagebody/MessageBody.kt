package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Attachment
import me.lampu.lampcord.shared.model.DiscordMedia
import me.lampu.lampcord.shared.model.Embed
import me.lampu.lampcord.shared.model.EmbedImage
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
            content = message.content,
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
    content: String? = null,
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

    // Discord does not always create an embed for direct GIF links. Render
    // those URLs as media as a fallback, especially for media.tenor.com links.
    val embeddedGifUrls = embeds.flatMap { embed ->
        listOfNotNull(
            embed.image?.url,
            embed.image?.proxy_url,
            embed.url
        ).filter { it.isGifUrl() }
    }.map { it.normalizedMediaUrl() }.toSet()

    extractGifUrls(content)
        .filterNot { it.normalizedMediaUrl() in embeddedGifUrls }
        .forEach { url ->
            val gif = EmbedImage(url = url, proxy_url = url)
            AttachmentImage(
                media = gif,
                onClick = { navigationStore.openAttachmentViewer(listOf(gif), 0) },
                modifier = Modifier
                    .padding(vertical = 4.dp)
                    .widthIn(max = 400.dp)
                    .fillMaxWidth()
                    .aspectRatio(1f)
            )
        }

    components?.let {
        MessageComponentsRow(it)
    }
}

private val directGifUrlPattern = Regex(
    """https?://[^\s<>()\[\]]+\.gif(?:\?[^\s<>()\[\]]*)?(?:#[^\s<>()\[\]]*)?""",
    RegexOption.IGNORE_CASE
)

private fun extractGifUrls(content: String?): List<String> = content
    ?.let { directGifUrlPattern.findAll(it).map { match -> match.value.trimEnd('.', ',', '!', '?', ';', ':') }.distinct().toList() }
    ?: emptyList()

private fun String.isGifUrl(): Boolean {
    val path = substringBefore('?').substringBefore('#')
    return path.endsWith(".gif", ignoreCase = true)
}

private fun String.normalizedMediaUrl(): String = trimEnd('.', ',', '!', '?', ';', ':')
