package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.unit.Dp

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
    error("Custom keyboard sheet state is not provided")
}

val LocalCustomKeyboardPortal = staticCompositionLocalOf<CustomKeyboardPortal> {
    error("Custom keyboard portal is not provided")
}

val LocalCustomKeyboardGeometry = staticCompositionLocalOf<CustomKeyboardGeometry> {
    error("Custom keyboard geometry is not provided")
}
