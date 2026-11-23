@file:OptIn(ExperimentalLayoutApi::class)

package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imeAnimationSource
import androidx.compose.foundation.layout.imeAnimationTarget
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Density

@Composable
actual fun systemKeyboardHeightPx(density: Density): Int {
    // Discord's getImeInsets(false) reports the settled keyboard height. Compose's
    // ime inset is animated, so using it directly persisted shrinking frames while
    // the keyboard hid and left the custom keyboard at the 200dp floor. The
    // animation source/target tells us whether the keyboard is opening, settled,
    // or closing, so only the final opening height is ever learned.
    return KeyboardSheetPolicy.stableSystemKeyboardHeight(
        currentPx = WindowInsets.ime.getBottom(density),
        sourcePx = WindowInsets.imeAnimationSource.getBottom(density),
        targetPx = WindowInsets.imeAnimationTarget.getBottom(density)
    )
}
