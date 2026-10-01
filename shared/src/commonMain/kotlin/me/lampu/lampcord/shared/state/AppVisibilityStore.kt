package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Whether the UI is currently on screen and able to receive input.
 *
 * Backgrounding a window does not dispose the Compose composition, so a chat view stays attached
 * and would otherwise keep acknowledging messages that arrive while the user is in another app.
 * Discord gates acknowledgements on the chat list being attached to a LinearLayoutManager, which
 * is only ever populated while the activity is resumed. This is the equivalent signal.
 */
object AppVisibilityStore {
    private val _isVisible = MutableStateFlow(true)

    val isVisible: StateFlow<Boolean> = _isVisible.asStateFlow()

    val isVisibleNow: Boolean get() = _isVisible.value

    /** Public so platform hosts (MainActivity) can drive visibility directly. */
    fun setVisible(visible: Boolean) {
        _isVisible.value = visible
    }

    /** Idempotent: repeated calls do not stack listeners. */
    fun start() = registerVisibilityListener(
        onVisible = { setVisible(true) },
        onHidden = { setVisible(false) }
    )
}

/**
 * Mirrors Discord's StoreChat.InteractionState: the chat view reports which channel it is showing
 * and whether the user is scrolled to the bottom. Discord requires both, plus a live
 * LinearLayoutManager, before it acknowledges anything.
 */
data class ChatInteraction(val channelId: String, val atBottom: Boolean)

internal expect fun registerVisibilityListener(onVisible: () -> Unit, onHidden: () -> Unit)