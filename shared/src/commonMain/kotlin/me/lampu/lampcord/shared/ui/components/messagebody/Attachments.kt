package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.VideoPlayer
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun VideoAttachment(video: Attachment) {
    val vw = video.width
    val aspectRatio = video.aspectRatio ?: (16f / 9f)
    
    var wantsPlayback by remember { mutableStateOf(false) }

    Box(
        modifier = Modifier
            .widthIn(max = 500.dp)
            .fillMaxWidth(if (vw != null && vw < 500) vw.toFloat() / 500f else 1f)
            .aspectRatio(aspectRatio.coerceIn(0.5f, 2.5f))
            .clip(RoundedCornerShape(8.dp))
            .background(Color.Black)
    ) {
        if (!wantsPlayback) {
            AttachmentImage(
                media = video,
                modifier = Modifier.fillMaxSize(),
                needsPoster = true
            )
            
            Box(
                modifier = Modifier.fillMaxSize().clickable { wantsPlayback = true },
                contentAlignment = Alignment.Center
            ) {
                Surface(
                    shape = CircleShape,
                    color = Color.Black.copy(alpha = 0.4f),
                    modifier = Modifier.size(64.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = "Play",
                            tint = Color.White,
                            modifier = Modifier.size(40.dp).offset(x = 2.dp)
                        )
                    }
                }
            }
        } else {
            VideoPlayer(
                url = video.url, 
                modifier = Modifier.fillMaxSize()
            )
        }
    }
}

@Composable
fun AttachmentImage(
    media: DiscordMedia,
    modifier: Modifier = Modifier,
    isMosaic: Boolean = false,
    needsPoster: Boolean = false
) {
    val url = media.proxy_url ?: media.url ?: ""
    
    val displayUrl = remember(url, isMosaic, needsPoster) {
        var result = url
        if (url.contains("media.discordapp.net")) {
            if (needsPoster && !url.contains("format=")) {
                val sep = if (result.contains("?")) "&" else "?"
                result = "$result${sep}format=png"
            }
            if (!result.contains("width=")) {
                val sep = if (result.contains("?")) "&" else "?"
                result = if (isMosaic) "${result}${sep}width=600&height=600" 
                        else "${result}${sep}width=1200&height=1200"
            }
        }
        result
    }

    if (isMosaic) {
        AsyncImage(
            model = displayUrl,
            contentDescription = null,
            modifier = modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
        )
    } else {
        val maxWidth = 500.dp
        val maxHeight = 300.dp
        val width = media.width?.dp ?: maxWidth
        val height = media.height?.dp ?: maxHeight
        val finalWidth = width.coerceAtMost(maxWidth)
        val finalHeight = height.coerceAtMost(maxHeight)

        Box(
            modifier = modifier
                .sizeIn(maxWidth = finalWidth, maxHeight = finalHeight)
                .aspectRatio(media.aspectRatio ?: 1f)
                .clip(RoundedCornerShape(8.dp))
                .background(Color.Transparent)
        ) {
            AsyncImage(
                model = displayUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Fit,
            )
        }
    }
}

@Composable
fun MessageMosaic(items: List<DiscordMedia>) {
    val spacing = 4.dp
    val maxWidth = 500.dp
    val rowLayouts = mapOf(2 to listOf(2), 4 to listOf(2, 2), 5 to listOf(2, 3), 6 to listOf(3, 3), 8 to listOf(4, 4), 9 to listOf(3, 3, 3))

    Box(modifier = Modifier.widthIn(max = maxWidth).clip(RoundedCornerShape(8.dp))) {
        when {
            items.isEmpty() -> {}
            items.size == 1 -> { AttachmentImage(media = items[0]) }
            items.size == 3 -> {
                Row(modifier = Modifier.fillMaxWidth().aspectRatio(1.5f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.weight(1f).fillMaxHeight().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true) }
                    Column(modifier = Modifier.weight(1f).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(spacing)) {
                        Box(Modifier.weight(1f).fillMaxWidth().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[1], isMosaic = true) }
                        Box(Modifier.weight(1f).fillMaxWidth().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[2], isMosaic = true) }
                    }
                }
            }
            items.size == 7 -> {
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.fillMaxWidth().aspectRatio(2.0f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true) }
                    repeat(2) { row ->
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            for (i in 0..2) {
                                val idx = 1 + row * 3 + i
                                Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[idx], isMosaic = true) }
                            }
                        }
                    }
                }
            }
            items.size == 10 -> {
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.fillMaxWidth().aspectRatio(2.0f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true) }
                    repeat(3) { row ->
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            for (i in 0..2) {
                                val idx = 1 + row * 3 + i
                                Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[idx], isMosaic = true) }
                            }
                        }
                    }
                }
            }
            else -> {
                val rows = rowLayouts[items.size] ?: listOf(items.size)
                Column(modifier = Modifier.widthIn(max = maxWidth), verticalArrangement = Arrangement.spacedBy(spacing)) {
                    var currentIdx = 0
                    rows.forEach { count ->
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            for (i in 0 until count) {
                                if (currentIdx < items.size) {
                                    Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[currentIdx++], isMosaic = true) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun FileAttachmentView(attachment: Attachment) {
    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 400.dp).fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(8.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(imageVector = Icons.Filled.Description, contentDescription = null, modifier = Modifier.size(32.dp), tint = MaterialTheme.colorScheme.primary)
            Spacer(modifier = Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(text = attachment.filename, style = MaterialTheme.typography.titleSmall, maxLines = 1, overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis)
                Text(text = "${attachment.size / 1024} KB", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            val uriHandler = LocalUriHandler.current
            IconButton(onClick = { uriHandler.openUri(attachment.url) }) {
                Icon(imageVector = Icons.Filled.Download, contentDescription = "Download", tint = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
