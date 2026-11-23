package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Density

/**
 * The stable system-keyboard height in pixels.
 *
 * Discord samples this when the keyboard finishes showing, then reuses the
 * value for custom keyboards. On Android the animated IME inset can report
 * partial or shrinking frames, so platform code uses the animation source and
 * target to return only the settled opening height.
 */
@Composable
expect fun systemKeyboardHeightPx(density: Density): Int
