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
import coil3.compose.AsyncImagePainter.State as CoilState
import coil3.compose.AsyncImage as CoilAsyncImage
import coil3.compose.LocalPlatformContext as CoilLocalContext
import coil3.request.CachePolicy as CoilCachePolicy
import coil3.request.ImageRequest as CoilImageRequest
import coil3.request.crossfade as coilCrossfade
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.imaging.ALLOW_ANIMATION_KEY
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GifCategory
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlin.time.Duration.Companion.milliseconds
import me.lampu.lampcord.shared.ui.kit.clickableCursor

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
                    // Keep the painter attached to the GIF rather than to a recycled grid slot or the current scroll index.
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
                            videoFallback = firstVideoUrl(favoritePreview.src, favoritePreview.gifSrc, favoritePreview.preview),
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
                            stillCandidates(category.src)
                        } else {
                            listOf(category.src)
                                .filter(String::isNotBlank)
                                .distinct()
                        },
                        animated = !reduceMotion,
                        videoFallback = firstVideoUrl(category.src),
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
    videoFallback: String? = null,
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
            forceVideo = forceVideo,
            videoFallback = videoFallback
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
        Box(Modifier.matchParentSize().clickableCursor(onClick = onClick))
    }
}

@Composable
private fun GifThumbnail(
    gif: Gif,
    onGifSelected: (Gif) -> Unit,
) {
    val reduceMotion = Settings.shared.reduceMotion
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val isDesktop = getPlatformName() == "windows"
    val animatePreview = !reduceMotion && (!isDesktop || isHovered)
    val candidates = remember(gif, reduceMotion, animatePreview) {
        if (animatePreview) gifAnimatedCandidates(gif) else gifStaticCandidates(gif)
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(1.5f)
            .clip(RoundedCornerShape(16.dp))
            .hoverable(interactionSource)
    ) {
        GifPreviewImage(
            candidates = candidates,
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            animated = animatePreview,
            forceVideo = gif.isVideo,
            videoFallback = firstVideoUrl(gif.src, gif.gifSrc, gif.preview)
        )
        Box(Modifier.matchParentSize().clickableCursor { onGifSelected(gif) })
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
    videoFallback: String? = null,
) {
    val context = CoilLocalContext.current
    // Keyed on the candidate list so switching Reduce Motion or hover resets the fallback position instead of
    // leaving it past the end of a shorter list.
    var candidateIndex by remember(candidates) { mutableIntStateOf(0) }
    var stillsExhausted by remember(candidates) { mutableStateOf(false) }
    val imageUrl = candidates.getOrNull(candidateIndex)
    // Video-only GIFs (Tenor categories, many favorites) have no image to decode, so the last resort for a
    // still preview is the video's first frame, decoded paused.
    val stillVideoUrl = if (!animated && (stillsExhausted || imageUrl == null)) videoFallback else null

    if (stillVideoUrl != null) {
        VideoPlayer(
            url = stillVideoUrl,
            modifier = modifier,
            loop = false,
            showControls = false,
            compact = true,
            autoPlay = false
        )
    } else if (imageUrl != null && animated && (forceVideo || isVideoMediaUrl(imageUrl))) {
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
            model = CoilImageRequest.Builder(context)
                .data(imageUrl)
                .memoryCacheKey("gif-preview:$imageUrl#${if (animated) "animated" else "still"}")
                .memoryCachePolicy(
                    if (getPlatformName() != "android" && getPlatformName() != "ios" && Settings.shared.desktopLowMemoryMode) CoilCachePolicy.DISABLED else CoilCachePolicy.ENABLED
                )
                .coilCrossfade(false)
                // Still candidates end with the original GIF, which must render as its first frame.
                .apply { extras[ALLOW_ANIMATION_KEY] = animated }
                .build(),
            contentDescription = contentDescription,
            modifier = modifier,
            contentScale = contentScale,
            onState = { state ->
                if (state is CoilState.Error) {
                    if (candidateIndex < candidates.lastIndex) candidateIndex++ else stillsExhausted = true
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
        // Discord's media proxy honours animated=false and returns the first frame even when the proxied
        // path ends in .gif, so that rewrite is a real still rather than a guess.
        !isAnimatedMediaUrl(url) || isProxiedStill(url)
    }.distinct()

    val variants = primary.flatMap { url ->
        buildList {
            val staticUrl = url.replace(
                Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE),
                "${'$'}1false"
            )
            val isAnimatedMedia = isAnimatedMediaUrl(staticUrl)
            if (staticUrl != url && !isAnimatedMedia) add(staticUrl)
            // Hosts other than Tenor and Discord's CDN answer a guessed rendition that doesn't exist with a
            // "content unavailable" placeholder image instead of an error, which would be shown as the preview.
            if (!supportsGuessedStills(staticUrl)) return@buildList

            val mediaSuffix = animatedMediaSuffix(staticUrl)
            if (mediaSuffix != null) {
                val suffixEnd = mediaSuffix.first + mediaSuffix.second.length
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
    return (staticPrimary + variants).distinct()
}

private fun isDiscordMediaProxy(url: String): Boolean =
    url.contains("images-ext-", ignoreCase = true) && url.contains(".discordapp.net/external/", ignoreCase = true) ||
        url.contains("media.discordapp.net", ignoreCase = true)

private fun isProxiedStill(url: String): Boolean =
    isDiscordMediaProxy(url) && url.contains(Regex("""[?&]animated=false""", RegexOption.IGNORE_CASE))

private fun supportsGuessedStills(url: String): Boolean {
    // A proxied URL embeds the origin host in its path, so check what it actually points at.
    val origin = url.substringAfter("/external/", url)
    if (origin.contains("klipy.com", ignoreCase = true)) return false
    return origin.contains("tenor.com", ignoreCase = true) ||
        origin.contains("cdn.discordapp.com", ignoreCase = true) ||
        origin.contains("media.discordapp.net", ignoreCase = true)
}

private fun gifStaticCandidates(gif: Gif): List<String> =
    stillCandidates(gif.preview, gif.gifSrc, gif.src)

// Guessed static renditions first, then the original GIF/WebP decoded without animation: Tenor in particular
// often has no static rendition at all, and without this fallback the preview stays empty.
private fun stillCandidates(vararg urls: String?): List<String> {
    val originals = urls.filterNotNull().map(String::trim).filter(String::isNotBlank)
    return (staticImageCandidates(*urls) + originals.filterNot(::isVideoMediaUrl)).distinct()
}

private fun firstVideoUrl(vararg urls: String?): String? =
    urls.filterNotNull().map(String::trim).firstOrNull { it.isNotBlank() && isVideoMediaUrl(it) }

private fun isAnimatedMediaUrl(url: String): Boolean =
    animatedMediaSuffix(url) != null

private fun isVideoMediaUrl(url: String): Boolean =
    videoMediaSuffix(url) != null

private fun videoMediaSuffix(url: String): Pair<Int, String>? {
    val path = url.substringBefore('?').substringBefore('#')
    val suffix = listOf(".webm", ".mp4", ".m4v", ".mov")
        .firstOrNull { path.endsWith(it, ignoreCase = true) }
        ?: return null
    return path.length - suffix.length to suffix
}

private fun animatedMediaSuffix(url: String): Pair<Int, String>? {
    val path = url.substringBefore('?').substringBefore('#')
    val suffix = listOf(".gif", ".webp", ".webm", ".mp4")
        .firstOrNull { path.endsWith(it, ignoreCase = true) }
        ?: return null
    return path.length - suffix.length to suffix
}

private fun gifAnimatedCandidates(gif: Gif): List<String> {
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
