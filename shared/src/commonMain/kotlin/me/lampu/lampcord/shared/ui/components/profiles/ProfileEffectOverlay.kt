package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.requiredHeight
import androidx.compose.foundation.layout.requiredWidth
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.model.EFFECT_CANVAS_WIDTH
import me.lampu.lampcord.shared.model.ProfileEffectProduct
import me.lampu.lampcord.shared.model.isEffectLayerActive
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ImageLoadState

private const val TICK_MS = 33L

@Composable
fun ProfileEffectOverlay(
    product: ProfileEffectProduct,
    modifier: Modifier = Modifier,
    animate: Boolean = true,
) {
    if (product.layers.isEmpty() && product.source == null) return

    BoxWithConstraints(modifier = modifier) {
        val canvasWidth = maxWidth.coerceIn(1.dp, 2000.dp)
        val scale = canvasWidth.value / EFFECT_CANVAS_WIDTH.toFloat()
        val surfaceHeight = remember(product.sku, canvasWidth, maxHeight) {
            val bottom = product.layers.maxOfOrNull { it.y + it.height } ?: EFFECT_CANVAS_WIDTH
            (bottom * scale).dp.coerceIn(1.dp, maxHeight.coerceAtLeast(1.dp))
        }
        var elapsed by remember(product.sku, animate) { mutableLongStateOf(0L) }

        LaunchedEffect(product.sku, animate) {
            if (!animate) {
                elapsed = 0L
                return@LaunchedEffect
            }
            while (true) {
                delay(TICK_MS)
                elapsed += TICK_MS
            }
        }

        Box(
            modifier = Modifier
                .width(canvasWidth)
                .height(surfaceHeight)
                .align(Alignment.TopStart)
        ) {
            // The static fallback is only for reduced motion / cancelled animations; drawing it
            // underneath live layers causes a visible cross-fade and layout jump on first load.
            if (!animate || product.layers.isEmpty()) {
                val fallbackSource = product.source ?: product.layers.firstOrNull()?.source
                fallbackSource?.let { source ->
                    AsyncImage(
                        model = source,
                        contentDescription = null,
                        modifier = Modifier.width(canvasWidth),
                        contentScale = ContentScale.Fit,
                        showPlaceholder = false
                    )
                }
            }

            for (layer in product.layers) {
                if (!isEffectLayerActive(layer, elapsed)) continue
                val layerWidth = (canvasWidth * (layer.width / EFFECT_CANVAS_WIDTH).toFloat())
                    .coerceIn(1.dp, 2000.dp)
                val layerHeight = (canvasWidth * (layer.height / EFFECT_CANVAS_WIDTH).toFloat())
                    .coerceIn(1.dp, 2000.dp)
                AsyncImage(
                    model = layer.source,
                    contentDescription = null,
                    modifier = Modifier
                        .offset(
                            x = canvasWidth * (layer.x / EFFECT_CANVAS_WIDTH).toFloat(),
                            y = canvasWidth * (layer.y / EFFECT_CANVAS_WIDTH).toFloat()
                        )
                        .requiredWidth(layerWidth)
                        .requiredHeight(layerHeight),
                    contentScale = ContentScale.Fit,
                    allowAnimation = animate,
                    showPlaceholder = false,
                    onState = { state ->
                        if (state is ImageLoadState.Error) {
                            println("[Effect] layer ${layer.source} failed: ${state.message}")
                        }
                    }
                )
            }
        }
    }
}
