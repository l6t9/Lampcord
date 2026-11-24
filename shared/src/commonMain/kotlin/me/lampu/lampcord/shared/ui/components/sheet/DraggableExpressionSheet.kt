package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.AnimationVector1D
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.State
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.util.lerp
import kotlin.concurrent.Volatile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.time.TimeSource

private val KeyboardSheetTopInset = 56.dp
private val CustomKeyboardHeight = 253.dp

/** Discord persists the learned height under "customKeyboardHeight". */
private object CustomKeyboardHeightStore {
    @Volatile
    var heightPx: Float? = null
}

/**
 * State for a two-detent keyboard-replacement sheet.
 *
 * The owning screen keeps this state so actions such as choosing an emoji can
 * collapse the sheet without closing it, matching Discord's expression picker.
 */
@Stable
class KeyboardSheetState internal constructor(
    internal val animatable: Animatable<Float, AnimationVector1D>,
    private val scope: CoroutineScope,
    private val animationSpec: () -> FiniteAnimationSpec<Float>,
    private val enabled: Boolean = true
) {
    val progress: State<Float> = animatable.asState()

    val expanded: Boolean by derivedStateOf {
        progress.value >= KeyboardSheetPolicy.DETENT_THRESHOLD
    }

    internal var onSettled: (() -> Unit)? = null

    fun expand() {
        if (!enabled) return
        animateTo(1f)
    }

    fun collapse() {
        if (!enabled) return
        animateTo(0f)
    }

    fun toggle() {
        if (!enabled) return
        if (expanded) collapse() else expand()
    }

    private fun animateTo(target: Float) {
        if (animatable.targetValue == target && animatable.value == target) return
        scope.launch {
            animatable.animateTo(target, animationSpec())
        }
    }

    internal suspend fun dragTo(value: Float) {
        if (!enabled) return
        animatable.snapTo(value.coerceIn(0f, 1f))
    }

    internal suspend fun settle(velocity: Float) {
        if (!enabled) return
        val before = animatable.value
        val target = KeyboardSheetPolicy.snapTarget(before, velocity)
        animatable.animateTo(target, animationSpec())
        if (KeyboardSheetPolicy.crossedDetent(before, target)) {
            onSettled?.invoke()
        }
    }

    companion object {
        val Disabled = KeyboardSheetState(
            animatable = Animatable(0f),
            scope = CoroutineScope(EmptyCoroutineContext),
            animationSpec = { snap() },
            enabled = false
        )
    }
}

@Composable
fun rememberKeyboardSheetState(
    initialExpanded: Boolean = false,
    reduceMotion: Boolean = me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion
): KeyboardSheetState {
    val scope = rememberCoroutineScope()
    val animatable = remember(initialExpanded) { Animatable(if (initialExpanded) 1f else 0f) }
    return remember(animatable, scope, reduceMotion) {
        KeyboardSheetState(
            animatable = animatable,
            scope = scope,
            animationSpec = {
                if (reduceMotion) {
                    snap()
                } else {
                    spring<Float>(dampingRatio = 0.8f, stiffness = 240f)
                }
            }
        )
    }
}

/**
 * Discord remembers the last system-keyboard height and reuses it as the
 * custom-keyboard height (with a 200dp floor). The value is only learned while
 * a chat input is focused, and defaults to 253dp before that happens.
 */
@Composable
fun rememberCustomKeyboardHeight(focused: Boolean): Dp {
    val density = LocalDensity.current
    val systemKeyboardHeight = systemKeyboardHeightPx(density)
    val persistedDp = CustomKeyboardHeightStore.heightPx?.let { with(density) { it.toDp().value } } ?: 0f
    var rememberedPx by remember {
        mutableFloatStateOf(
            if (persistedDp > 0f) {
                with(density) { persistedDp.dp.toPx() }
            } else {
                CustomKeyboardHeightStore.heightPx ?: with(density) { CustomKeyboardHeight.toPx() }
            }
        )
    }

    LaunchedEffect(systemKeyboardHeight, focused) {
        if (!focused || systemKeyboardHeight <= 0) return@LaunchedEffect
        val systemHeightDp = with(density) { systemKeyboardHeight.toDp() }
        val heightDp = maxOf(systemHeightDp, 200.dp)
        val heightPx = with(density) { heightDp.toPx() }
        CustomKeyboardHeightStore.heightPx = heightPx
        rememberedPx = heightPx
    }

    return with(density) { rememberedPx.toDp() }
}

/**
 * Discord's custom keyboard geometry: minimum is the persisted custom keyboard
 * height, maximum is the window height minus the 56dp multiline nav bar and
 * the safe-area top inset.
 */
@Composable
fun rememberKeyboardSheetDetents(focused: Boolean): Pair<Dp, Dp> {
    val density = LocalDensity.current
    val windowHeight = with(density) { LocalWindowInfo.current.containerSize.height.toDp() }
    val safeAreaTop = with(density) { WindowInsets.safeDrawing.getTop(density).toDp() }
    val keyboardHeight = rememberCustomKeyboardHeight(focused)

    val (minimum, maximum) = KeyboardSheetPolicy.detents(
        windowHeightPx = with(density) { windowHeight.toPx() },
        safeAreaTopPx = with(density) { safeAreaTop.toPx() },
        keyboardHeightPx = with(density) { keyboardHeight.toPx() },
        navBarHeightPx = with(density) { KeyboardSheetTopInset.toPx() }
    )
    return with(density) { minimum.toDp() to maximum.toDp() }
}

