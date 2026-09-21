package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.*
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
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
import com.github.panpf.sketch.AsyncImage as SketchAsyncImage
import com.github.panpf.sketch.LocalPlatformContext
import com.github.panpf.sketch.PainterState
import com.github.panpf.sketch.cache.CachePolicy
import com.github.panpf.sketch.request.ImageRequest
import com.github.panpf.sketch.rememberAsyncImageState
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GifCategory
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun GifPicker(
    query: String,
    onQueryChange: (String) -> Unit,
    mediaApi: MediaApi,
    onGifSelected: (Gif) -> Unit,
    userApi: UserApi = koinInject()
) {
    var gifResults by remember { mutableStateOf<List<Gif>>(emptyList()) }
    var categories by remember { mutableStateOf<List<GifCategory>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var isCategoryMode by remember { mutableStateOf(false) }
    var showFavorites by remember { mutableStateOf(false) }
    var favoriteGifs by remember { mutableStateOf<List<Gif>>(emptyList()) }
    val reduceMotion = Settings.shared.reduceMotion

    LaunchedEffect(Unit) {
        favoriteGifs = userApi.getFavoriteGifs()
    }

    LaunchedEffect(showFavorites, favoriteGifs) {
        if (showFavorites && query.isBlank()) {
            gifResults = favoriteGifs
        }
    }

    LaunchedEffect(query, isCategoryMode, showFavorites) {
        if (showFavorites && query.isNotBlank()) {
            showFavorites = false
            return@LaunchedEffect
        }
        isLoading = true
        if (showFavorites) {
            gifResults = favoriteGifs
            categories = emptyList()
        } else if (query.isBlank()) {
            val response = mediaApi.getTrendingGifCategories()
            // Keep the last good category list if the CDN/API briefly returns
            // an empty response while coming back from a category.
            response?.categories
                ?.takeIf { it.isNotEmpty() }
                ?.let { categories = it }
            gifResults = emptyList()
            isCategoryMode = false
        } else if (isCategoryMode) {
            gifResults = mediaApi.getTrendingGifCategory(query)
        } else {
            delay(500.milliseconds)
            gifResults = mediaApi.searchGifs(query)
            isCategoryMode = false
        }
        isLoading = false
    }

    if (isLoading) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            ContainedLoadingIndicator()
        }
    } else {
        if (showFavorites || query.isNotBlank()) {
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
        } else {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                if (favoriteGifs.isNotEmpty()) {
                    item(key = "__favorite_gifs__") {
                        val favoritePreview = favoriteGifs.first()
                        GifCategoryTile(
                            name = "Favorites",
                            candidates = if (reduceMotion) {
                                gifStaticCandidates(favoritePreview)
                            } else {
                                gifAnimatedCandidates(favoritePreview)
                            },
                            animated = !reduceMotion,
                            forceVideo = favoritePreview.isVideo,
                            onClick = {
                                isCategoryMode = false
                                showFavorites = true
                                onQueryChange("")
                            }
                        )
                    }
                }
                items(
                    items = categories,
                    key = { category -> category.src.ifBlank { category.name } }
                ) { category ->
                    GifCategoryTile(
                        name = category.name,
                        candidates = if (reduceMotion) {
                            staticImageCandidates(category.src)
                        } else {
                            listOf(category.src)
                                .filter(String::isNotBlank)
                                .distinct()
                        },
                        animated = !reduceMotion,
                        onClick = {
                            isCategoryMode = true
                            showFavorites = false
                            onQueryChange(category.name)
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun GifCategoryTile(
    name: String,
    candidates: List<String>,
    animated: Boolean,
    forceVideo: Boolean = false,
    onClick: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(1.8f)
            .clip(RoundedCornerShape(16.dp))
    ) {
        GifPreviewImage(
            candidates = candidates,
            contentDescription = name,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            animated = animated,
            forceVideo = forceVideo
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.3f)),
            contentAlignment = Alignment.BottomStart
        ) {
            Text(
                text = name,
                color = Color.White,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(8.dp),
                style = MaterialTheme.typography.labelLarge
            )
        }
        // Keep the tile actionable when the animated preview is backed by a
        // native media view that consumes pointer input.
        Box(Modifier.matchParentSize().clickable(onClick = onClick))
    }
}

@Composable
private fun GifThumbnail(
    gif: Gif,
    onGifSelected: (Gif) -> Unit,
) {
    val context = LocalPlatformContext.current
    val reduceMotion = Settings.shared.reduceMotion
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val isDesktop = getPlatformName() == "windows"
    // A decoder per visible GIF makes the desktop picker stutter, especially
    // when a Favorites list contains many video-format entries. Keep the
    // desktop grid lightweight until a tile is hovered.
    val animatePreview = !reduceMotion && (!isDesktop || isHovered)
    val candidates = remember(gif, reduceMotion, animatePreview) {
        if (reduceMotion || (isDesktop && !animatePreview)) {
            gifStaticCandidates(gif)
        } else {
            gifAnimatedCandidates(gif)
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
            .hoverable(interactionSource)
    ) {
        if (imageUrl != null) {
            if (animatePreview && (isDesktop || gif.isVideo || isAnimatedMediaUrl(imageUrl))) {
                VideoPlayer(
                    url = imageUrl,
                    modifier = Modifier.fillMaxSize(),
                    loop = true,
                    showControls = false,
                    compact = true,
                    autoPlay = true
                )
            } else {
                val state = rememberAsyncImageState()
                state.onPainterState = { painterState ->
                    if (painterState is PainterState.Error && candidateIndex < candidates.lastIndex) {
                        candidateIndex++
                    }
                }
                SketchAsyncImage(
                    request = ImageRequest.Builder(context, imageUrl)
                        .memoryCacheKey("gif-preview:$imageUrl")
                        .memoryCachePolicy(
                            if (getPlatformName() != "android" && getPlatformName() != "ios" && Settings.shared.desktopLowMemoryMode) CachePolicy.DISABLED else CachePolicy.ENABLED
                        )
                        .crossfade(false)
                        .build(),
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    state = state
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
    forceVideo: Boolean = false,
) {
    val context = LocalPlatformContext.current
    var candidateIndex by remember(candidates) { mutableIntStateOf(0) }
    val imageUrl = candidates.getOrNull(candidateIndex)

    if (imageUrl != null && animated && (forceVideo || getPlatformName() == "windows" || isAnimatedMediaUrl(imageUrl))) {
        VideoPlayer(
            url = imageUrl,
            modifier = modifier,
            loop = true,
            showControls = false,
            compact = true,
            autoPlay = true
        )
    } else if (imageUrl != null) {
        val state = rememberAsyncImageState()
        state.onPainterState = { painterState ->
            if (painterState is PainterState.Error && candidateIndex < candidates.lastIndex) {
                candidateIndex++
            }
        }
        SketchAsyncImage(
            request = ImageRequest.Builder(context, imageUrl)
                .memoryCacheKey("gif-preview:$imageUrl")
                .memoryCachePolicy(
                    if (getPlatformName() != "android" && getPlatformName() != "ios" && Settings.shared.desktopLowMemoryMode) CachePolicy.DISABLED else CachePolicy.ENABLED
                )
                .crossfade(false)
                .build(),
            contentDescription = contentDescription,
            modifier = modifier,
            contentScale = contentScale,
            state = state
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
        !isAnimatedMediaUrl(url)
    }.distinct()

    val variants = primary.flatMap { url ->
        buildList {
            val staticUrl = url.replace(
                Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE),
                "${'$'}1false"
            )
            val isAnimatedMedia = isAnimatedMediaUrl(staticUrl)
            if (staticUrl != url && !isAnimatedMedia) add(staticUrl)

            // Several provider CDNs expose the same asset as .webp when the
            // requested media format is ignored by the gateway.
            val mediaSuffix = animatedMediaSuffix(staticUrl)
            if (mediaSuffix != null) {
                val suffixEnd = mediaSuffix.first + mediaSuffix.second.length
                // Tenor uses a size/format code in the path as well as the
                // file extension. `...AAAAd/*` is animated, while the
                // matching still preview is usually `...AAAAD/*.png`.
                tenorStaticVariant(staticUrl)?.let(::add)
                add(staticUrl.substring(0, mediaSuffix.first) + ".png" + staticUrl.substring(suffixEnd))
                add(staticUrl.substring(0, mediaSuffix.first) + ".webp" + staticUrl.substring(suffixEnd))
            }

            if (!isAnimatedMedia && !staticUrl.contains("format=", ignoreCase = true)) {
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

private fun gifStaticCandidates(gif: Gif): List<String> =
    staticImageCandidates(gif.preview, gif.gifSrc, gif.src)

private fun isAnimatedMediaUrl(url: String): Boolean =
    animatedMediaSuffix(url) != null

private fun animatedMediaSuffix(url: String): Pair<Int, String>? {
    val path = url.substringBefore('?').substringBefore('#')
    val suffix = listOf(".gif", ".webp", ".webm", ".mp4")
        .firstOrNull { path.endsWith(it, ignoreCase = true) }
        ?: return null
    return path.length - suffix.length to suffix
}

private fun gifAnimatedCandidates(gif: Gif): List<String> {
    // Desktop requests MP4 from Discord's GIF endpoint so FFmpeg can play
    // the preview reliably. Mobile keeps the GIF representation.
    val animatedSources = if (getPlatformName() == "windows") {
        listOfNotNull(gif.src, gif.gifSrc)
    } else {
        listOfNotNull(gif.gifSrc, gif.src)
    }
    return (animatedSources + listOfNotNull(gif.preview))
        .filter(String::isNotBlank)
        .distinct()
}

private fun tenorStaticVariant(url: String): String? {
    if (!url.contains("media.tenor.com", ignoreCase = true)) return null

    val authorityEnd = url.indexOf("//").takeIf { it >= 0 }?.let { start ->
        url.indexOf('/', start + 2)
    } ?: return null
    val idStart = authorityEnd + 1
    val idEnd = url.indexOf('/', idStart)
    if (idEnd <= idStart) return null

    val mediaId = url.substring(idStart, idEnd)
    if (mediaId.isEmpty() || !mediaId.last().isLetter()) return null

    return url.replaceRange(idStart, idEnd, mediaId.dropLast(1) + "D")
        .replace(Regex("\\.(gif|webp|webm|mp4)(?=($|[?#]))", RegexOption.IGNORE_CASE), ".png")
}
