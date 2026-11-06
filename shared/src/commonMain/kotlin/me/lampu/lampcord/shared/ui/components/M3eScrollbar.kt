package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.awaitTouchSlopOrCancellation
import androidx.compose.foundation.gestures.drag
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChange
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import kotlin.math.abs
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

    val averageItemSize = (visibleItems.sumOf { it.size } / visibleItems.size.toFloat()).coerceAtLeast(1f)
    val estimatedContentSize = averageItemSize * totalItems
    if (estimatedContentSize <= layoutInfo.viewportSize.height) return

    val maxFirstIndex = (totalItems - visibleItems.size).coerceAtLeast(1)

    ScrollbarThumb(
        modifier = modifier,
        scrollablePixels = estimatedContentSize - layoutInfo.viewportSize.height,
        progress = (visibleItems.first().index.toFloat() / maxFirstIndex).coerceIn(0f, 1f),
        reverseLayout = reverseLayout
    ) { fraction ->
        val index = (fraction * maxFirstIndex).roundToInt().coerceIn(0, totalItems - 1)
        if (index != state.firstVisibleItemIndex) state.scrollToItem(index)
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
        reverseLayout = false
    ) { fraction ->
        state.scrollTo((fraction * state.maxValue).roundToInt())
    }
}

@Composable
private fun ScrollbarThumb(
    modifier: Modifier,
    scrollablePixels: Float,
    progress: Float,
    reverseLayout: Boolean,
    scrollToFraction: suspend (Float) -> Unit
) {
    BoxWithConstraints(modifier.width(SCROLLBAR_HIT_WIDTH).fillMaxHeight()) {
        val viewportHeight = constraints.maxHeight.toFloat()
        if (viewportHeight <= 0f) return@BoxWithConstraints

        val density = LocalDensity.current
        val thumbHeight = (viewportHeight * viewportHeight / (viewportHeight + scrollablePixels))
            .coerceAtLeast(with(density) { SCROLLBAR_MIN_THUMB.toPx() })
            .coerceAtMost(viewportHeight)

        val maxThumbTop = viewportHeight - thumbHeight
        val restingTop = maxThumbTop * (if (reverseLayout) 1f - progress else progress)

        var dragTop by remember { mutableStateOf<Float?>(null) }
        val pressed = dragTop != null
        val scope = rememberCoroutineScope()

        val seek: (Float) -> Unit = { top ->
            val clamped = top.coerceIn(0f, maxThumbTop)
            dragTop = clamped
            val fraction = if (maxThumbTop > 0f) clamped / maxThumbTop else 0f
            scope.launch { scrollToFraction(if (reverseLayout) 1f - fraction else fraction) }
        }

        val latestSeek by rememberUpdatedState(seek)
        val latestRestingTop by rememberUpdatedState(restingTop)

        val thumbWidth by animateDpAsState(
            if (pressed) 8.dp else 4.dp,
            tween(120),
            label = "scrollbarWidth"
        )

        val thumbColor by animateColorAsState(
            if (pressed) MaterialTheme.colorScheme.primary.copy(alpha = 0.85f)
            else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.42f),
            tween(120),
            label = "scrollbarColor"
        )

        Box(
            Modifier
                .align(Alignment.TopCenter)
                .offset { IntOffset(0, (dragTop ?: restingTop).roundToInt()) }
                .width(SCROLLBAR_HIT_WIDTH)
                .height(with(density) { thumbHeight.toDp() })
                .pointerInput(Unit) {
                    awaitEachGesture {
                        val down = awaitFirstDown(requireUnconsumed = false)

                        var top = latestRestingTop
                        try {
                            dragTop = top

                            val slop = awaitTouchSlopOrCancellation(down.id) { change, overSlop ->
                                if (abs(overSlop.y) > abs(overSlop.x)) change.consume()
                            }

                            if (slop != null) drag(slop.id) { change ->
                                top += change.positionChange().y
                                latestSeek(top)
                                change.consume()
                            }
                        } finally {
                            dragTop = null
                        }
                    }
                }
        ) {
            Box(
                Modifier
                    .align(Alignment.TopCenter)
                    .width(thumbWidth)
                    .fillMaxHeight()
                    .background(thumbColor, CircleShape)
            )
        }
    }
}