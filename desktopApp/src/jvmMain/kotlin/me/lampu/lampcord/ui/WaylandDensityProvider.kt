package me.lampu.lampcord.ui

import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
import androidx.compose.ui.window.WindowScope
import me.lampu.lampcord.utils.WaylandScale
import kotlinx.coroutines.delay
import java.awt.event.ComponentAdapter
import java.awt.event.ComponentEvent

@Composable
fun WindowScope.WaylandDensityProvider(content: @Composable () -> Unit) {
    // Only override density on Wayland. On X11/Windows/macOS the compositor
    // and skiko.uiScale already handle HiDPI; forcing Density(1f) would shrink the UI.
    if (!WaylandScale.isWayland() || me.lampu.lampcord.shared.settings.Settings.shared.disableWaylandScaling) {
        content()
        return
    }

    val window = window // access the AWT window from WindowScope
    var scale by remember { mutableFloatStateOf(WaylandScale.getWindowScale(window.x, window.y)) }

    DisposableEffect(window) {
        val listener =
            object : ComponentAdapter() {
                override fun componentMoved(e: ComponentEvent) {
                    // Instantly check scale when moved
                    val newScale = WaylandScale.getWindowScale(window.x, window.y)
                    if (newScale != scale) {
                        scale = newScale
                    }
                }
                
                override fun componentResized(e: ComponentEvent) {
                    val newScale = WaylandScale.getWindowScale(window.x, window.y)
                    if (newScale != scale) {
                        scale = newScale
                    }
                }
            }

        window.addComponentListener(listener)
        onDispose {
            window.removeComponentListener(listener)
        }
    }

    // Compositors sometimes swallow move events during active drag, so poll the in-memory monitor cache.
    LaunchedEffect(Unit) {
        while (true) {
            delay(200)
            val newScale = WaylandScale.getWindowScale(window.x, window.y)
            if (newScale != scale) {
                scale = newScale
            }
        }
    }

    val density = Density(density = scale, fontScale = 1.0f)
    CompositionLocalProvider(LocalDensity provides density) {
        content()
    }
}
