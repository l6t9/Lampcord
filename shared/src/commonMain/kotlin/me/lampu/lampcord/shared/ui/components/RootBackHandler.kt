package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable

/**
 * Root-only back handler. [onBack] returns true when navigation or an overlay
 * consumed the event; when it returns false, the platform handler performs the
 * double-back exit confirmation.
 */
@Composable
expect fun PlatformRootBackHandler(
    enabled: Boolean,
    onBack: () -> Boolean,
    onExit: () -> Unit,
)

class BackExitConfirmation {
    private var lastBackMillis: Long? = null

    fun onBack(nowMillis: Long): Boolean {
        val previous = lastBackMillis
        if (previous != null && nowMillis - previous in 0L until 2_000L) {
            reset()
            return true
        }
        lastBackMillis = nowMillis
        return false
    }

    fun reset() {
        lastBackMillis = null
    }
}
