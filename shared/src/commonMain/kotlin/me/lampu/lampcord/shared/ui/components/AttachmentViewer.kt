package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.carousel.HorizontalUncontainedCarousel
import androidx.compose.material3.carousel.rememberCarouselState
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Attachment
import me.lampu.lampcord.shared.model.DiscordMedia
import me.lampu.lampcord.shared.model.EmbedVideo
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.downloadToDownloads
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@Composable
fun AttachmentViewer(
    items: List<DiscordMedia>,
    selectedIndex: Int,
    onIndexChange: (Int) -> Unit,
    onDismiss: () -> Unit
) {
    if (items.isEmpty()) return
    PlatformBackHandler(onBack = onDismiss)

    val index = selectedIndex.coerceIn(0, items.lastIndex)
    val item = items[index]
    val reduceMotion = Settings.shared.reduceMotion

    var showControls by remember { mutableStateOf(true) }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) { focusRequester.requestFocus() }
    LaunchedEffect(index) { showControls = true }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .navigationBarsPadding()
            .focusRequester(focusRequester)
            .focusable()
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown) {
                    when (event.key) {
                        Key.Escape -> {
                            onDismiss()
                            true
                        }
                        Key.DirectionLeft -> {
                            if (index > 0) onIndexChange(index - 1)
                            true
                        }
                        Key.DirectionRight -> {
                            if (index < items.lastIndex) onIndexChange(index + 1)
                            true
                        }
                        else -> false
                    }
                } else {
                    false
                }
            }
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            when {
                item.isVideo() -> {
                    val isGifv = item.isGifv()
                    VideoPlayer(
                        url = item.url ?: item.proxy_url ?: "",
                        loop = isGifv && !reduceMotion,
                        showControls = !isGifv || reduceMotion,
                        showSeekBar = !isGifv,
                        autoPlay = !isGifv || !reduceMotion,
                        title = (item as? Attachment)?.filename,
                        subtitle = (item as? Attachment)?.content_type,
                        onFullscreenClick = if (isGifv) null else onDismiss,
                        modifier = Modifier
                            .fillMaxSize()
                            .pointerInput(Unit) {
                                detectTapGestures(onTap = { showControls = !showControls })
                            }
                    )
                }
                item.isImage() -> {
                    ZoomableImageView(
                        url = item.proxy_url ?: item.url ?: "",
                        placeholderHash = item.placeholder,
                        modifier = Modifier.fillMaxSize(),
                        onSingleTap = { showControls = !showControls }
                    )
                }
                else -> {
                    Text(
                        text = (item as? Attachment)?.filename ?: "Unsupported attachment type",
                        color = Color.White
                    )
                }
            }
        }

        AnimatedVisibility(
            visible = showControls,
            enter = if (reduceMotion) EnterTransition.None else fadeIn(),
            exit = if (reduceMotion) ExitTransition.None else fadeOut(),
            modifier = Modifier.align(Alignment.TopCenter)
        ) {
            Surface(
                modifier = Modifier.fillMaxWidth().height(80.dp),
                color = Color.Black.copy(alpha = 0.5f)
            ) {
                Row(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp).statusBarsPadding(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = (item as? Attachment)?.filename ?: "Image",
                            style = MaterialTheme.typography.titleMedium,
                            color = Color.White,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                        Text(
                            text = (item as? Attachment)?.content_type ?: "image/png",
                            style = MaterialTheme.typography.labelSmall,
                            color = Color.White.copy(alpha = 0.7f)
                        )
                    }
                    
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        val scope = rememberCoroutineScope()
                        ViewerRoundButton(onClick = {
                            scope.launch {
                                val url = item.url ?: item.proxy_url ?: ""
                                val filename = (item as? Attachment)?.filename ?: "image.png"
                                val ok = downloadToDownloads(url, filename)
                                if (ok) showToast("Saved to Downloads") else showToast("Download failed")
                            }
                        }) {
                            Icon(Icons.Filled.Download, "Download", tint = Color.White, modifier = Modifier.size(20.dp))
                        }
                        ViewerRoundButton(onClick = {
                            setClipboardText(item.url ?: item.proxy_url ?: "")
                        }) {
                            Icon(Icons.Filled.ContentCopy, "Copy URL", tint = Color.White, modifier = Modifier.size(20.dp))
                        }
                        ViewerRoundButton(onClick = onDismiss) {
                            Icon(Icons.Filled.Close, "Close", tint = Color.White, modifier = Modifier.size(20.dp))
                        }
                    }
                }
            }
        }

        if (items.size > 1) {
            AnimatedVisibility(
                visible = showControls,
                enter = if (reduceMotion) EnterTransition.None else fadeIn(),
                exit = if (reduceMotion) ExitTransition.None else fadeOut(),
                modifier = Modifier.align(Alignment.CenterStart).padding(start = 16.dp)
            ) {
                ViewerRoundButton(onClick = { if (index > 0) onIndexChange(index - 1) }, enabled = index > 0) {
                    Icon(Icons.Filled.ChevronLeft, "Previous", tint = Color.White, modifier = Modifier.size(28.dp))
                }
            }
            AnimatedVisibility(
                visible = showControls,
                enter = if (reduceMotion) EnterTransition.None else fadeIn(),
                exit = if (reduceMotion) ExitTransition.None else fadeOut(),
                modifier = Modifier.align(Alignment.CenterEnd).padding(end = 16.dp)
            ) {
                ViewerRoundButton(
                    onClick = { if (index < items.lastIndex) onIndexChange(index + 1) },
                    enabled = index < items.lastIndex
                ) {
                    Icon(Icons.Filled.ChevronRight, "Next", tint = Color.White, modifier = Modifier.size(28.dp))
                }
            }
        }

        if (items.size > 1) {
            AnimatedVisibility(
                visible = showControls,
                enter = if (reduceMotion) EnterTransition.None else fadeIn(),
                exit = if (reduceMotion) ExitTransition.None else fadeOut(),
                modifier = Modifier.align(Alignment.BottomCenter)
            ) {
                AttachmentCarousel(items = items, selectedIndex = index, onSelect = onIndexChange)
            }
        }
    }
}

