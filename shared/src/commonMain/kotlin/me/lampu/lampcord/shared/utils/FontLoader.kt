package me.lampu.lampcord.shared.utils

import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.FontStyle
import me.lampu.lampcord.shared.settings.FontOption

expect fun loadFont(
    path: String,
    weight: FontWeight = FontWeight.Normal,
    style: FontStyle = FontStyle.Normal
): Font

fun appFontFamily(option: FontOption, customFontPath: String): FontFamily = when (option) {
    FontOption.SYSTEM -> FontFamily.Default
    FontOption.INTER -> bundledFont("Inter", "otf")
    FontOption.MAPLE_MONO -> bundledFont("MapleMono", "ttf")
    FontOption.CUSTOM -> if (customFontPath.isNotEmpty()) {
        runCatching { FontFamily(loadFont(customFontPath)) }.getOrDefault(FontFamily.Default)
    } else {
        FontFamily.Default
    }
}

private fun bundledFont(prefix: String, extension: String): FontFamily {
    val fonts = listOf(
        "Regular" to FontWeight.Normal,
        "Medium" to FontWeight.Medium,
        "SemiBold" to FontWeight.SemiBold,
        "Bold" to FontWeight.Bold
    ).mapNotNull { (name, weight) ->
        runCatching { loadFont("font/$prefix-$name.$extension", weight) }.getOrNull()
    }
    return if (fonts.isEmpty()) FontFamily.Default else FontFamily(fonts)
}
