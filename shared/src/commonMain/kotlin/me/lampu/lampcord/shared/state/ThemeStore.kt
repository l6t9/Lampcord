package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import kotlinx.coroutines.CoroutineScope
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import me.lampu.lampcord.shared.model.LampcordTheme
import me.lampu.lampcord.shared.settings.Settings

class ThemeStore(
    private val scope: CoroutineScope
) {
    var activeTheme by mutableStateOf<LampcordTheme?>(null)
    var availableThemes = mutableStateOf<List<LampcordTheme>>(emptyList())

    private val json = Json { ignoreUnknownKeys = true }

    init {
        val savedTheme = Settings.shared.activeThemeJson
        if (savedTheme.isNotBlank()) {
            loadThemeFromJson(savedTheme, persist = false)
        }
    }

    fun loadThemeFromJson(jsonString: String, persist: Boolean = true) {
        try {
            val theme = json.decodeFromString<LampcordTheme>(jsonString)
            activeTheme = theme
            if (persist) {
                Settings.shared.activeThemeJson = jsonString
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun resolveColor(colorName: String, colorScheme: androidx.compose.material3.ColorScheme): Color? {
        val theme = activeTheme ?: return null
        
        val keys = when (colorName) {
            "primary" -> listOf("accent", "brand_new_500", "brand_500")
            "onPrimary" -> listOf("primary_dark_100", "white")
            "primaryContainer" -> listOf("brand_new_500_alpha_20", "brand_500_alpha_20")
            "secondary" -> listOf("accent2", "brand_new_360", "brand_360")
            "background" -> listOf("background", "primary_dark_600")
            "onBackground" -> listOf("primary_dark_100", "primary_dark_300")
            "surface" -> listOf("background_secondary", "primary_dark_800")
            "onSurface" -> listOf("primary_dark_100", "primary_dark_300")
            "surfaceVariant" -> listOf("background_tertiary", "primary_dark_900")
            "onSurfaceVariant" -> listOf("primary_dark_300", "primary_dark_400")
            "outline" -> listOf("primary_dark_400")
            "outlineVariant" -> listOf("primary_dark_500")
            else -> listOf(colorName)
        }

        for (key in keys) {
            val element = theme.colors[key] ?: theme.simple_colors[key]
            if (element != null) {
                val colorValue = if (element is JsonPrimitive) element.content else element.toString()
                parseColorValue(colorValue, colorScheme)?.let { return it }
            }
        }
        
        return null
    }

    private fun parseColorValue(value: String, colorScheme: androidx.compose.material3.ColorScheme): Color? {
        if (value.startsWith("#")) {
            return try {
                val hex = value.removePrefix("#")
                if (hex.length == 6) {
                    Color(hex.toLong(16) or 0xFF000000)
                } else {
                    Color(hex.toLong(16))
                }
            } catch (e: Exception) {
                null
            }
        }
        
        if (value.startsWith("system_")) {
            return when {
                value.contains("accent1") -> colorScheme.primary
                value.contains("accent2") -> colorScheme.secondary
                value.contains("accent3") -> colorScheme.tertiary
                value.contains("neutral1") -> colorScheme.surface
                value.contains("neutral2") -> colorScheme.surfaceVariant
                else -> null
            }
        }
        
        // Also support integer colors (Aliucord format)
        return try {
            // Aliucord uses 32-bit signed integers for colors
            val intColor = value.toInt()
            Color(intColor)
        } catch (e: Exception) {
            try {
                val longColor = value.toLong()
                if (longColor > 0xFFFFFFFFL) Color(longColor) else Color(longColor.toInt())
            } catch (e2: Exception) {
                null
            }
        }
    }
}
