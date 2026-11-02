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
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

@Composable
fun ExpandableInputPanel(
    collapsedHeight: Dp,
    expandedHeight: Dp,
    appearFraction: Float,
    onDismissRequest: () -> Unit,
    onExpandedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val density = LocalDensity.current
    val collapsedPx = with(density) { collapsedHeight.toPx() }
    val expandedPx = with(density) { expandedHeight.toPx() }
    val minPx = collapsedPx * 0.45f
    val appear = appearFraction.coerceIn(0f, 1f)

    var expanded by remember { mutableStateOf(false) }
    var dragOffset by remember { mutableFloatStateOf(0f) }
    var dragging by remember { mutableStateOf(false) }

    val basePx = if (expanded) expandedPx else collapsedPx
    val targetPx = (basePx + dragOffset).coerceIn(minPx, expandedPx) * appear
    val heightPx by animateFloatAsState(
        targetValue = targetPx,
        animationSpec = if (dragging) snap() else tween(220),
        label = "inputPanelHeight"
    )

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .layout { measurable, constraints ->
                val placeable = measurable.measure(
                    constraints.copy(minHeight = 0, maxHeight = Constraints.Infinity)
                )
                val reportedHeight = (collapsedPx * appear).roundToInt().coerceIn(0, constraints.maxHeight)
                layout(placeable.width, reportedHeight) {
                    placeable.place(0, reportedHeight - placeable.height)
                }
            },
        shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .height(with(density) { heightPx.toDp() })
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(28.dp)
                    .pointerInput(collapsedPx, expandedPx) {
                        detectVerticalDragGestures(
                            onDragStart = { dragging = true },
                            onVerticalDrag = { change, dragAmount ->
                                change.consume()
                                dragOffset = (dragOffset - dragAmount)
                                    .coerceIn(minPx - basePx, expandedPx - basePx)
                            },
                            onDragEnd = {
                                val total = basePx + dragOffset
                                dragging = false
                                dragOffset = 0f
                                if (total < collapsedPx * 0.7f) {
                                    if (expanded) {
                                        expanded = false
                                        onExpandedChange(false)
                                    }
                                    onDismissRequest()
                                } else {
                                    val nowExpanded = total > (collapsedPx + expandedPx) / 2f
                                    if (nowExpanded != expanded) {
                                        expanded = nowExpanded
                                        onExpandedChange(nowExpanded)
                                    }
                                }
                            },
                            onDragCancel = {
                                dragging = false
                                dragOffset = 0f
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
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                content()
            }
        }
    }
}
