package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.DefaultAlpha
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import com.github.panpf.sketch.AsyncImage as SketchAsyncImage
import com.github.panpf.sketch.LocalPlatformContext
import com.github.panpf.sketch.PainterState
import com.github.panpf.sketch.cache.CachePolicy
import com.github.panpf.sketch.fetch.newBase64Uri
import com.github.panpf.sketch.rememberAsyncImageState
import com.github.panpf.sketch.request.ImageRequest
import com.github.panpf.sketch.request.disallowAnimatedImage
import com.github.panpf.sketch.util.Size
import kotlin.io.encoding.ExperimentalEncodingApi
import me.lampu.lampcord.shared.model.TWEMOJI_CDN_BASE_URL
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.getPlatformName

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun AsyncImage(
    model: Any?,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    onState: ((PainterState) -> Unit)? = null,
    alignment: Alignment = Alignment.Center,
    contentScale: ContentScale = ContentScale.Fit,
    alpha: Float = DefaultAlpha,
    colorFilter: ColorFilter? = null,
    filterQuality: FilterQuality = FilterQuality.High,
    shape: Shape? = null,
    showPlaceholder: Boolean = true,
    placeholderHash: String? = null,
    size: Int? = null,
    allowAnimation: Boolean = true
) {
    val context = LocalPlatformContext.current
    val isDesktop = remember { getPlatformName() != "android" && getPlatformName() != "ios" }
    val lowMemoryMode = isDesktop && Settings.shared.desktopLowMemoryMode
    val reducedMotion = Settings.shared.reduceMotion
    val staticModel = remember(model, reducedMotion) {
        if (reducedMotion) model.toStaticDiscordGif() else model
    }
    // Some animated-avatar hashes do not expose a PNG representation. Never
    // leave an avatar blank in that case; retry its original CDN URL.
    var useOriginalModel by remember(model, reducedMotion) { mutableStateOf(false) }
    val effectiveModel = if (useOriginalModel) model else staticModel

    val request = remember(effectiveModel, reducedMotion, lowMemoryMode, allowAnimation) {
        effectiveModel.toRequestUri()?.let { uri ->
            ImageRequest.Builder(context, uri)
                // Keep lazy-list cells from reusing a request for a different URL, and
                // keep the animated and static variants of the same URL in separate
                // cache entries. Otherwise whichever renders first (the static first
                // frame, or the animated image) is served to the other and decoration
                // hover/profile animation never starts.
                .memoryCacheKey(effectiveModel?.toString() + if (allowAnimation) "#animated" else "#static")
                .memoryCachePolicy(if (lowMemoryMode) CachePolicy.DISABLED else CachePolicy.ENABLED)
                .crossfade(!reducedMotion && !lowMemoryMode)
                // Decode at the image's full resolution. Sketch's default auto-size
                // resolver downsamples to the exact on-screen pixel size with a cheap
                // box sample, which aliases ("pixelates") static images. Full-res decode
                // + Compose's high-quality filter renders as crisp as the animated images.
                .size(Size.Empty)
                .disallowAnimatedImage(!allowAnimation)
                .build()
        }
    }

    val sketchState = rememberAsyncImageState()
    // Keep the callbacks on the shared state so AsyncImageTarget can invoke
    // them when the painter state changes.
    sketchState.onPainterState = { painterState ->
        val retryOriginal = painterState is PainterState.Error &&
            !reducedMotion &&
            !useOriginalModel && staticModel != model
        if (retryOriginal) {
            useOriginalModel = true
        } else {
            onState?.invoke(painterState)
        }
    }

    val isLoading = sketchState.painterState == null || sketchState.painterState is PainterState.Loading

    Box(
        modifier = modifier.then(if (shape != null) Modifier.clip(shape) else Modifier),
        contentAlignment = Alignment.Center
    ) {
        if (request != null) {
            SketchAsyncImage(
                request = request,
                contentDescription = contentDescription,
                modifier = Modifier.fillMaxSize(),
                state = sketchState,
                alignment = alignment,
                contentScale = contentScale,
                alpha = alpha,
                colorFilter = colorFilter,
                filterQuality = filterQuality
            )
        }

        if (showPlaceholder && isLoading && request != null) {
            ImageLoadingPlaceholder(Modifier.fillMaxSize())
        }
    }
}

