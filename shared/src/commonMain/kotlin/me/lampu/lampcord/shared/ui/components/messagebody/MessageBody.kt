package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Attachment
import me.lampu.lampcord.shared.model.Embed
import me.lampu.lampcord.shared.model.EmbedImage
import me.lampu.lampcord.shared.model.EmbedVideo
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageComponent
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.StickerItem
import me.lampu.lampcord.shared.model.collectReferencedAttachments
import me.lampu.lampcord.shared.state.NavigationStore
import org.koin.compose.koinInject

@Composable
fun MessageBody(
    message: Message
) {
    Column(modifier = Modifier.padding(top = 0.dp)) {
        MessageAttachments(
            attachments = message.attachments,
            embeds = message.embeds,
            content = message.content,
            stickerItems = message.sticker_items,
            stickers = message.stickers,
            poll = message.poll,
            components = message.components,
            message = message
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
    message: Message? = null,
    navigationStore: NavigationStore = koinInject()
) {
    val hasLinkContent = remember(content) {
        content?.let {
            it.contains("discord.gg", ignoreCase = true) ||
                it.contains("discord.com/invite", ignoreCase = true) ||
                it.contains("discordapp.com/invite", ignoreCase = true) ||
                it.contains(".gif", ignoreCase = true) ||
                it.contains(".mp4", ignoreCase = true) ||
                it.contains(".webm", ignoreCase = true) ||
                it.contains(".mov", ignoreCase = true)
        } == true
    }
    if (attachments.isEmpty() && embeds.isEmpty() && stickerItems.isNullOrEmpty() &&
        stickers.isNullOrEmpty() && poll == null && components.isNullOrEmpty() && !hasLinkContent
    ) {
        return
    }

    val referencedAttachmentKeys = remember(components) { collectReferencedAttachments(components.orEmpty()) }

    val visibleAttachments = remember(attachments, referencedAttachmentKeys) {
        if (referencedAttachmentKeys.isEmpty()) {
            attachments
        } else {
            attachments.filterNot { it.id in referencedAttachmentKeys || it.filename in referencedAttachmentKeys }
        }
    }

    val images = remember(visibleAttachments) { visibleAttachments.filter { it.isImage() } }
    val videos = remember(visibleAttachments) { visibleAttachments.filter { it.isVideo() } }
    val otherFiles = remember(visibleAttachments) { visibleAttachments.filter { !it.isImage() && !it.isVideo() } }

    val viewableItems = remember(images, videos) { images + videos }

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

    poll?.let { PollView(poll = it, message = message) }

    val processedInvites = mutableSetOf<String>()

    embeds.forEach { embed ->
        EmbedView(embed)

        embed.url?.let { url ->
            DISCORD_INVITE_PATTERN.find(url)?.groupValues?.get(1)?.let { code ->
                if (processedInvites.add(code)) {
                    InviteEmbedView(code)
                }
            }
        }
    }

    content?.let {
        if (it.contains("discord", ignoreCase = true)) {
            val codes = remember(it) {
                DISCORD_INVITE_PATTERN.findAll(it)
                    .map { match -> match.groupValues[1] }
                    .distinct()
                    .toList()
            }
            codes.forEach { code ->
                if (processedInvites.add(code)) {
                    InviteEmbedView(code)
                }
            }
        }
    }

    val embeddedMediaUrls = remember(embeds) {
        embeds.flatMap { embed ->
            listOfNotNull(
                embed.image?.url,
                embed.image?.proxy_url,
                embed.video?.url,
                embed.url
            )
        }.map { it.normalizedMediaUrl() }.toSet()
    }

    val standaloneMediaUrls = remember(content, embeddedMediaUrls) {
        extractMediaUrls(content).filterNot { it.normalizedMediaUrl() in embeddedMediaUrls }
    }

    standaloneMediaUrls.forEach { url ->
            if (url.isGifUrl()) {
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
            } else {
                val video = EmbedVideo(url = url, proxy_url = url)
                GifvView(
                    video = video,
                    modifier = Modifier
                        .padding(vertical = 4.dp)
                        .widthIn(max = 500.dp)
                        .fillMaxWidth()
                        .aspectRatio(16/9f)
                )
            }
        }

    components?.let {
        MessageComponentsRow(it, attachments)
    }
}

private val mediaUrlPattern = Regex(
    """https?://[^\s<>()\[\]]+\.(gif|mp4|webm|mov)(?:\?[^\s<>()\[\]]*)?(?:#[^\s<>()\[\]]*)?""",
    RegexOption.IGNORE_CASE
)

private val DISCORD_INVITE_PATTERN =
    Regex("""(?:discord\.gg/|discord\.com/invite/|discordapp\.com/invite/|discord\.me/|discord\.li/|discord\.io/)([a-zA-Z0-9\-]+)""")

private fun extractMediaUrls(content: String?): List<String> {
    if (content == null) return emptyList()
    if (!content.contains(".gif", ignoreCase = true) &&
        !content.contains(".mp4", ignoreCase = true) &&
        !content.contains(".webm", ignoreCase = true) &&
        !content.contains(".mov", ignoreCase = true)
    ) {
        return emptyList()
    }
    return mediaUrlPattern.findAll(content)
        .map { match -> match.value.trimEnd('.', ',', '!', '?', ';', ':') }
        .distinct()
        .toList()
}

private fun String.isGifUrl(): Boolean {
    val path = substringBefore('?').substringBefore('#')
    return path.endsWith(".gif", ignoreCase = true)
}

private fun String.normalizedMediaUrl(): String = trimEnd('.', ',', '!', '?', ';', ':')
