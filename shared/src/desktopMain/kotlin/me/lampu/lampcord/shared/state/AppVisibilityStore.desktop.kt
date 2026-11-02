package me.lampu.lampcord.shared.state

/**
 * Desktop window visibility is driven by the Nucleus window focus flow in the desktop
 * entry point.
 *
 * Do not use AWT here: touching `java.awt.Window` at startup loads `libawt_xawt`, which
 * puts `GDK_BACKEND=x11` into the process environment before Tao opens GTK. That forces
 * the whole app onto XWayland, where a 2x monitor only gets an upscaled 1x buffer.
 */
internal actual fun registerVisibilityListener(onVisible: () -> Unit, onHidden: () -> Unit) {
}