/** Sketch's ImageRequest only accepts a URI; Coil accepted arbitrary bytes. */
@OptIn(ExperimentalEncodingApi::class)
private fun Any?.toRequestUri(): String? = when (this) {
    is ByteArray -> newBase64Uri("image/png", this)
    is String -> {
        if (startsWith("$TWEMOJI_CDN_BASE_URL/")) {
            // Prefer the bundled Twemoji copy instead of downloading it.
            substringAfterLast('/')
                .let { ResourceLoader.readBytes("twemoji/72x72/$it") }
                ?.let { newBase64Uri("image/png", it) }
                ?: this
        } else {
            this
        }
    }
    else -> this?.toString()
}

/** Discord's media proxy can return the first frame of GIF media as PNG. */
private fun Any?.toStaticDiscordGif(): Any? {
    val url = this as? String ?: return this
    val isDiscordMedia = url.contains("cdn.discordapp.com", ignoreCase = true) ||
        url.contains("media.discordapp.net", ignoreCase = true)
    if (!isDiscordMedia) return this

    // Custom animated emojis use a WebP URL with animated=true rather than
    // a .gif extension. Ask Discord for the static frame when motion is off.
    if (url.contains(Regex("""[?&]animated=true""", RegexOption.IGNORE_CASE))) {
        return url.replace(
            Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE),
            "${'$'}1false"
        )
    }

    val gifSuffix = url.indexOf(".gif", ignoreCase = true)
    if (gifSuffix < 0) return this

    // Asking the CDN for the PNG path is more reliable than format=png on
    // Android, especially for animated avatars and custom status emojis.
    val suffixEnd = gifSuffix + 4
    if (suffixEnd == url.length || url[suffixEnd] == '?') {
        return url.substring(0, gifSuffix) + ".png" + url.substring(suffixEnd)
    }

    return if (url.contains(Regex("""[?&]format=""", RegexOption.IGNORE_CASE))) {
        url.replace(Regex("""([?&]format=)[^&]*""", RegexOption.IGNORE_CASE), "${'$'}1png")
    } else {
        "$url${if (url.contains("?")) "&" else "?"}format=png"
    }
}

@Composable
fun ShimmerBox(
    modifier: Modifier = Modifier,
    shape: Shape = RoundedCornerShape(4.dp)
) {
    ImageLoadingPlaceholder(modifier.clip(shape))
}

@Composable
fun ImageLoadingPlaceholder(
    modifier: Modifier = Modifier
) {
    val colorScheme = MaterialTheme.colorScheme
    val isDark = colorScheme.surface.luminance() < 0.5f

    val base = if (isDark) {
        colorScheme.surfaceContainerHigh.let { 
            if (it == Color.Black) Color(0xFF121212) else it 
        }
    } else {
        colorScheme.surfaceContainerHigh
    }
    
    val highlight = if (isDark) {
        colorScheme.surfaceVariant.let {
            if (it == Color.Black) Color(0xFF1E1E1E) else it
        }
    } else {
        colorScheme.surfaceVariant
    }

    // A static placeholder avoids starting one animation clock per image while
    // a Windows message list is loading.
    if (Settings.shared.reduceMotion || remember { getPlatformName() == "windows" }) {
        Box(modifier = modifier.background(base))
        return
    }
    
    val transition = rememberInfiniteTransition(label = "shimmer")
    val progress by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 1500, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "shimmerProgress"
    )

    Box(
        modifier = modifier.drawBehind {
            val width = size.width
            val height = size.height
            val xOffset = progress * (width * 2) - width
            
            drawRect(
                brush = Brush.linearGradient(
                    colors = listOf(base, highlight, base),
                    start = Offset(xOffset, 0f),
                    end = Offset(xOffset + width, height)
                )
            )
        }
    )
}