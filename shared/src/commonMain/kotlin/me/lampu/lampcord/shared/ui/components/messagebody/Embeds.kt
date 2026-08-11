package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Embed
import me.lampu.lampcord.shared.model.EmbedVideo
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.VideoPlayer
import org.koin.compose.koinInject

@Composable
fun GifvView(video: EmbedVideo, modifier: Modifier = Modifier) {
    Box(modifier = modifier.clip(RoundedCornerShape(8.dp)).background(Color.Black)) {
        VideoPlayer(url = video.url ?: "", modifier = Modifier.fillMaxSize())
    }
}

@Composable
fun EmbedView(
    embed: Embed,
    navigationStore: NavigationStore = koinInject()
) {
    if (embed.type == "gifv" && embed.video != null) {
        GifvView(
            video = embed.video,
            modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 500.dp).fillMaxWidth().aspectRatio(embed.video.aspectRatio ?: 1f)
        )
        return
    }

    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 425.dp).fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(4.dp),
    ) {
        Row(modifier = Modifier.fillMaxWidth().height(IntrinsicSize.Min)) {
            val color = embed.color?.let { Color(it or 0xFF000000.toInt()) } ?: MaterialTheme.colorScheme.outlineVariant
            Box(modifier = Modifier.width(4.dp).fillMaxHeight().background(color))
            Column(modifier = Modifier.padding(8.dp).weight(1f)) {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
                    Column(modifier = Modifier.weight(1f)) {
                        embed.author?.let { author ->
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                val iconUrl = author.proxy_icon_url ?: author.icon_url
                                if (iconUrl != null) {
                                    AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.size(20.dp).clip(CircleShape))
                                    Spacer(Modifier.width(8.dp))
                                }
                                Text(text = author.name, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                            }
                            Spacer(Modifier.height(8.dp))
                        }
                        embed.title?.let { title ->
                            val titleText = buildAnnotatedString { withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)) { append(title) } }
                            if (embed.url != null) {
                                val uriHandler = LocalUriHandler.current
                                Text(text = titleText, style = MaterialTheme.typography.titleMedium, modifier = Modifier.clickable { uriHandler.openUri(embed.url) })
                            } else {
                                Text(text = title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                            }
                            Spacer(Modifier.height(8.dp))
                        }
                        embed.description?.let { desc ->
                            DiscordMarkdownText(content = desc, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(8.dp))
                        }
                    }
                    embed.thumbnail?.let { thumb ->
                        if (embed.image == null) {
                            Box(modifier = Modifier.padding(start = 8.dp).size(72.dp).clip(RoundedCornerShape(6.dp))) {
                                AttachmentImage(
                                    media = thumb,
                                    isMosaic = true,
                                    onClick = { navigationStore.openAttachmentViewer(listOf(thumb), 0) }
                                )
                            }
                        }
                    }
                }
                if (!embed.fields.isNullOrEmpty()) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        embed.fields.chunked(if (embed.fields.any { it.inline }) 3 else 1).forEach { rowFields ->
                            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                                rowFields.forEach { field ->
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(field.name, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold)
                                        DiscordMarkdownText(field.value, style = MaterialTheme.typography.bodySmall)
                                    }
                                }
                            }
                        }
                    }
                    Spacer(Modifier.height(8.dp))
                }
                embed.image?.let { image ->
                    AttachmentImage(
                        media = image,
                        onClick = { navigationStore.openAttachmentViewer(listOf(image), 0) }
                    )
                    Spacer(Modifier.height(8.dp))
                }
                embed.footer?.let { footer ->
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        val iconUrl = footer.proxy_icon_url ?: footer.icon_url
                        if (iconUrl != null) {
                            AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.size(16.dp).clip(CircleShape))
                            Spacer(Modifier.width(8.dp))
                        }
                        Text(text = footer.text, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}
