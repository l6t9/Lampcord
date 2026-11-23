package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** One source of truth for the sheet detents used by both the input and the sheet. */
data class CustomKeyboardGeometry(val minimum: Dp, val maximum: Dp)

/**
 * ChatInputBar owns the picker content; MobileBaseplate owns the sheet window.
 * This portal connects them so the sheet is rendered in the same window as the
 * chat input instead of in a separate popup window.
 */
class CustomKeyboardPortal {
    private val _content = mutableStateOf<(@Composable (expanded: Boolean) -> Unit)?>(null)

    var content: (@Composable (expanded: Boolean) -> Unit)?
        get() = _content.value
        set(value) {
            _content.value = value
        }
}

val LocalCustomKeyboardSheetState = staticCompositionLocalOf<KeyboardSheetState> {
    KeyboardSheetState.Disabled
}

val LocalCustomKeyboardPortal = staticCompositionLocalOf<CustomKeyboardPortal> {
    CustomKeyboardPortal()
}

val LocalCustomKeyboardGeometry = staticCompositionLocalOf<CustomKeyboardGeometry> {
    CustomKeyboardGeometry(0.dp, 0.dp)
}
