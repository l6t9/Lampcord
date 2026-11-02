package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.requiredHeight
import androidx.compose.foundation.layout.requiredWidth
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
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ImageLoadState

private const val DEFAULT_HEIGHT_RATIO = 0.75f
internal const val FRAME_ART_SCALE = 1.0f

@Composable
fun ProfileFrameOverlay(
    product: ProfileFrameProduct,
    modifier: Modifier = Modifier,
    railTop: Dp = 0.dp,
    showBottom: Boolean = true,
) {
    BoxWithConstraints(modifier = modifier) {
        FrameLayers(product, maxWidth, railTop, showBottom)
    }
}

@Composable
private fun BoxScope.FrameLayers(
    product: ProfileFrameProduct,
    cardWidth: Dp,
    railTop: Dp,
    showBottom: Boolean,
) {
    val metrics = product.metrics
    val innerWidth = metrics.innerWidth.takeIf { it > 0 } ?: 1200
    val rawScale = cardWidth.value / innerWidth.toFloat() * FRAME_ART_SCALE
    val scale = rawScale.takeIf { it.isFinite() && it > 0f } ?: 1f
    val declaredWidth = ((innerWidth + 2f * metrics.overflowHorizontal) * scale)
        .dp
        .coerceIn(1.dp, 2000.dp)
    val topOverflow = (metrics.overflowTop * scale).dp.coerceIn(0.dp, 2000.dp)
    val bottomOverflow = (metrics.overflowBottom * scale).dp.coerceIn(0.dp, 2000.dp)

    LaunchedEffect(product.sku, cardWidth) {
        println(
            "[Frame] ${product.sku} card=$cardWidth layer=$declaredWidth top=$topOverflow " +
                "bottom=$bottomOverflow inner=${metrics.innerWidth} oh=${metrics.overflowHorizontal} " +
                "ot=${metrics.overflowTop} ob=${metrics.overflowBottom} " +
                "layers=${product.layers.map { "${it.id}:a${it.anchor}:f${it.front}:r${it.rail}" }}"
        )
    }

    for (layer in product.layers.sortedBy { it.front }) {
        val isBottom = layer.anchor == ANCHOR_BOTTOM
        if (isBottom && !showBottom) continue
        var heightRatio by remember(product.sku, layer.id) { mutableFloatStateOf(DEFAULT_HEIGHT_RATIO) }
        val y = when {
            isBottom -> bottomOverflow
            layer.rail -> railTop
            else -> -topOverflow
        }
        val rawLayerHeight = declaredWidth * heightRatio
        val layerHeight = if (rawLayerHeight.value.isFinite() && rawLayerHeight.value > 0f) {
            rawLayerHeight.coerceIn(1.dp, 2000.dp)
        } else {
            declaredWidth
        }
        AsyncImage(
            model = ProfileCollectibles.frameAssetUrl(product.sku, layer.id),
            contentDescription = null,
            modifier = Modifier
                .requiredWidth(declaredWidth)
                .requiredHeight(layerHeight)
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
