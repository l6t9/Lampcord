package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.platform.WindowInfo
import kotlin.math.max
import kotlin.math.min

@Stable
@OptIn(ExperimentalComposeUiApi::class)
class WindowVisibility internal constructor(private val threshold: Float) {
    private var windowInfo: WindowInfo? = null

    var isVisible by mutableStateOf(true)
        private set

    val modifier: Modifier = Modifier.onGloballyPositioned { coordinates ->
        val container = windowInfo?.containerSize ?: return@onGloballyPositioned
        if (container.width <= 0 || container.height <= 0) {
            if (!isVisible) isVisible = true
            return@onGloballyPositioned
        }

        val bounds = coordinates.boundsInWindow()
        val width = bounds.width
        val height = bounds.height
        if (width <= 0f || height <= 0f) return@onGloballyPositioned

        val overlapWidth = min(bounds.right, container.width.toFloat()) - max(bounds.left, 0f)
        val overlapHeight = min(bounds.bottom, container.height.toFloat()) - max(bounds.top, 0f)
        val visibleFraction = if (overlapWidth > 0f && overlapHeight > 0f) {
            (overlapWidth * overlapHeight) / (width * height)
        } else {
            0f
        }

        val visible = visibleFraction >= threshold
        if (visible != isVisible) isVisible = visible
    }

    internal fun attach(info: WindowInfo) {
        windowInfo = info
    }
}

@Composable
fun rememberWindowVisibility(threshold: Float = 0.25f): WindowVisibility {
    val windowInfo = LocalWindowInfo.current
    return remember(windowInfo, threshold) {
        WindowVisibility(threshold).also { it.attach(windowInfo) }
    }
}
