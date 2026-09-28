package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.requiredHeight
import androidx.compose.foundation.layout.requiredWidth
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.ANCHOR_BOTTOM
import me.lampu.lampcord.shared.model.ProfileCollectibles
import me.lampu.lampcord.shared.model.ProfileFrameProduct
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ImageLoadState

private const val DEFAULT_HEIGHT_RATIO = 0.75f
private const val DEFAULT_INNER_WIDTH = 1200f
internal const val FRAME_ART_SCALE = 1.0f

internal data class FrameInsets(
    val horizontal: Dp = 0.dp,
    val top: Dp = 0.dp,
    val bottom: Dp = 0.dp,
)

@Composable
internal fun rememberFrameInsets(
    profile: UserProfile?,
    profileStore: ProfileStore,
    cardWidth: Dp,
    gap: Dp = 8.dp,
): FrameInsets {
    val sku = remember(profile) {
        profile?.let { ProfileCollectibles.frameSku(it.user_profile, it.guild_member_profile) }
    }
    val metrics = remember(sku) { sku?.let { profileStore.getFrame(it)?.metrics } }
    return remember(metrics, cardWidth, gap) {
        val scale = cardWidth.value / (metrics?.innerWidth?.toFloat() ?: DEFAULT_INNER_WIDTH) * FRAME_ART_SCALE
        FrameInsets(
            horizontal = ((metrics?.overflowHorizontal ?: 0) * scale).dp + gap,
            top = ((metrics?.overflowTop ?: 0) * scale).dp + gap,
            bottom = ((metrics?.overflowBottom ?: 0) * scale).dp + gap,
        )
    }
}

@Composable
fun ProfileFrameOverlay(
    product: ProfileFrameProduct,
    modifier: Modifier = Modifier,
    railTop: Dp = 0.dp,
) {
    BoxWithConstraints(modifier = modifier) {
        val metrics = product.metrics
        val scale = maxWidth.value / metrics.innerWidth.toFloat() * FRAME_ART_SCALE
        val declaredWidth = ((metrics.innerWidth + 2f * metrics.overflowHorizontal) * scale).dp
        val topOverflow = (metrics.overflowTop * scale).dp
        val bottomOverflow = (metrics.overflowBottom * scale).dp

        LaunchedEffect(product.sku, maxWidth) {
            println(
                "[Frame] ${product.sku} card=$maxWidth layer=$declaredWidth top=$topOverflow " +
                    "bottom=$bottomOverflow inner=${metrics.innerWidth} oh=${metrics.overflowHorizontal} " +
                    "ot=${metrics.overflowTop} ob=${metrics.overflowBottom} " +
                    "layers=${product.layers.map { "${it.id}:a${it.anchor}:f${it.front}:r${it.rail}" }}"
            )
        }

        for (layer in product.layers.sortedBy { it.front }) {
            val isBottom = layer.anchor == ANCHOR_BOTTOM
            var heightRatio by remember(product.sku, layer.id) { mutableFloatStateOf(DEFAULT_HEIGHT_RATIO) }
            val y = when {
                isBottom -> bottomOverflow
                layer.rail -> railTop
                else -> -topOverflow
            }
            AsyncImage(
                model = ProfileCollectibles.frameAssetUrl(product.sku, layer.id),
                contentDescription = null,
                modifier = Modifier
                    .requiredWidth(declaredWidth)
                    .requiredHeight(declaredWidth * heightRatio)
                    .align(if (isBottom) Alignment.BottomCenter else Alignment.TopCenter)
                    .offset(y = y),
                contentScale = ContentScale.Fit,
                showPlaceholder = false,
                onState = { state ->
                    if (state is ImageLoadState.Error) {
                        println("[Frame] layer ${layer.id} of ${product.sku} failed: ${state.message}")
                    }
                },
                onSize = { width, height ->
                    if (width > 0 && height > 0) {
                        heightRatio = height.toFloat() / width
                    }
                }
            )
        }
    }
}
