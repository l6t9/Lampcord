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
import coil3.compose.AsyncImagePainter.State as CoilState
import coil3.compose.LocalPlatformContext as CoilLocalContext
import coil3.compose.AsyncImagePainter as CoilPainter
import coil3.compose.AsyncImage as CoilAsyncImage
import coil3.decode.ImageSource
import coil3.request.CachePolicy as CoilCachePolicy
import coil3.request.ImageRequest as CoilImageRequest
import coil3.request.crossfade as coilCrossfade
import coil3.size.Precision as CoilPrecision
import coil3.size.Scale as CoilScale
import coil3.size.Size as CoilSize
import kotlin.io.encoding.ExperimentalEncodingApi
import me.lampu.lampcord.shared.model.TWEMOJI_CDN_BASE_URL
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.getPlatformName
import okio.Buffer
import okio.FileSystem

private val ANIMATED_TRUE_PARAM = Regex("""[?&]animated=true""", RegexOption.IGNORE_CASE)
private val ANIMATED_TRUE_PARAM_VALUE = Regex("""([?&]animated=)true""", RegexOption.IGNORE_CASE)
private val FORMAT_PARAM = Regex("""[?&]format=""", RegexOption.IGNORE_CASE)
private val FORMAT_PARAM_VALUE = Regex("""([?&]format=)[^&]*""", RegexOption.IGNORE_CASE)

sealed interface ImageLoadState {
    data object Loading : ImageLoadState
    data object Success : ImageLoadState
    data class Error(val message: String? = null) : ImageLoadState
}

private const val MAX_ANIMATED_DECODE = 512

private fun Any?.toCoilData(): Any? = when (this) {
    is ByteArray -> this
    is String -> {
        if (startsWith("$TWEMOJI_CDN_BASE_URL/")) {
            substringAfterLast('/')
                .let { ResourceLoader.readBytes("twemoji/72x72/$it") }
                ?: this
        } else {
            this
        }
    }

    else -> this
}

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun AsyncImage(
    model: Any?,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    onState: ((ImageLoadState) -> Unit)? = null,
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
    val coilContext = CoilLocalContext.current
    val isDesktop = remember { getPlatformName() != "android" && getPlatformName() != "ios" }
    val lowMemoryMode = isDesktop && Settings.shared.desktopLowMemoryMode
    val reducedMotion = Settings.shared.reduceMotion
    val staticModel = remember(model, reducedMotion, allowAnimation) {
        if (reducedMotion || !allowAnimation) model.toStaticDiscordGif() else model
    }
    var useOriginalModel by remember(model, reducedMotion) { mutableStateOf(false) }
    val effectiveModel = if (useOriginalModel) model else staticModel
    var loading by remember(effectiveModel) { mutableStateOf(true) }

    val coilRequest = remember(coilContext, effectiveModel, lowMemoryMode, size, allowAnimation) {
        effectiveModel.toCoilData()?.let { data ->
            CoilImageRequest.Builder(coilContext)
                .data(data)
                .memoryCacheKey(if (effectiveModel is ByteArray) null else "${effectiveModel}@${size ?: 0}")
                .memoryCachePolicy(if (lowMemoryMode) CoilCachePolicy.DISABLED else CoilCachePolicy.ENABLED)
                .networkCachePolicy(if (lowMemoryMode) CoilCachePolicy.DISABLED else CoilCachePolicy.ENABLED)
                .coilCrossfade(!reducedMotion && !lowMemoryMode)
                .precision(CoilPrecision.INEXACT)
                .scale(CoilScale.FIT)
                .apply {
                    if (size != null && size > 0) {
                        val targetSize = if (allowAnimation) minOf(size, MAX_ANIMATED_DECODE) else size
                        size(CoilSize(targetSize, targetSize))
                    }
                }
                .build()
        }
    }

    Box(
        modifier = modifier.then(if (shape != null) Modifier.clip(shape) else Modifier),
        contentAlignment = Alignment.Center
    ) {
        if (coilRequest != null) {
            CoilAsyncImage(
                model = coilRequest,
                contentDescription = contentDescription,
                modifier = Modifier.fillMaxSize(),
                onState = { state ->
                    loading = state is CoilState.Loading || state is CoilState.Empty
                    when (state) {
                        is CoilState.Success -> onState?.invoke(ImageLoadState.Success)
                        is CoilState.Error -> {
                            val retryOriginal = !reducedMotion && !useOriginalModel && staticModel != model
                            if (retryOriginal) {
                                useOriginalModel = true
                            } else {
                                onState?.invoke(ImageLoadState.Error(state.result.throwable.message))
                            }
                        }

                        else -> Unit
                    }
                    state
                },
                alignment = alignment,
                contentScale = contentScale,
                alpha = alpha,
                colorFilter = colorFilter,
                filterQuality = filterQuality
            )
        }

        if (showPlaceholder && loading && coilRequest != null) {
            ImageLoadingPlaceholder(Modifier.fillMaxSize())
        }
    }
}

private fun Any?.toStaticDiscordGif(): Any? {
    val url = this as? String ?: return this
    if (url.contains("/attachments/")) return this

    val isDiscordMedia = url.contains("cdn.discordapp.com", ignoreCase = true) ||
        url.contains("media.discordapp.net", ignoreCase = true)
    if (!isDiscordMedia) return this

    // Custom animated emojis use a WebP URL with animated=true rather than a .gif extension. Ask Discord for the static frame when motion is off.
    if (url.contains(ANIMATED_TRUE_PARAM)) {
        return url.replace(ANIMATED_TRUE_PARAM_VALUE, "${'$'}1false")
    }

    val gifSuffix = url.indexOf(".gif", ignoreCase = true)
    if (gifSuffix < 0) return this

    val suffixEnd = gifSuffix + 4
    if (suffixEnd == url.length || url[suffixEnd] == '?') {
        return url.substring(0, gifSuffix) + ".png" + url.substring(suffixEnd)
    }

    return if (url.contains(FORMAT_PARAM)) {
        url.replace(FORMAT_PARAM_VALUE, "${'$'}1png")
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

    // A static placeholder avoids starting one animation clock per image while a Windows message list is loading.
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