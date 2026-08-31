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
fun WindowScope.WaylandDensityProvider(
    content: @Composable () -> Unit
) {
    // Wayland scaling is Linux-only. Avoid starting listeners and a polling
    // coroutine on Windows and macOS, where the scale is always the default.
    val isWayland = remember { WaylandScale.isWayland() }
    if (!isWayland) {
        content()
        return
    }

    var scale by remember { mutableFloatStateOf(1.0f) }
    val window = window // access the AWT window from WindowScope

    // Cache scale to avoid redundant recompositions
    LaunchedEffect(window) {
        // Initial detection
        scale = WaylandScale.getWindowScale(window.x, window.y)
    }

    DisposableEffect(window) {
        val listener = object : ComponentAdapter() {
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

    // Faster polling (200ms) for smoother transitions during dragging
    // Compositors sometimes "swallow" move events during active drag
    LaunchedEffect(Unit) {
        while (true) {
            kotlinx.coroutines.delay(200)
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
