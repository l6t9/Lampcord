package me.lampu.lampcord.shared.state

import java.awt.Window
import java.util.Timer
import java.util.TimerTask

private var registered = false

/**
 * Polls AWT window focus. Neither java.awt.event nor KeyboardFocusManager's focus API is exposed
 * on this compile classpath, so a 500ms poll of Window.getWindows() is used instead. The latency
 * is irrelevant for deciding whether a message counts as read.
 */
internal actual fun registerVisibilityListener(onVisible: () -> Unit, onHidden: () -> Unit) {
    if (registered) return
    registered = true

    var last = false

    fun poll() {
        val focused = Window.getWindows().any { it.isShowing && it.isFocused }
        if (focused == last) return
        last = focused
        if (focused) onVisible() else onHidden()
    }

    Timer("lampcord-visibility", true).scheduleAtFixedRate(object : TimerTask() {
        override fun run() {
            poll()
        }
    }, 0L, 500L)
}