package me.lampu.lampcord.shared.utils

import androidx.compose.ui.graphics.Color
import kotlin.math.abs

object ColorUtils {
    fun colorToHsv(color: Color): FloatArray {
        val r = color.red
        val g = color.green
        val b = color.blue

        val max = maxOf(r, g, b)
        val min = minOf(r, g, b)
        val delta = max - min

        var h = 0f
        if (delta != 0f) {
            h = when (max) {
                r -> (g - b) / delta % 6f
                g -> (b - r) / delta + 2f
                else -> (r - g) / delta + 4f
            }
            h *= 60f
            if (h < 0) h += 360f
        }

        val s = if (max == 0f) 0f else delta / max

        return floatArrayOf(h, s, max)
    }

    fun hsvToColor(h: Float, s: Float, v: Float, alpha: Float = 1f): Color {
        val c = v * s
        val x = c * (1f - abs((h / 60f % 2f) - 1f))
        val m = v - c

        val (r, g, b) = when {
            h < 60f -> Triple(c, x, 0f)
            h < 120f -> Triple(x, c, 0f)
            h < 180f -> Triple(0f, c, x)
            h < 240f -> Triple(0f, x, c)
            h < 300f -> Triple(x, 0f, c)
            else -> Triple(c, 0f, x)
        }

        return Color(r + m, g + m, b + m, alpha)
    }
}
