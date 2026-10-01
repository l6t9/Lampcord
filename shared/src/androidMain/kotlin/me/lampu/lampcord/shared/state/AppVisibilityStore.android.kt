package me.lampu.lampcord.shared.state

/**
 * Driven by MainActivity.onStart/onStop. Activity callbacks are the only reliable signal here:
 * there is no custom Application class to hang ProcessLifecycleOwner off.
 */
internal actual fun registerVisibilityListener(onVisible: () -> Unit, onHidden: () -> Unit) {
}