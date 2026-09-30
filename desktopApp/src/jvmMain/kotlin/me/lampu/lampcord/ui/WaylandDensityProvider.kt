package me.lampu.lampcord.ui

import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
import dev.nucleusframework.application.LocalNucleusWindow
import me.lampu.lampcord.utils.WaylandScale
import kotlinx.coroutines.delay

@Composable
fun WaylandDensityProvider(content: @Composable () -> Unit) {
    if (!WaylandScale.isWayland() || !me.lampu.lampcord.shared.settings.Settings.shared.enableWaylandScaling) {
        content()
        return
    }

    val window = LocalNucleusWindow.current
    var scale by remember { mutableFloatStateOf(WaylandScale.getWindowScale(0, 0)) }

    LaunchedEffect(Unit) {
        while (true) {
            delay(200)
            val newScale = WaylandScale.getWindowScale(0, 0)
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
