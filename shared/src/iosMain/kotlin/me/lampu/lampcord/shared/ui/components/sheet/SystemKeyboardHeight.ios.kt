package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Density

@Composable
actual fun systemKeyboardHeightPx(density: Density): Int =
    WindowInsets.ime.getBottom(density)
