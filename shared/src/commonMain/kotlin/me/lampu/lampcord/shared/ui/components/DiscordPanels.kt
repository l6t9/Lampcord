package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

enum class DiscordPanelValue {
    Start, Center, End
}

class DiscordPanelsState(
    initialValue: DiscordPanelValue = DiscordPanelValue.Center
) {
    var currentValue by mutableStateOf(initialValue)
    var offset by mutableFloatStateOf(0f)

    fun openStart() {
        currentValue = DiscordPanelValue.Start
    }

    fun openEnd() {
        currentValue = DiscordPanelValue.End
    }

    fun close() {
        currentValue = DiscordPanelValue.Center
    }
}

@Composable
fun rememberDiscordPanelsState(
    initialValue: DiscordPanelValue = DiscordPanelValue.Center
): DiscordPanelsState {
    return remember { DiscordPanelsState(initialValue) }
}

@Composable
fun DiscordPanels(
    state: DiscordPanelsState,
    modifier: Modifier = Modifier,
    startPanel: @Composable BoxScope.() -> Unit,
    endPanel: @Composable BoxScope.() -> Unit,
    centerPanel: @Composable BoxScope.() -> Unit
) {
    BoxWithConstraints(modifier = modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val width = maxWidth
        val panelWidthPx = with(density) { (width - 56.dp).toPx() }

        val targetOffset = when (state.currentValue) {
            DiscordPanelValue.Start -> panelWidthPx
            DiscordPanelValue.Center -> 0f
            DiscordPanelValue.End -> -panelWidthPx
        }

        val animatedOffset by animateFloatAsState(
            targetValue = targetOffset + state.offset,
            animationSpec = spring(
                dampingRatio = 0.8f,
                stiffness = 1500f
            ),
            label = "panelOffset"
        )

        Box(modifier = Modifier.fillMaxSize()) {
            // Start Panel (Left)
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .width(width - 56.dp)
            ) {
                startPanel()
            }

            // End Panel (Right)
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .width(width - 56.dp)
                    .offset { IntOffset(with(density) { 56.dp.toPx() }.roundToInt(), 0) }
            ) {
                endPanel()
            }

            // Center Panel (Main)
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(animatedOffset.roundToInt(), 0) }
                    .draggable(
                        orientation = Orientation.Horizontal,
                        state = rememberDraggableState { delta ->
                            state.offset += delta
                        },
                        onDragStopped = { velocity ->
                            val threshold = panelWidthPx / 2
                            val currentTotalOffset = targetOffset + state.offset
                            
                            state.currentValue = when {
                                currentTotalOffset > threshold || (velocity > 500f && state.currentValue != DiscordPanelValue.End) -> DiscordPanelValue.Start
                                currentTotalOffset < -threshold || (velocity < -500f && state.currentValue != DiscordPanelValue.Start) -> DiscordPanelValue.End
                                else -> DiscordPanelValue.Center
                            }
                            state.offset = 0f
                        }
                    )
            ) {
                centerPanel()
            }
        }
    }
}
