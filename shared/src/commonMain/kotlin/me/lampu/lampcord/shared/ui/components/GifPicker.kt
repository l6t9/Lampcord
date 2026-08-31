package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage as CoilAsyncImage
import coil3.compose.AsyncImagePainter
import coil3.compose.LocalPlatformContext
import coil3.request.ImageRequest
import coil3.request.crossfade
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GifCategory
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun GifPicker(
    query: String,
    onQueryChange: (String) -> Unit,
    mediaApi: MediaApi,
    onGifSelected: (Gif) -> Unit
) {
    var gifResults by remember { mutableStateOf<List<Gif>>(emptyList()) }
    var categories by remember { mutableStateOf<List<GifCategory>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var isCategoryMode by remember { mutableStateOf(false) }
    val reduceMotion = Settings.shared.reduceMotion

    LaunchedEffect(query, reduceMotion) {
        isLoading = true
        if (query.isBlank()) {
            categories = mediaApi.getTrendingGifCategories()?.categories ?: emptyList()
            gifResults = emptyList()
            isCategoryMode = false
        } else if (isCategoryMode) {
            gifResults = mediaApi.getTrendingGifCategory(query)
            categories = emptyList()
        } else {
            delay(500.milliseconds)
            gifResults = mediaApi.searchGifs(query)
            categories = emptyList()
            isCategoryMode = false
        }
        isLoading = false
    }

    if (isLoading) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            ContainedLoadingIndicator()
        }
    } else {
        if (query.isBlank()) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(
                    items = categories,
                    key = { category -> category.src.ifBlank { category.name } }
                ) { category ->
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(1.8f)
                            .clip(RoundedCornerShape(16.dp))
                    ) {
                        val openCategory = {
                            isCategoryMode = true
                            onQueryChange(category.name)
                        }
                        GifPreviewImage(
                            candidates = if (reduceMotion) {
                                staticImageCandidates(category.src)
                            } else {
                                listOf(category.src).filter(String::isNotBlank)
                            },
                            contentDescription = category.name,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop,
                            animated = !reduceMotion
                        )
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(Color.Black.copy(alpha = 0.3f)),
                            contentAlignment = Alignment.BottomStart
                        ) {
                            Text(
                                text = category.name,
                                color = Color.White,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(8.dp),
                                style = MaterialTheme.typography.labelLarge
                            )
                        }
                        Box(Modifier.matchParentSize().clickable(onClick = openCategory))
                    }
                }
            }
        } else {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                items(
                    items = gifResults,
                    // Keep the painter attached to the GIF rather than to a
                    // recycled grid slot or the current scroll index.
                    key = { gif ->
                        "${gif.url}|${gif.src}|${gif.gifSrc}|${gif.preview}"
                    }
                ) { gif ->
                    GifThumbnail(gif = gif, onGifSelected = onGifSelected)
                }
            }
        }
    }
}

@Composable
private fun GifThumbnail(
    gif: Gif,
    onGifSelected: (Gif) -> Unit,
) {
    val context = LocalPlatformContext.current
    val reduceMotion = Settings.shared.reduceMotion
    val candidates = remember(gif, reduceMotion) {
        if (reduceMotion) {
            // preview is Klipy's dedicated still thumbnail. Keep gif_src as
            // a compatibility fallback only when a result has no preview.
            staticImageCandidates(gif.preview, gif.gifSrc, gif.src, gif.url)
        } else {
            (listOfNotNull(gif.gifSrc) + listOfNotNull(gif.src, gif.preview, gif.url))
                .filter(String::isNotBlank)
                .distinct()
        }
    }
    // Switching Reduced Motion changes the available preview URLs. Reset the
    // fallback index too, otherwise a previous failed animated URL can leave
    // the new, static candidate list permanently out of bounds.
    var candidateIndex by remember(gif, reduceMotion) { mutableIntStateOf(0) }
    val imageUrl = candidates.getOrNull(candidateIndex)

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(1.5f)
            .clip(RoundedCornerShape(16.dp))
    ) {
        if (imageUrl != null) {
            if (getPlatformName() == "windows" && !reduceMotion) {
                VideoPlayer(
                    url = imageUrl,
                    modifier = Modifier.fillMaxSize(),
                    loop = true,
                    showControls = false,
                    compact = true,
                    autoPlay = true
                )
            } else {
                CoilAsyncImage(
                    model = ImageRequest.Builder(context)
                        .data(imageUrl)
                        .memoryCacheKey("gif-preview:$imageUrl")
                        .crossfade(false)
                        .build(),
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    onState = { state ->
                        if (state is AsyncImagePainter.State.Error && candidateIndex < candidates.lastIndex) {
                            candidateIndex++
                        }
                    }
                )
            }
        }
        // Keep selection reliable even when a native/desktop media renderer
        // consumes pointer input itself.
        Box(Modifier.matchParentSize().clickable { onGifSelected(gif) })
    }
}

