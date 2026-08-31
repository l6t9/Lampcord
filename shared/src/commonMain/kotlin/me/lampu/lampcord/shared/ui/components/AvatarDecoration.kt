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
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AvatarDecorationData
import me.lampu.lampcord.shared.settings.Settings
import coil3.compose.AsyncImagePainter

@Composable
fun AvatarWithDecoration(
    avatarUrl: String?,
    fallbackAvatarUrl: String? = null,
    decorationData: AvatarDecorationData?,
    size: Dp,
    modifier: Modifier = Modifier,
    status: String? = null
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
                    onState = { state ->
                        if (state is AsyncImagePainter.State.Error && avatarCandidateIndex < avatarCandidates.lastIndex) {
                            avatarCandidateIndex++
                        }
                    }
                )
            }
        }

        // Decorations can be animated APNGs. Hide them while motion is
        // reduced instead of letting the CDN animation continue.
        if (decorationData != null && !Settings.shared.reduceMotion) {
            val decorationUrl = "https://cdn.discordapp.com/avatar-decoration-presets/${decorationData.asset}.png"
            AsyncImage(
                model = decorationUrl,
                contentDescription = null,
                modifier = Modifier
                    .fillMaxSize()
                    .scale(1.15f),
                filterQuality = androidx.compose.ui.graphics.FilterQuality.High
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
