package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val SCROLLBAR_HIT_WIDTH = 24.dp
private val SCROLLBAR_MIN_THUMB = 36.dp

@Composable
fun M3eVerticalScrollbar(
    state: LazyListState,
    modifier: Modifier = Modifier,
    isVisible: Boolean = true,
    reverseLayout: Boolean = false
) {
    if (!isVisible) return

    val layoutInfo = state.layoutInfo
    val visibleItems = layoutInfo.visibleItemsInfo
    val totalItems = layoutInfo.totalItemsCount
    if (visibleItems.isEmpty() || totalItems <= 0) return

    val averageItemSize = visibleItems.map { it.size }.average().toFloat().coerceAtLeast(1f)
    val estimatedContentSize = averageItemSize * totalItems
    if (estimatedContentSize <= layoutInfo.viewportSize.height) return

    val maxFirstIndex = (totalItems - visibleItems.size).coerceAtLeast(1)
    val progress = (visibleItems.first().index.toFloat() / maxFirstIndex).coerceIn(0f, 1f)

    ScrollbarThumb(
        modifier = modifier,
        scrollablePixels = (estimatedContentSize - layoutInfo.viewportSize.height).coerceAtLeast(1f),
        progress = progress,
        reverseLayout = reverseLayout,
        onSeek = { fraction ->
            val targetIndex = (fraction * maxFirstIndex).roundToInt().coerceIn(0, totalItems - 1)
            if (targetIndex != state.firstVisibleItemIndex) targetIndex else null
        }
    ) { index ->
        state.scrollToItem(index)
    }
}

@Composable
fun M3eVerticalScrollbar(
    state: ScrollState,
    modifier: Modifier = Modifier,
    isVisible: Boolean = true
) {
    if (!isVisible || state.maxValue <= 0) return

    ScrollbarThumb(
        modifier = modifier,
        scrollablePixels = state.maxValue.toFloat(),
        progress = (state.value.toFloat() / state.maxValue.toFloat()).coerceIn(0f, 1f),
        reverseLayout = false,
        onSeek = { fraction -> (fraction * state.maxValue).roundToInt() }
    ) { target ->
        state.scrollTo(target)
    }
}

@Composable
private fun ScrollbarThumb(
    modifier: Modifier,
    scrollablePixels: Float,
    progress: Float,
    reverseLayout: Boolean,
    onSeek: (Float) -> Int?,
    scrollTo: suspend (Int) -> Unit
) {
    BoxWithConstraints(
        modifier = modifier
            .width(SCROLLBAR_HIT_WIDTH)
            .fillMaxHeight()
    ) {
        val viewportHeight = constraints.maxHeight.toFloat()
        if (viewportHeight <= 0f || scrollablePixels <= 0f) return@BoxWithConstraints

        val minThumbPx = with(LocalDensity.current) { SCROLLBAR_MIN_THUMB.toPx() }
        val thumbHeight = (viewportHeight * viewportHeight / (viewportHeight + scrollablePixels))
            .coerceIn(minThumbPx.coerceAtMost(viewportHeight), viewportHeight)
        val maxThumbTop = (viewportHeight - thumbHeight).coerceAtLeast(0f)
        val thumbTop = if (reverseLayout) {
            maxThumbTop * (1f - progress)
        } else {
            maxThumbTop * progress
        }

        var dragging by remember { mutableStateOf(false) }
        val scrollJob = remember { mutableStateOf<Job?>(null) }
        val scope = rememberCoroutineScope()

        val thumbWidth by animateDpAsState(
            targetValue = if (dragging) 8.dp else 4.dp,
            animationSpec = tween(120),
            label = "scrollbarWidth"
        )
        val thumbAlpha by animateFloatAsState(
            targetValue = if (dragging) 0.85f else 0.42f,
            animationSpec = tween(120),
            label = "scrollbarAlpha"
        )
        val thumbColor = if (dragging) {
            MaterialTheme.colorScheme.primary
        } else {
            MaterialTheme.colorScheme.onSurface
        }

        fun seek(y: Float) {
            if (maxThumbTop <= 0f) return
            val top = (y - thumbHeight / 2f).coerceIn(0f, maxThumbTop)
            val fraction = if (reverseLayout) {
                1f - (top / maxThumbTop)
            } else {
                top / maxThumbTop
            }
            val target = onSeek(fraction) ?: return
            scrollJob.value?.cancel()
            scrollJob.value = scope.launch { scrollTo(target) }
        }

        Box(
            modifier = Modifier
                .fillMaxHeight()
                .width(SCROLLBAR_HIT_WIDTH)
                .pointerInput(scrollablePixels, reverseLayout, thumbHeight, viewportHeight) {
                    awaitEachGesture {
                        val down = awaitFirstDown(requireUnconsumed = false)
                        dragging = true
                        seek(down.position.y)

                        while (true) {
                            val event = awaitPointerEvent()
                            val change = event.changes.firstOrNull { it.id == down.id } ?: break
                            if (!change.pressed) break
                            seek(change.position.y)
                            change.consume()
                        }
                        dragging = false
                    }
                }
        ) {
            Box(
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .offset { IntOffset(0, thumbTop.roundToInt()) }
                    .width(thumbWidth)
                    .fillMaxHeight(thumbHeight / viewportHeight)
                    .background(
                        color = thumbColor.copy(alpha = thumbAlpha),
                        shape = RoundedCornerShape(percent = 50)
                    )
            )
        }
    }
}
