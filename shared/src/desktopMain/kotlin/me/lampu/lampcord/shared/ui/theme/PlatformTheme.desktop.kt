package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption

@Composable
actual fun rememberDynamicSeedColor(): Color? = null

@Composable
actual fun rememberAppFontFamily(option: FontOption): FontFamily {
    return FontFamily.Default // Simplified for now
}

@Composable
actual fun rememberPlatformColorScheme(
    seedColor: Color,
    isDark: Boolean,
    paletteStyle: PaletteStyle,
    useMaterialYou: Boolean
): ColorScheme {
    return rememberDynamicColorScheme(
        seedColor = seedColor,
        isDark = isDark,
        style = paletteStyle
    )
}
