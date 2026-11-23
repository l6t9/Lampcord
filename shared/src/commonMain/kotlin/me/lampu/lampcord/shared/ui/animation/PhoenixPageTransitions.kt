package me.lampu.lampcord.shared.ui.animation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically

internal val PhoenixStandardEasing = CubicBezierEasing(0.4f, 0f, 0.2f, 1f)

internal object PhoenixPageTransitions {
    private const val PushDuration = 300
    private const val FadeDuration = 200
    private const val CollapsedScale = 0.94f

    fun pushEnter(): EnterTransition =
        slideInHorizontally(tween(PushDuration, easing = PhoenixStandardEasing)) { it } +
            fadeIn(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun pushExit(): ExitTransition =
        slideOutHorizontally(tween(PushDuration, easing = PhoenixStandardEasing)) { -it / 3 } +
            fadeOut(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun popEnter(): EnterTransition =
        slideInHorizontally(tween(PushDuration, easing = PhoenixStandardEasing)) { -it / 3 } +
            fadeIn(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun popExit(): ExitTransition =
        slideOutHorizontally(tween(PushDuration, easing = PhoenixStandardEasing)) { it } +
            fadeOut(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun bringUpEnter(): EnterTransition =
        slideInVertically(spring(dampingRatio = 0.78f, stiffness = 300f)) { it } +
            fadeIn(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun bringUpExit(): ExitTransition =
        scaleOut(
            targetScale = CollapsedScale,
            animationSpec = spring(dampingRatio = 0.78f, stiffness = 300f)
        ) + fadeOut(tween(150, easing = PhoenixStandardEasing))

    fun bringDownEnter(): EnterTransition =
        scaleIn(
            initialScale = CollapsedScale,
            animationSpec = spring(dampingRatio = 0.78f, stiffness = 300f)
        ) + fadeIn(tween(FadeDuration, easing = PhoenixStandardEasing))

    fun bringDownExit(): ExitTransition =
        slideOutVertically(spring(dampingRatio = 0.78f, stiffness = 300f)) { it } +
            fadeOut(tween(150, easing = PhoenixStandardEasing))
}
