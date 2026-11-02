package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * Inline panel below the message input. It starts at [collapsedHeight] (the keyboard height)
 * and can be dragged by its handle up to [expandedHeight] like a bottom sheet.
 */
@Composable
fun ExpandableInputPanel(
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    collapsedHeight: Dp,
    expandedHeight: Dp,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val density = LocalDensity.current
    val collapsedPx = with(density) { collapsedHeight.toPx() }
    val expandedPx = with(density) { expandedHeight.toPx() }
    val basePx = if (expanded) expandedPx else collapsedPx

    var dragOffset by remember { mutableFloatStateOf(0f) }
    var dragging by remember { mutableStateOf(false) }
    val heightPx by animateFloatAsState(
        targetValue = (basePx + dragOffset).coerceIn(
            minOf(collapsedPx, expandedPx),
            maxOf(collapsedPx, expandedPx)
        ),
        animationSpec = if (dragging) snap() else tween(220),
        label = "inputPanelHeight"
    )

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .height(with(density) { heightPx.toDp() }),
        shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh
    ) {
        Column {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(28.dp)
                    .pointerInput(collapsedPx, expandedPx, basePx) {
                        detectVerticalDragGestures(
                            onDragStart = { dragging = true },
                            onVerticalDrag = { change, dragAmount ->
                                change.consume()
                                val min = minOf(collapsedPx, expandedPx) - basePx
                                val max = maxOf(collapsedPx, expandedPx) - basePx
                                dragOffset = (dragOffset + dragAmount).coerceIn(min, max)
                            },
                            onDragEnd = {
                                val total = basePx + dragOffset
                                onExpandedChange(total > (collapsedPx + expandedPx) / 2f)
                                dragOffset = 0f
                                dragging = false
                            },
                            onDragCancel = {
                                dragOffset = 0f
                                dragging = false
                            }
                        )
                    },
                contentAlignment = Alignment.Center
            ) {
                Box(
                    modifier = Modifier
                        .size(width = 36.dp, height = 4.dp)
                        .background(
                            MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f),
                            CircleShape
                        )
                )
            }
            Box(modifier = Modifier.weight(1f)) {
                content()
            }
        }
    }
}