@Composable
private fun Modifier.keyboardSheetDrag(
    state: KeyboardSheetState,
    travelPx: Float,
    scope: CoroutineScope,
): Modifier = pointerInput(state, travelPx) {
        var lastMark = TimeSource.Monotonic.markNow()
        var velocity = 0f
        var dragJob: Job? = null

        detectVerticalDragGestures(
            onDragStart = {
                lastMark = TimeSource.Monotonic.markNow()
                velocity = 0f
            },
            onDragEnd = {
                dragJob?.cancel()
                scope.launch { state.settle(velocity) }
            },
            onDragCancel = {
                dragJob?.cancel()
                scope.launch { state.settle(0f) }
            }
        ) { _, dragAmount ->
            val now = TimeSource.Monotonic.markNow()
            val elapsedMillis = (now - lastMark).inWholeMilliseconds.coerceAtLeast(1)
            velocity = dragAmount * 1000f / elapsedMillis
            lastMark = now

            val next = KeyboardSheetPolicy.progressAfterDrag(
                currentProgress = state.animatable.value,
                dragDeltaPx = dragAmount,
                travelPx = travelPx
            )
            dragJob?.cancel()
            dragJob = scope.launch { state.dragTo(next) }
        }
}

/**
 * A keyboard-replacement sheet with two detents, matching Discord's expression
 * and media keyboards. It is hosted at the root by MobileBaseplate, so
 * expanding it overlays the input bar instead of pushing the chat layout.
 */
@Composable
fun DraggableKeyboardSheet(
    modifier: Modifier = Modifier,
    state: KeyboardSheetState = rememberKeyboardSheetState(),
    peekHeight: Dp = CustomKeyboardHeight,
    expandedHeight: Dp = 600.dp,
    onBackdropClick: (() -> Unit)? = null,
    content: @Composable (expanded: Boolean) -> Unit
) {
    val density = LocalDensity.current
    val haptic = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()
    val screenReader = screenReaderActive()

    DisposableEffect(state, haptic) {
        state.onSettled = { haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove) }
        onDispose {
            state.onSettled = null
        }
    }

    // With a screen reader the compact detent hides too much of the picker, so
    // Discord presents the sheet at its full height instead.
    LaunchedEffect(screenReader) {
        if (screenReader) state.expand()
    }

    val peekPx = with(density) { peekHeight.toPx() }
    val maxPx = with(density) { expandedHeight.toPx() }
    val travelPx = (maxPx - peekPx).coerceAtLeast(1f)
    val progress = state.progress.value
    val currentHeight = with(density) { lerp(peekPx, maxPx, progress).toDp() }
    val cornerRadius = KeyboardSheetPolicy.topCornerRadius(progress, 16f, 28f).dp
    val scrimAlpha = KeyboardSheetPolicy.scrimAlpha(progress)

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .height(currentHeight),
        shape = RoundedCornerShape(topStart = cornerRadius, topEnd = cornerRadius),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        // Discord's collapsed keyboard has no shadow bleeding over the input.
        shadowElevation = lerp(0f, 12f, progress).dp
    ) {
        Column {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(MaterialTheme.colorScheme.outlineVariant.copy(alpha = scrimAlpha))
            )
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    // Phoenix's generic ActionSheetDragHandle is 20dp tall:
                    // 8dp padding, a 4dp bar, then 8dp padding. That centers
                    // the bar vertically in the slot.
                    .height(20.dp)
                    .keyboardSheetDrag(state, travelPx, scope)
                    .clickable {
                        if (state.expanded) onBackdropClick?.invoke() ?: state.collapse()
                        else state.expand()
                    }
                    .semantics {
                        role = Role.Button
                        contentDescription = if (state.expanded) {
                            "Collapse keyboard panel"
                        } else {
                            "Expand keyboard panel"
                        }
                    },
                contentAlignment = Alignment.Center
            ) {
                Spacer(
                    modifier = Modifier
                        .width(31.dp)
                        .height(4.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f))
                )
            }
            Box(
                modifier = Modifier
                    .weight(1f)
            ) {
                content(state.expanded)
            }
        }
    }
}

@Composable
fun DraggableExpressionSheet(
    modifier: Modifier = Modifier,
    state: KeyboardSheetState = rememberKeyboardSheetState(),
    peekHeight: Dp = CustomKeyboardHeight,
    expandedHeight: Dp = 600.dp,
    onBackdropClick: (() -> Unit)? = null,
    content: @Composable (expanded: Boolean) -> Unit
) {
    DraggableKeyboardSheet(
        modifier = modifier,
        state = state,
        peekHeight = peekHeight,
        expandedHeight = expandedHeight,
        onBackdropClick = onBackdropClick,
        content = content
    )
}