@Composable
private fun GifPreviewImage(
    candidates: List<String>,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    animated: Boolean,
) {
    val context = LocalPlatformContext.current
    var candidateIndex by remember(candidates) { mutableIntStateOf(0) }
    val imageUrl = candidates.getOrNull(candidateIndex)

    if (imageUrl != null && animated && getPlatformName() == "windows") {
        VideoPlayer(
            url = imageUrl,
            modifier = modifier,
            loop = true,
            showControls = false,
            compact = true,
            autoPlay = true
        )
    } else if (imageUrl != null) {
        CoilAsyncImage(
            model = ImageRequest.Builder(context)
                .data(imageUrl)
                .memoryCacheKey("gif-preview:$imageUrl")
                .crossfade(false)
                .build(),
            contentDescription = contentDescription,
            modifier = modifier,
            contentScale = contentScale,
            onState = { state ->
                if (state is AsyncImagePainter.State.Error && candidateIndex < candidates.lastIndex) {
                    candidateIndex++
                }
            }
        )
    }
}

private fun staticImageCandidates(vararg urls: String?): List<String> {
    val primary = urls
        .filterNotNull()
        .map(String::trim)
        .filter(String::isNotBlank)
        .distinct()
    if (primary.isEmpty()) return emptyList()

    val staticPrimary = primary.map { url ->
        url.replace(
            Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE),
            "${'$'}1false"
        )
    }.filter { url ->
        !url.substringBefore('?').endsWith(".gif", ignoreCase = true)
    }.distinct()

    val variants = primary.flatMap { url ->
        buildList {
            val staticUrl = url.replace(
                Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE),
                "${'$'}1false"
            )
            val isGifPath = staticUrl.substringBefore('?').endsWith(".gif", ignoreCase = true)
            if (staticUrl != url && !isGifPath) add(staticUrl)

            // Several provider CDNs expose the same asset as .webp when the
            // requested media format is ignored by the gateway.
            val gifSuffix = staticUrl.indexOf(".gif", ignoreCase = true)
            if (gifSuffix >= 0) {
                val suffixEnd = gifSuffix + 4
                if (suffixEnd == staticUrl.length || staticUrl[suffixEnd] == '?') {
                    add(staticUrl.substring(0, gifSuffix) + ".png" + staticUrl.substring(suffixEnd))
                    add(staticUrl.substring(0, gifSuffix) + ".webp" + staticUrl.substring(suffixEnd))
                }
            }

            if (!isGifPath && !staticUrl.contains("format=", ignoreCase = true)) {
                val separator = if (staticUrl.contains("?")) "&" else "?"
                add("${staticUrl}${separator}format=png")
                add("${staticUrl}${separator}format=webp")
            }
        }
    }
    // Klipy's first source is its known-good preview. Try it before derived
    // CDN variants: those variants are not supported by every provider and
    // caused reduced-motion searches to render as empty tiles.
    return (staticPrimary + variants).distinct()
}
