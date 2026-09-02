package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName
import kotlin.math.abs
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AdaptiveModalBottomSheet(
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    sheetState: SheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
    sheetMaxWidth: Dp = BottomSheetDefaults.SheetMaxWidth,
    shape: Shape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp),
    containerColor: Color = MaterialTheme.colorScheme.surface,
    contentColor: Color = contentColorFor(containerColor),
    tonalElevation: Dp = 0.dp,
    scrimColor: Color = BottomSheetDefaults.ScrimColor,
    dragHandle: @Composable (() -> Unit)? = {
        Box(
            modifier = Modifier
                .padding(vertical = 12.dp)
                .size(width = 40.dp, height = 4.dp)
                .clip(RoundedCornerShape(2.dp))
                .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
        )
    },
    contentWindowInsets: @Composable () -> WindowInsets = { BottomSheetDefaults.modalWindowInsets },
    properties: ModalBottomSheetProperties = ModalBottomSheetDefaults.properties,
    peekHeight: Dp? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val platform = getPlatformName()
    val isMobile = platform == "android" || platform == "ios"

    if (isMobile && !Settings.shared.reduceMotion) {
        AppBottomSheet(
            onDismissRequest = onDismissRequest,
            sheetMaxWidth = sheetMaxWidth,
            shape = shape,
            containerColor = containerColor,
            contentColor = contentColor,
            scrimColor = scrimColor,
            dragHandle = dragHandle,
            peekHeight = peekHeight,
            content = content
        )
    } else if (Settings.shared.reduceMotion) {
        Popup(
            onDismissRequest = onDismissRequest,
            alignment = Alignment.BottomCenter,
            properties = PopupProperties(
                focusable = true,
                dismissOnBackPress = true,
                dismissOnClickOutside = true
            )
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(scrimColor)
            ) {
                Box(
                    modifier = Modifier
                        .matchParentSize()
                        .clickable(
                            indication = null,
                            interactionSource = remember { MutableInteractionSource() },
                            onClick = onDismissRequest
                        )
                )
                Surface(
                    modifier = modifier
                        .align(Alignment.BottomCenter)
                        .widthIn(max = sheetMaxWidth)
                        .fillMaxWidth()
                        .fillMaxHeight(0.85f)
                        .padding(contentWindowInsets().asPaddingValues()),
                    shape = shape,
                    color = containerColor,
                    contentColor = contentColor,
                    tonalElevation = tonalElevation
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        dragHandle?.invoke()
                        content()
                    }
                }
            }
        }
    } else {
        ModalBottomSheet(
            onDismissRequest = onDismissRequest,
            modifier = modifier,
            sheetState = sheetState,
            sheetMaxWidth = sheetMaxWidth,
            shape = shape,
            containerColor = containerColor,
            contentColor = contentColor,
            tonalElevation = tonalElevation,
            scrimColor = scrimColor,
            dragHandle = dragHandle,
            contentWindowInsets = contentWindowInsets,
            properties = properties,
            content = content,
        )
    }
}

@Composable
private fun AppBottomSheet(
    onDismissRequest: () -> Unit,
    sheetMaxWidth: Dp,
    shape: Shape,
    containerColor: Color,
    contentColor: Color,
    scrimColor: Color,
    dragHandle: @Composable (() -> Unit)?,
    peekHeight: Dp?,
    content: @Composable ColumnScope.() -> Unit
) {
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    
    var isVisible by remember { mutableStateOf(false) }
    val offsetY = remember { Animatable(1000f) }
    var sheetHeight by remember { mutableStateOf(0f) }
    
    val peekHeightPx = remember(peekHeight, density) {
        peekHeight?.let { with(density) { it.toPx() } }
    }

    LaunchedEffect(sheetHeight) {
        if (sheetHeight > 0f && !isVisible) {
            isVisible = true
            val initialTarget = if (peekHeightPx != null) {
                (sheetHeight - peekHeightPx).coerceAtLeast(0f)
            } else 0f
            
            offsetY.animateTo(
                targetValue = initialTarget,
                animationSpec = spring(
                    dampingRatio = 0.85f,
                    stiffness = Spring.StiffnessMediumLow
                )
            )
        }
    }

    fun animateDismiss() {
        scope.launch {
            isVisible = false
            offsetY.animateTo(
                targetValue = 2000f,
                animationSpec = tween(durationMillis = 200, easing = FastOutLinearInEasing)
            )
            onDismissRequest()
        }
    }

    Popup(
        onDismissRequest = { animateDismiss() },
        alignment = Alignment.BottomCenter,
        properties = PopupProperties(
            focusable = true,
            dismissOnBackPress = true,
            dismissOnClickOutside = true
        )
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            val scrimAlpha by animateFloatAsState(
                targetValue = if (isVisible) 1f else 0f,
                animationSpec = tween(durationMillis = 300)
            )
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(scrimColor.copy(alpha = scrimColor.alpha * scrimAlpha))
                    .clickable(
                        indication = null,
                        interactionSource = remember { MutableInteractionSource() },
                        onClick = { animateDismiss() }
                    )
            )

            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(0, offsetY.value.roundToInt()) }
                    .draggable(
                        orientation = Orientation.Vertical,
                        state = rememberDraggableState { delta ->
                            scope.launch {
                                offsetY.snapTo((offsetY.value + delta).coerceAtLeast(0f))
                            }
                        },
                        onDragStopped = { velocity ->
                            scope.launch {
                                val currentOffset = offsetY.value
                                val peekOffset = sheetHeight - (peekHeightPx ?: 0f)

                                val hasPeek = peekHeightPx != null && peekOffset > 0f
                                val flingUp = velocity < -500f
                                val flingDown = velocity > 500f

                                when {
                                    // Flinging up (or already at the top) always expands fully.
                                    flingUp -> offsetY.animateTo(0f, spring(dampingRatio = 0.8f))
                                    // A sheet resting at peek collapses/dismisses on a downward fling.
                                    flingDown -> if (hasPeek && currentOffset <= peekOffset + 100f) {
                                        animateDismiss()
                                    } else {
                                        // Flinging down from full state first collapses to peek instead
                                        // of dismissing outright, matching Discord's bottom sheet.
                                        if (hasPeek) {
                                            offsetY.animateTo(peekOffset, spring(dampingRatio = 0.8f))
                                        } else {
                                            animateDismiss()
                                        }
                                    }
                                    hasPeek -> {
                                        val distToFull = currentOffset
                                        val distToPeek = abs(currentOffset - peekOffset)
                                        if (currentOffset > peekOffset + 100f && distToFull < distToPeek) {
                                            offsetY.animateTo(0f, spring(dampingRatio = 0.8f))
                                        } else if (currentOffset > peekOffset + 100f) {
                                            animateDismiss()
                                        } else {
                                            offsetY.animateTo(peekOffset, spring(dampingRatio = 0.8f))
                                        }
                                    }
                                    currentOffset > 300f -> animateDismiss()
                                    else -> offsetY.animateTo(0f, spring(dampingRatio = 0.8f))
                                }
                            }
                        }
                    ),
                contentAlignment = Alignment.BottomCenter
            ) {
                Surface(
                    modifier = Modifier
                        .widthIn(max = sheetMaxWidth)
                        .fillMaxWidth()
                        .wrapContentHeight()
                        .onGloballyPositioned { sheetHeight = it.size.height.toFloat() }
                        .navigationBarsPadding(),
                    shape = shape,
                    color = containerColor,
                    contentColor = contentColor,
                    tonalElevation = 0.dp
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        if (dragHandle != null) {
                            dragHandle()
                        }
                        content()
                    }
                }
            }
        }
    }
}
