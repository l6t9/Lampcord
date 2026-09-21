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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.settings.PanelAnimation
import me.lampu.lampcord.shared.settings.PanelType
import me.lampu.lampcord.shared.settings.Settings
import kotlin.math.abs

enum class DiscordPanelValue {
    Start, Center, End
}

class DiscordPanelsState(
    initialValue: DiscordPanelValue = DiscordPanelValue.Center
) {
    var currentValue by mutableStateOf(initialValue)
    var offset by mutableFloatStateOf(0f)
    var progress by mutableFloatStateOf(0f)
    var startPanelWidthPx by mutableFloatStateOf(0f)
    var endPanelWidthPx by mutableFloatStateOf(0f)

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
        val panelType = Settings.shared.panelType
        val isOverlapping = panelType == PanelType.OVERLAPPING

        val startPanelWidth = if (isOverlapping) screenWidth else (screenWidth - 72.dp)
        val endPanelWidth = if (isOverlapping) screenWidth else (screenWidth - 72.dp)

        val startPanelWidthPx = with(density) { startPanelWidth.toPx() }
        val endPanelWidthPx = with(density) { endPanelWidth.toPx() }
        
        state.startPanelWidthPx = startPanelWidthPx
        state.endPanelWidthPx = endPanelWidthPx

        val animationType = Settings.shared.panelAnimation
        val reduceMotion = Settings.shared.reduceMotion

        val targetOffset = when (state.currentValue) {
            DiscordPanelValue.Start -> startPanelWidthPx
            DiscordPanelValue.Center -> 0f
            DiscordPanelValue.End -> -endPanelWidthPx
        }

        val springSpec = if (animationType == PanelAnimation.MINIMAL) {
            spring<Float>(
                dampingRatio = Spring.DampingRatioNoBouncy,
                stiffness = Spring.StiffnessMedium
            )
        } else {
            spring<Float>(
                dampingRatio = 0.85f,
                stiffness = 1200f
            )
        }

        val animatedOffset by animateFloatAsState(
            targetValue = targetOffset + state.offset,
            animationSpec = if (reduceMotion) snap() else springSpec,
            label = "panelOffset"
        )

        val progress = if (animatedOffset >= 0) {
            (animatedOffset / startPanelWidthPx).coerceIn(0f, 1f)
        } else {
            (animatedOffset / endPanelWidthPx).coerceIn(-1f, 0f)
        }
        state.progress = progress
        val absProgress = abs(progress)

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
                                val minFlingVelocity = with(density) { 400.dp.toPx() }
                                val isFling = abs(velocity) > minFlingVelocity
                                val isRightSwipe = velocity > 0f

                                state.currentValue = when {
                                    isFling -> {
                                        if (isRightSwipe) {
                                            when (state.currentValue) {
                                                DiscordPanelValue.End -> DiscordPanelValue.Center
                                                else -> DiscordPanelValue.Start
                                            }
                                        } else {
                                            when (state.currentValue) {
                                                DiscordPanelValue.Start -> DiscordPanelValue.Center
                                                else -> DiscordPanelValue.End
                                            }
                                        }
                                    }
                                    currentTotalOffset > startPanelWidthPx * 0.45f -> DiscordPanelValue.Start
                                    currentTotalOffset < -endPanelWidthPx * 0.45f -> DiscordPanelValue.End
                                    else -> DiscordPanelValue.Center
                                }
                                state.offset = 0f
                            }
                        )
                    } else Modifier
                )
        ) {
            // Start Panel (Left - Server & Channels Drawer)
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .width(startPanelWidth)
                    .graphicsLayer {
                        val isEffectivelyVisible = progress > 0.001f || state.currentValue == DiscordPanelValue.Start
                        alpha = if (isEffectivelyVisible) 1f else 0f
                        shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
                        clip = true
                        shadowElevation = 4.dp.toPx()
                        transformOrigin = TransformOrigin(0.5f, 0f)
                        
                        if (reduceMotion) {
                            translationX = 0f
                            scaleX = 1f
                            scaleY = 1f
                        } else if (animationType == PanelAnimation.EXPRESSIVE) {
                            translationX = (progress - 1f) * (startPanelWidthPx * 0.3f)
                            val scale = 0.92f + (progress * 0.08f)
                            scaleX = scale
                            scaleY = scale
                        } else {
                            translationX = 0f
                        }
                    }
                    .zIndex(if (progress > 0) 1f else 0f)
            ) {
                startPanel()
            }

            // Center Panel (Main Chat View)
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .zIndex(2f)
                    .graphicsLayer {
                        translationX = if (isOverlapping) {
                            if (animatedOffset > 0) animatedOffset else 0f
                        } else {
                            animatedOffset
                        }
                        
                        val isExpressive = animationType == PanelAnimation.EXPRESSIVE
                        val cornerRadius = if (reduceMotion) {
                            0f
                        } else if (isOverlapping) {
                            if (progress > 0) 16.dp.toPx() * absProgress else 0f
                        } else {
                            16.dp.toPx() * absProgress
                        }
                        
                        shape = RoundedCornerShape(topStart = cornerRadius, topEnd = cornerRadius)
                        clip = !reduceMotion && absProgress > 0.01f
                        transformOrigin = TransformOrigin(0.5f, 0f)
                        
                        if (reduceMotion) {
                            scaleX = 1f
                            scaleY = 1f
                            shadowElevation = 0f
                        } else {
                            val shrinkFactor = if (isOverlapping && progress < 0) {
                                0f
                            } else {
                                if (isExpressive) 0.06f else 0.04f
                            }

                            val scale = 1f - (absProgress * shrinkFactor)
                            scaleX = scale
                            scaleY = scale

                            shadowElevation = if (absProgress > 0.01f) {
                                if (isExpressive) 12.dp.toPx() else 6.dp.toPx()
                            } else 0f
                        }
                    }
            ) {
                centerPanel()
                
                // Dimming scrim on center panel when side panels are active
                if (absProgress > 0.01f) {
                    val scrimAlpha = if (progress < 0) {
                        absProgress * 0.45f
                    } else if (animationType == PanelAnimation.EXPRESSIVE) {
                        absProgress * 0.45f
                    } else {
                        absProgress * 0.3f
                    }

                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(Color.Black.copy(alpha = scrimAlpha))
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null
                            ) {
                                state.close()
                            }
                    )
                }
            }

            // End Panel (Right - Member List / Detail Panel)
            val endSheetVisible = progress < -0.001f || state.currentValue == DiscordPanelValue.End
            if (endSheetVisible) {
                val isOverlapping = panelType == PanelType.OVERLAPPING
                Surface(
                    modifier = Modifier
                        .fillMaxHeight()
                        .width(endPanelWidth)
                        .align(Alignment.CenterEnd)
                        .padding(end = if (!isOverlapping) 8.dp else 0.dp)
                        .zIndex(if (isOverlapping) 3f else 1f)
                        .graphicsLayer {
                            if (isOverlapping) {
                                translationX = if (progress < 0) endPanelWidthPx * (1f + progress) else endPanelWidthPx
                                shadowElevation = if (reduceMotion) 0f else 8.dp.toPx()
                            } else {
                                translationX = 0f
                                alpha = 1f
                            }
                            shape = if (!isOverlapping) RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp) else RoundedCornerShape(0.dp)
                            clip = !isOverlapping
                        },
                    shape = if (!isOverlapping) RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp) else RoundedCornerShape(0.dp),
                    color = if (isOverlapping) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.background,
                    tonalElevation = 0.dp
                ) {
                    Box(modifier = Modifier.fillMaxSize()) {
                        endPanel()
                    }
                }
            }
        }
    }
}
