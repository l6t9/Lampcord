package me.lampu.lampcord.shared.ui.components.sheet

/**
 * Pure geometry and gesture decisions for the two-detent keyboard sheet.
 *
 * Keeping these separate from Compose makes Discord's keyboard-replacement
 * behavior testable without a platform UI: a short collapsed detent, a tall
 * expanded detent, velocity-aware snapping, and haptics only when the detent
 * actually changes.
 */
object KeyboardSheetPolicy {
    const val FLING_VELOCITY_THRESHOLD = 300f
    const val DETENT_THRESHOLD = 0.5f

    /**
     * Discord's custom keyboard geometry, in pixels:
     * - maximum: window height minus the multiline nav bar and safe-area top
     * - minimum: custom keyboard height, capped to the maximum
     * - if the keyboard would consume the whole sheet, step back one nav bar
     */
    fun detents(
        windowHeightPx: Float,
        safeAreaTopPx: Float,
        keyboardHeightPx: Float,
        navBarHeightPx: Float
    ): Pair<Float, Float> {
        val maximum = (windowHeightPx - navBarHeightPx - safeAreaTopPx).coerceAtLeast(0f)
        var minimum = keyboardHeightPx.coerceAtMost(maximum)
        if (minimum >= maximum) {
            minimum = (maximum - navBarHeightPx).coerceAtLeast(0f)
        }
        return minimum to maximum
    }

    /** Upward drags (negative delta) increase progress toward the expanded detent. */
    fun progressAfterDrag(
        currentProgress: Float,
        dragDeltaPx: Float,
        travelPx: Float
    ): Float {
        if (travelPx <= 0f) return currentProgress.coerceIn(0f, 1f)
        return (currentProgress - dragDeltaPx / travelPx).coerceIn(0f, 1f)
    }

    /**
     * Fling direction wins when the gesture is fast enough; otherwise the
     * sheet settles to the detent it is already closest to.
     */
    fun snapTarget(
        currentProgress: Float,
        velocityPxPerSecond: Float
    ): Float = when {
        velocityPxPerSecond < -FLING_VELOCITY_THRESHOLD -> 1f
        velocityPxPerSecond > FLING_VELOCITY_THRESHOLD -> 0f
        else -> if (currentProgress >= DETENT_THRESHOLD) 1f else 0f
    }

    fun crossedDetent(before: Float, after: Float): Boolean =
        (before >= DETENT_THRESHOLD) != (after >= DETENT_THRESHOLD)

    fun topCornerRadius(
        progress: Float,
        collapsedDp: Float,
        expandedDp: Float
    ): Float = collapsedDp + (expandedDp - collapsedDp) * progress.coerceIn(0f, 1f)

    fun scrimAlpha(progress: Float): Float = 0.06f + 0.10f * progress.coerceIn(0f, 1f)

    /**
     * Discord samples the settled IME height, never an animation frame. Android
     * exposes the animation's source and target separately:
     * - opening: target > source, so the target is the final keyboard height
     * - settled: source == target, so the current inset is already final
     * - closing: target < source, so we must not learn the shrinking frames
     */
    fun stableSystemKeyboardHeight(currentPx: Int, sourcePx: Int, targetPx: Int): Int =
        when {
            targetPx > sourcePx -> targetPx
            targetPx == sourcePx && currentPx > 0 -> currentPx
            else -> 0
        }

    /**
     * Discord Android's channel bottom safe area:
     * - custom keyboard: the custom sheet's collapsed detent
     * - system keyboard: the measured IME inset alone, never nav bar + IME
     * - resting: max(navigation bar inset, CHAT_INPUT_FLOATING_OFFSET_MINIMUM)
     */
    fun channelBottomInset(
        systemKeyboardHeightPx: Float,
        navigationBottomPx: Float,
        customKeyboardHeightPx: Float,
        customKeyboardVisible: Boolean,
        minimumInsetPx: Float
    ): Float = when {
        customKeyboardVisible -> customKeyboardHeightPx
        systemKeyboardHeightPx > 0f -> systemKeyboardHeightPx
        else -> maxOf(navigationBottomPx, minimumInsetPx)
    }

    /** Full-window backdrop darkens as the custom keyboard approaches full height. */
    fun backdropAlpha(progress: Float): Float = 0.12f + 0.28f * progress.coerceIn(0f, 1f)
}
