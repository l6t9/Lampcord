package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import kotlin.math.abs
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
    swipeEnabled: Boolean = true,
    startPanel: @Composable BoxScope.() -> Unit,
    endPanel: @Composable BoxScope.() -> Unit,
    centerPanel: @Composable BoxScope.() -> Unit
) {
    BoxWithConstraints(modifier = modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val screenWidth = maxWidth
        val sidePanelWidth = screenWidth - 56.dp
        val sidePanelWidthPx = with(density) { sidePanelWidth.toPx() }

        val targetOffset = when (state.currentValue) {
            DiscordPanelValue.Start -> sidePanelWidthPx
            DiscordPanelValue.Center -> 0f
            DiscordPanelValue.End -> -sidePanelWidthPx
        }

        // Snappy spring spec matching legacy Discord's feel
        // Adjusted stiffness to be less "stiff" and more "fluid"
        val springSpec = spring<Float>(
            dampingRatio = 0.85f,
            stiffness = 1500f
        )

        val animatedOffset by animateFloatAsState(
            targetValue = targetOffset + state.offset,
            animationSpec = springSpec,
            label = "panelOffset"
        )

        val progress = animatedOffset / sidePanelWidthPx
        val absProgress = abs(progress).coerceIn(0f, 1f)

        Box(
            modifier = Modifier
                .fillMaxSize()
                .then(
                    if (swipeEnabled) {
                        Modifier.draggable(
                            orientation = Orientation.Horizontal,
                            state = rememberDraggableState { delta ->
                                state.offset += delta
                            },
                            onDragStopped = { velocity ->
                                val currentTotalOffset = targetOffset + state.offset
                                val threshold = sidePanelWidthPx * 0.45f // Slightly more than half-way to center
                                
                                // Use density-independent velocity for consistent feel across devices
                                val minFlingVelocity = with(density) { 400.dp.toPx() }
                                val isFling = abs(velocity) > minFlingVelocity
                                val isRightSwipe = velocity > 0f

                                state.currentValue = when {
                                    isFling -> {
                                        if (isRightSwipe) {
                                            // Right swipe: if at End -> Center, if at Center -> Start
                                            when (state.currentValue) {
                                                DiscordPanelValue.End -> DiscordPanelValue.Center
                                                else -> DiscordPanelValue.Start
                                            }
                                        } else {
                                            // Left swipe: if at Start -> Center, if at Center -> End
                                            when (state.currentValue) {
                                                DiscordPanelValue.Start -> DiscordPanelValue.Center
                                                else -> DiscordPanelValue.End
                                            }
                                        }
                                    }
                                    // Snap based on position when not flinging
                                    currentTotalOffset > threshold -> DiscordPanelValue.Start
                                    currentTotalOffset < -threshold -> DiscordPanelValue.End
                                    else -> DiscordPanelValue.Center
                                }
                                state.offset = 0f
                            }
                        )
                    } else Modifier
                )
        ) {
            // Start Panel (Left)
            if (progress > 0) {
                Box(
                    modifier = Modifier
                        .fillMaxHeight()
                        .width(sidePanelWidth)
                        .graphicsLayer {
                            translationX = (progress - 1f) * (sidePanelWidthPx * 0.3f)
                            alpha = (0.4f + (progress * 0.6f)).coerceIn(0f, 1f)
                            val scale = 0.92f + (progress * 0.08f)
                            scaleX = scale
                            scaleY = scale
                        }
                        .zIndex(0f)
                ) {
                    startPanel()
                }
            }

            // End Panel (Right)
            if (progress < 0) {
                Box(
                    modifier = Modifier
                        .fillMaxHeight()
                        .width(sidePanelWidth)
                        .align(Alignment.CenterEnd)
                        .graphicsLayer {
                            translationX = (progress + 1f) * (sidePanelWidthPx * 0.3f)
                            alpha = (0.4f + (absProgress * 0.6f)).coerceIn(0f, 1f)
                            val scale = 0.92f + (absProgress * 0.08f)
                            scaleX = scale
                            scaleY = scale
                        }
                        .zIndex(0f)
                ) {
                    endPanel()
                }
            }

            // Center Panel (Main)
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .zIndex(1f)
                    .offset { IntOffset(animatedOffset.roundToInt(), 0) }
                    .graphicsLayer {
                        val cornerRadius = 18.dp.toPx() * absProgress
                        shape = RoundedCornerShape(cornerRadius)
                        clip = absProgress > 0.01f
                        shadowElevation = if (absProgress > 0.01f) 8.dp.toPx() else 0f
                    }
            ) {
                centerPanel()
                
                // Dimming scrim and click-to-close handler on center panel
                if (absProgress > 0.01f) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(Color.Black.copy(alpha = absProgress * 0.35f))
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null
                            ) {
                                state.close()
                            }
                    )
                }
            }
        }
    }
}
