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
import me.lampu.lampcord.shared.settings.PanelAnimation
import me.lampu.lampcord.shared.settings.Settings
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
    var progress by mutableFloatStateOf(0f)
    var sidePanelWidthPx by mutableFloatStateOf(0f)

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
        val sidePanelWidth = screenWidth - 72.dp
        val sidePanelWidthPx = with(density) { sidePanelWidth.toPx() }
        state.sidePanelWidthPx = sidePanelWidthPx
        
        val animationType = Settings.shared.panelAnimation

        val targetOffset = when (state.currentValue) {
            DiscordPanelValue.Start -> sidePanelWidthPx
            DiscordPanelValue.Center -> 0f
            DiscordPanelValue.End -> -sidePanelWidthPx
        }

        // Snappy spring spec matching legacy Discord's feel
        val springSpec = if (animationType == PanelAnimation.MINIMAL) {
            spring<Float>(
                dampingRatio = Spring.DampingRatioNoBouncy,
                stiffness = Spring.StiffnessMedium
            )
        } else {
            spring<Float>(
                dampingRatio = 0.85f,
                stiffness = 1500f
            )
        }

        val animatedOffset by animateFloatAsState(
            targetValue = targetOffset + state.offset,
            animationSpec = springSpec,
            label = "panelOffset"
        )

        val progress = animatedOffset / sidePanelWidthPx
        state.progress = progress
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
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .width(sidePanelWidth)
                    .padding(start = 0.dp, top = 0.dp, bottom = 0.dp, end = 0.dp)
                    .graphicsLayer {
                        val isEffectivelyVisible = progress > 0.001f || state.currentValue == DiscordPanelValue.Start
                        alpha = if (isEffectivelyVisible) 1f else 0f
                        shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
                        clip = true
                        shadowElevation = 4.dp.toPx()
                        
                        if (animationType == PanelAnimation.EXPRESSIVE) {
                            translationX = (progress - 1f) * (sidePanelWidthPx * 0.3f)
                            val scale = 0.92f + (progress * 0.08f)
                            scaleX = scale
                            scaleY = scale
                        } else {
                            // Minimal: Slight parallax translation, no scale/alpha
                            translationX = (progress - 1f) * (sidePanelWidthPx * 0.1f)
                        }
                    }
                    .zIndex(if (progress > 0) 1f else 0f)
            ) {
                // To keep state (scroll position, etc.), it MUST stay in composition. 
                // We keep it composed but only visible when relevant.
                startPanel()
            }

            // End Panel (Right)
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .width(sidePanelWidth)
                    .align(Alignment.CenterEnd)
                    .padding(start = 0.dp, top = 0.dp, bottom = 0.dp, end = 16.dp)
                    .graphicsLayer {
                        val isEffectivelyVisible = progress < -0.001f || state.currentValue == DiscordPanelValue.End
                        alpha = if (isEffectivelyVisible) 1f else 0f
                        shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
                        clip = true
                        shadowElevation = 4.dp.toPx()

                        if (animationType == PanelAnimation.EXPRESSIVE) {
                            translationX = (progress + 1f) * (sidePanelWidthPx * 0.3f)
                            val scale = 0.92f + (absProgress * 0.08f)
                            scaleX = scale
                            scaleY = scale
                        } else {
                            // Minimal: Slight parallax translation, no scale/alpha
                            translationX = (progress + 1f) * (sidePanelWidthPx * 0.1f)
                        }
                    }
                    .zIndex(if (progress < 0) 1f else 0f)
            ) {
                endPanel()
            }

            // Center Panel (Main)
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .zIndex(2f)
                    .offset { IntOffset(animatedOffset.roundToInt(), 0) }
                    .padding(
                        start = 16.dp * absProgress,
                        top = 0.dp,
                        end = 16.dp * absProgress,
                        bottom = 0.dp
                    )
                    .graphicsLayer {
                        val isExpressive = animationType == PanelAnimation.EXPRESSIVE
                        
                        // Corner rounding and scaling for the "border" effect
                        // Discord-like: rounded only when open
                        val cornerRadius = 16.dp.toPx() * absProgress
                        shape = RoundedCornerShape(topStart = cornerRadius, topEnd = cornerRadius)
                        clip = absProgress > 0.01f
                        
                        if (isExpressive) {
                            val scale = 1f - (absProgress * 0.04f)
                            scaleX = scale
                            scaleY = scale
                            shadowElevation = if (absProgress > 0.01f) 12.dp.toPx() else 0f
                        } else {
                            shadowElevation = if (absProgress > 0.01f) 6.dp.toPx() else 0f
                        }
                    }
            ) {
                centerPanel()
                
                // Dimming scrim and click-to-close handler on center panel
                if (absProgress > 0.01f) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(Color.Black.copy(alpha = if (animationType == PanelAnimation.EXPRESSIVE) absProgress * 0.35f else absProgress * 0.2f))
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
