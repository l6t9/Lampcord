package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import me.lampu.lampcord.shared.ui.rememberLinkOpener
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Embed
import me.lampu.lampcord.shared.model.EmbedImage
import me.lampu.lampcord.shared.model.EmbedVideo
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.VideoPlayer
import me.lampu.lampcord.shared.settings.Settings
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.utils.getPlatformName
import dev.nucleusframework.webview.web.WebView
import dev.nucleusframework.webview.web.rememberWebViewNavigator
import dev.nucleusframework.webview.web.rememberWebViewState
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

private val spotifyUrlRe = Regex("https://open\\.spotify\\.com/(\\w+)/(\\w+)")
private val youtubeUrlRe =
    Regex(
        "(?:https?://)?(?:(?:www|m)\\.)?(?:youtu\\.be/|youtube(?:-nocookie)?\\.com/" +
                "(?:embed/|v/|watch\\?v=|watch\\?.+&v=|shorts/))((\\w|-){11})" +
                "(?:(?:\\?|&)(?:star)?t=(\\d+))?(?:\\S+)?"
    )
private val youtubeClipRe =
    Regex(
        "(?:https?://)?(?:(?:www|m)\\.)?(?:youtu\\.be/|youtube(?:-nocookie)?\\.com/clip/)" +
                "((\\w|-){36})(?:(?:\\?|&)(?:star)?t=(\\d+))?(?:\\S+)?"
    )

@Composable
fun GifvView(
    video: EmbedVideo,
    modifier: Modifier = Modifier,
    onFullscreenClick: (() -> Unit)? = null
) {
    Box(modifier = modifier.clip(RoundedCornerShape(8.dp)).background(Color.Black)) {
        VideoPlayer(
            url = video.url ?: "",
            loop = !Settings.shared.reduceMotion,
            showControls = Settings.shared.reduceMotion,
            showSeekBar = false,
            compact = true,
            autoPlay = !Settings.shared.reduceMotion,
            onFullscreenClick = null,
            modifier = Modifier.fillMaxSize()
        )
    }
}

@Composable
fun PlayableEmbedView(
    url: String,
    provider: String,
    modifier: Modifier = Modifier
) {
    val navigator = rememberWebViewNavigator()
    val webViewState = rememberWebViewState(url)
    
    LaunchedEffect(webViewState) {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        webViewState.webSettings.apply {
            isJavaScriptEnabled = true
            customUserAgentString = if (isMobile) {
                "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36"
            } else {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
            }
        }
        
        // "webview refer fix" - spoofing Referer to avoid embed restrictions
        navigator.loadUrl(url, mapOf("Referer" to "https://discord.com"))
    }
    
    Box(modifier = modifier.clip(RoundedCornerShape(8.dp)).background(Color.Black)) {
        WebView(
            state = webViewState,
            navigator = navigator,
            modifier = Modifier.fillMaxSize()
        )
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
            onFullscreenClick = {
                navigationStore.openAttachmentViewer(listOf(embed.video), 0)
            },
            modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 500.dp).fillMaxWidth().aspectRatio(embed.video.aspectRatio ?: 1f)
        )
        return
    }

    val playableUrl = remember(embed) {
        val embedUrl = embed.url ?: return@remember null
        when (embed.provider?.name) {
            "YouTube" -> {
                youtubeUrlRe.find(embedUrl)?.let { res ->
                    val videoId = res.groupValues[1]
                    val timestamp = res.groupValues[3].takeIf { it.isNotBlank() }
                    "https://www.youtube-nocookie.com/embed/$videoId${if (timestamp != null) "?start=$timestamp" else ""}"
                } ?: youtubeClipRe.find(embedUrl)?.let { res ->
                    val clipId = res.groupValues[1]
                    "https://www.youtube-nocookie.com/clip/$clipId"
                }
            }
            "Spotify" -> {
                spotifyUrlRe.find(embedUrl)?.let { res ->
                    val type = res.groupValues[1]
                    val itemId = res.groupValues[2]
                    "https://open.spotify.com/embed/$type/$itemId"
                }
            }
            else -> {
                if (embed.video != null) embedUrl else null
            }
        }
    }

    val gifUrl = listOfNotNull(
        embed.image?.url,
        embed.image?.proxy_url,
        embed.url
    ).firstOrNull { it.isGifUrl() }
    if (gifUrl != null) {
        val gifImage = embed.image?.copy(url = gifUrl, proxy_url = gifUrl)
            ?: EmbedImage(url = gifUrl, proxy_url = gifUrl)
        AttachmentImage(
            media = gifImage,
            onClick = { navigationStore.openAttachmentViewer(listOf(gifImage), 0) },
            modifier = Modifier
                .padding(vertical = 4.dp)
                .widthIn(max = 400.dp)
                .fillMaxWidth()
                .aspectRatio(gifImage.aspectRatio ?: 1f)
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
    val openLink = rememberLinkOpener()
                                Text(text = titleText, style = MaterialTheme.typography.titleMedium, modifier = Modifier.clickableCursor { openLink(embed.url) })
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
                        if (embed.image == null && playableUrl == null) {
                            Box(modifier = Modifier.padding(start = 8.dp).size(72.dp).clip(RoundedCornerShape(6.dp))) {
                                AttachmentImage(
                                    media = thumb,
                                    isMosaic = true,
                                    title = embed.title,
                                    subtitle = embed.provider?.name,
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
                if (playableUrl != null) {
                    PlayableEmbedView(
                        url = playableUrl,
                        provider = embed.provider?.name ?: "",
                        modifier = Modifier
                            .padding(vertical = 4.dp)
                            .fillMaxWidth()
                            .aspectRatio(if (embed.provider?.name == "Spotify") 1.8f else 1.77f)
                            .heightIn(max = if (embed.provider?.name == "Spotify") 152.dp else 400.dp)
                    )
                    Spacer(Modifier.height(8.dp))
                }
                embed.image?.let { image ->
                    AttachmentImage(
                        media = image,
                        title = embed.title,
                        subtitle = embed.provider?.name,
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

private fun String?.isGifUrl(): Boolean {
    val path = this?.substringBefore('?')?.substringBefore('#') ?: return false
    return path.endsWith(".gif", ignoreCase = true)
}