@Composable
private fun ViewerRoundButton(
    onClick: () -> Unit,
    enabled: Boolean = true,
    content: @Composable () -> Unit
) {
    Surface(
        shape = CircleShape,
        color = Color.Black.copy(alpha = 0.6f),
        modifier = Modifier.size(44.dp)
    ) {
        IconButton(onClick = onClick, enabled = enabled) {
            content()
        }
    }
}

@Composable
private fun ZoomableImageView(
    url: String,
    placeholderHash: String? = null,
    modifier: Modifier = Modifier,
    onSingleTap: (() -> Unit)? = null
) {
    var scale by remember(url) { mutableFloatStateOf(1f) }
    var offset by remember(url) { mutableStateOf(Offset.Zero) }
    
    val sizeState = remember { mutableStateOf(androidx.compose.ui.unit.IntSize.Zero) }

    Box(
        modifier = modifier
            .clipToBounds()
            .onGloballyPositioned {
                sizeState.value = it.size
            }
            .pointerInput(url) {
                detectTransformGestures { _, pan, zoom, _ ->
                    val newScale = (scale * zoom).coerceIn(1f, 8f)
                    
                    if (newScale != scale) {
                        offset *= (newScale / scale)
                    }
                    
                    val maxX = (sizeState.value.width * (newScale - 1f)) / 2f
                    val maxY = (sizeState.value.height * (newScale - 1f)) / 2f
                    
                    val newOffset = offset + pan
                    offset = Offset(
                        newOffset.x.coerceIn(-maxX, maxX),
                        newOffset.y.coerceIn(-maxY, maxY)
                    )

                    scale = newScale
                }
            }
            .pointerInput(url) {
                detectTapGestures(
                    onDoubleTap = {
                        if (scale > 1f) {
                            scale = 1f
                            offset = Offset.Zero
                        } else {
                            scale = 3f
                            offset = Offset.Zero
                        }
                    },
                    onTap = { onSingleTap?.invoke() }
                )
            },
        contentAlignment = Alignment.Center
    ) {
        AsyncImage(
            model = url,
            contentDescription = null,
            placeholderHash = placeholderHash,
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer {
                    scaleX = scale
                    scaleY = scale
                    translationX = offset.x
                    translationY = offset.y
                },
            contentScale = ContentScale.Fit
        )
    }
}

@Composable
private fun AttachmentCarousel(
    items: List<DiscordMedia>,
    selectedIndex: Int,
    onSelect: (Int) -> Unit
) {
    val carouselState = rememberCarouselState(
        initialItem = selectedIndex,
        itemCount = { items.size }
    )
    LaunchedEffect(selectedIndex) {
        val target = selectedIndex.coerceIn(0, items.lastIndex)
        if (!Settings.shared.reduceMotion) carouselState.animateScrollToItem(target) else carouselState.scrollToItem(target)
    }
    HorizontalUncontainedCarousel(
        state = carouselState,
        itemWidth = 57.dp,
        itemSpacing = 8.dp,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 16.dp)
            .height(57.dp),
        contentPadding = PaddingValues(horizontal = 16.dp)
    ) { index ->
        val media = items[index]
        val thumbUrl = media.thumbnailUrl(isPoster = true)
        Box(
            modifier = Modifier
                .fillMaxSize()
                .maskClip(RoundedCornerShape(4.dp))
                .background(Color.White.copy(alpha = 0.1f))
                .clickableCursor { onSelect(index) }
        ) {
            if (thumbUrl != null) {
                AsyncImage(
                    model = thumbUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    showPlaceholder = false,
                    placeholderHash = media.placeholder
                )
            }
            if (index == selectedIndex) {
                Box(
                    modifier = Modifier
                        .matchParentSize()
                        .maskBorder(BorderStroke(2.dp, Color.White), RoundedCornerShape(4.dp))
                )
            }
        }
    }
}

private fun DiscordMedia.thumbnailUrl(isPoster: Boolean): String? {
    val url = proxy_url ?: url ?: return null
    if (!isPoster || !isVideo()) return url
    return if (url.contains("format=")) url else "$url${if (url.contains("?")) "&" else "?"}format=png"
}
