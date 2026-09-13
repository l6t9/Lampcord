package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AvatarDecorationData
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.utils.getPlatformName
import coil3.compose.AsyncImagePainter

@Composable
fun AvatarWithDecoration(
    avatarUrl: String?,
    fallbackAvatarUrl: String? = null,
    decorationData: AvatarDecorationData?,
    size: Dp,
    modifier: Modifier = Modifier,
    status: String? = null,
    animated: Boolean = !Settings.shared.reduceMotion,
    isHovered: Boolean = false,
    forceAnimate: Boolean = false
) {
    val avatarCandidates = remember(avatarUrl, fallbackAvatarUrl) {
        buildList {
            avatarUrl?.let(::add)
            avatarUrl
                ?.replace(Regex("(?i)\\.webp(?=\\?|$)"), ".png")
                ?.takeIf { it != avatarUrl }
                ?.let(::add)
            fallbackAvatarUrl?.let(::add)
        }.distinct()
    }
    var avatarCandidateIndex by remember(avatarCandidates) { mutableStateOf(0) }
    // Decode straight to the drawn size instead of the CDN size.
    val sizePx = with(LocalDensity.current) { size.roundToPx() }

    Box(modifier = modifier.size(size)) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            shape = CircleShape,
            color = MaterialTheme.colorScheme.secondaryContainer
        ) {
            val displayedAvatarUrl = avatarCandidates.getOrNull(avatarCandidateIndex)
            if (displayedAvatarUrl != null) {
                AsyncImage(
                    model = displayedAvatarUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                    filterQuality = androidx.compose.ui.graphics.FilterQuality.High,
                    size = sizePx,
                    onState = { state ->
                        if (state is AsyncImagePainter.State.Error && avatarCandidateIndex < avatarCandidates.lastIndex) {
                            avatarCandidateIndex++
                        }
                    }
                )
            }
        }

        // Decorations can be animated APNGs. Request the video-capable variant
        // only when hovered/visible, and always honor reduce-motion.
        if (decorationData != null) {
            val shouldAnimate = animated && (isHovered || forceAnimate)
            val decorationUrl = CdnUrls.getAvatarDecorationUrl(decorationData.asset)
            val isAndroid = remember { getPlatformName() == "android" }
            val effectiveUrl = if (shouldAnimate) {
                // On Android, use .gif for better animation support in Coil
                if (isAndroid) {
                    decorationUrl?.replace(".png", ".gif")
                } else {
                    decorationUrl
                }
            } else {
                decorationUrl?.replace("passthrough=true", "passthrough=false")
            }
            AsyncImage(
                model = effectiveUrl,
                contentDescription = null,
                modifier = Modifier
                    .fillMaxSize()
                    .scale(1.2f),
                filterQuality = androidx.compose.ui.graphics.FilterQuality.High,
                showPlaceholder = false,
                playAnimatedVideo = false,
                size = sizePx
            )
        }

        if (status != null) {
            StatusIndicator(
                status = status,
                size = size * 0.35f,
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .offset(x = 2.dp, y = 2.dp),
                borderColor = MaterialTheme.colorScheme.surface
            )
        }
    }
}
