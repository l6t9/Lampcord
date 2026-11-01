package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent

private const val FRAME_INTERVAL_NANOS = 33_000_000L

@Composable
internal actual fun Modifier.animateWhilePlaying(enabled: Boolean): Modifier {
    if (!enabled) return this

    var tick by remember { mutableIntStateOf(0) }
    LaunchedEffect(enabled) {
        var previous = withFrameNanos { it }
        while (true) {
            val now = withFrameNanos { it }
            if (now - previous >= FRAME_INTERVAL_NANOS) {
                previous = now
                tick++
            }
        }
    }

    return this.drawWithContent {
        tick
        drawContent()
    }
}
