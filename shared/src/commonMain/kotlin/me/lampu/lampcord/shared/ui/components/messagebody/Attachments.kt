package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.rememberCoroutineScope
import kotlinx.coroutines.launch
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
import me.lampu.lampcord.shared.utils.SnackbarManager
import me.lampu.lampcord.shared.utils.openDownloadsFolderAndSelect
import me.lampu.lampcord.shared.utils.sanitizeFilename
import me.lampu.lampcord.shared.utils.ensureUniqueDownloadFilename
import me.lampu.lampcord.shared.utils.downloadToDownloads
import me.lampu.lampcord.shared.utils.showToast

@Composable
fun AttachmentImage(
    media: DiscordMedia,
    modifier: Modifier = Modifier,
    isMosaic: Boolean = false,
    title: String? = null,
    subtitle: String? = null,
    onClick: (() -> Unit)? = null
) {
    val isVideo = media.isVideo()
    val isGifv = media.isGifv()
    var isInlinePlaying by remember { mutableStateOf(isGifv) }
    
    val url = media.proxy_url ?: media.url ?: ""
    
    // Use the proxy URL as-is for images,
    // only append format=png for video posters. No width/height resizing.
    val displayUrl = remember(url, isVideo) {
        var result = url
        if (isVideo && result.isNotEmpty() && !result.contains("format=")) {
            val sep = if (result.contains("?")) "&" else "?"
            result = "$result${sep}format=png"
        }
        result
    }

    val interactionSource = remember { MutableInteractionSource() }
    val clickModifier = if (!isInlinePlaying && onClick != null) {
        Modifier.clickable(interactionSource = interactionSource, indication = null) { 
            if (isVideo && !isGifv) {
                isInlinePlaying = true
            } else {
                onClick()
            }
        }
    } else {
        Modifier
    }

    if (isInlinePlaying && isVideo) {
        val maxWidth = 500.dp
        val maxHeight = 300.dp
        val finalWidth = (media.width?.dp ?: maxWidth).coerceAtMost(maxWidth)
        val finalHeight = (media.height?.dp ?: maxHeight).coerceAtMost(maxHeight)

        VideoPlayer(
            url = url,
            loop = isGifv,
            showControls = !isGifv,
            title = title ?: (media as? Attachment)?.filename,
            subtitle = subtitle ?: (media as? Attachment)?.content_type,
            compact = true,
            onFullscreenClick = if (!isGifv) onClick else null,
            modifier = modifier
                .sizeIn(maxWidth = finalWidth, maxHeight = finalHeight)
                .aspectRatio(media.aspectRatio ?: (16f / 9f))
                .clip(RoundedCornerShape(8.dp))
        )
    } else if (isMosaic) {
        Box(modifier = modifier.fillMaxSize().then(clickModifier), contentAlignment = Alignment.Center) {
            AsyncImage(
                model = displayUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                placeholderHash = media.placeholder
            )
            if (isVideo && !isGifv) {
                Surface(
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.primaryContainer,
                    modifier = Modifier.size(48.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.size(24.dp)
                        )
                    }
                }
            }
        }
    } else {
        val maxWidth = 500.dp
        val maxHeight = 300.dp

        // Cap the box at
        // min(500, imageWidth) x min(300, imageHeight) and let the
        // aspect ratio decide the final size, so tall images (e.g. phone
        // screenshots) are shown in full and never cropped.
        val finalWidth = (media.width?.dp ?: maxWidth).coerceAtMost(maxWidth)
        val finalHeight = (media.height?.dp ?: maxHeight).coerceAtMost(maxHeight)

        Box(
            modifier = modifier
                .then(clickModifier)
                .sizeIn(maxWidth = finalWidth, maxHeight = finalHeight)
                .aspectRatio(media.aspectRatio ?: 1f)
                .clip(RoundedCornerShape(8.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLow),
            contentAlignment = Alignment.Center
        ) {
            AsyncImage(
                model = displayUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Fit,
                placeholderHash = media.placeholder
            )
            if (isVideo && !isGifv) {
                Surface(
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.primaryContainer,
                    modifier = Modifier.size(48.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Filled.PlayArrow,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.size(24.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun MessageMosaic(items: List<DiscordMedia>, onOpenItem: ((Int) -> Unit)? = null) {
    val spacing = 4.dp
    val maxWidth = 500.dp
    val rowLayouts = mapOf(2 to listOf(2), 4 to listOf(2, 2), 5 to listOf(2, 3), 6 to listOf(3, 3), 8 to listOf(4, 4), 9 to listOf(3, 3, 3))

    Box(modifier = Modifier.widthIn(max = maxWidth).clip(RoundedCornerShape(8.dp))) {
        when {
            items.isEmpty() -> {}
            items.size == 1 -> { AttachmentImage(media = items[0], onClick = { onOpenItem?.invoke(0) }) }
            items.size == 3 -> {
                Row(modifier = Modifier.fillMaxWidth().aspectRatio(1.5f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.weight(1f).fillMaxHeight().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true, onClick = { onOpenItem?.invoke(0) }) }
                    Column(modifier = Modifier.weight(1f).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(spacing)) {
                        Box(Modifier.weight(1f).fillMaxWidth().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[1], isMosaic = true, onClick = { onOpenItem?.invoke(1) }) }
                        Box(Modifier.weight(1f).fillMaxWidth().clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[2], isMosaic = true, onClick = { onOpenItem?.invoke(2) }) }
                    }
                }
            }
            items.size == 7 -> {
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.fillMaxWidth().aspectRatio(2.0f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true, onClick = { onOpenItem?.invoke(0) }) }
                    repeat(2) { row ->
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            for (i in 0..2) {
                                val idx = 1 + row * 3 + i
                                Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[idx], isMosaic = true, onClick = { onOpenItem?.invoke(idx) }) }
                            }
                        }
                    }
                }
            }
            items.size == 10 -> {
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Box(Modifier.fillMaxWidth().aspectRatio(2.0f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[0], isMosaic = true, onClick = { onOpenItem?.invoke(0) }) }
                    repeat(3) { row ->
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            for (i in 0..2) {
                                val idx = 1 + row * 3 + i
                                Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[idx], isMosaic = true, onClick = { onOpenItem?.invoke(idx) }) }
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
                                    val idx = currentIdx
                                    Box(Modifier.weight(1f).aspectRatio(1.04f).clip(RoundedCornerShape(4.dp))) { AttachmentImage(media = items[idx], isMosaic = true, onClick = { onOpenItem?.invoke(idx) }) }
                                    currentIdx++
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
                    val scope = rememberCoroutineScope()
                    var isDownloading by remember { mutableStateOf(false) }
            IconButton(onClick = {
                        scope.launch {
                            isDownloading = true
                            val raw = attachment.filename ?: "download"
                            val sanitized = sanitizeFilename(raw)
                            val filename = ensureUniqueDownloadFilename(sanitized)
                            val ok = downloadToDownloads(attachment.url, filename)
                            isDownloading = false
                            if (ok) {
                                // show in-app snackbar with Open action
                                try {
                                    SnackbarManager.show("Downloaded $filename to Downloads", "Open") { openDownloadsFolderAndSelect(filename) }
                                } catch (e: Exception) {
                                    showToast("Downloaded $filename to Downloads")
                                }
                            } else showToast("Download failed")
                        }
            }) {
                        if (isDownloading) {
                            CircularProgressIndicator(modifier = Modifier.size(20.dp), strokeWidth = 2.dp)
                        } else {
                            Icon(imageVector = Icons.Filled.Download, contentDescription = "Download", tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
            }
        }
    }
}
