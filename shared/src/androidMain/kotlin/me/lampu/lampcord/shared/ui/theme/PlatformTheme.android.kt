package me.lampu.lampcord.shared.ui.theme

import android.os.Build
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val context = LocalContext.current
        val colorScheme = dynamicLightColorScheme(context)
        return colorScheme.primary
    }
    return null
}

@Composable
actual fun rememberAppFontFamily(option: FontOption, customFontPath: String): FontFamily {
    return when (option) {
        FontOption.CUSTOM -> {
            if (customFontPath.isNotEmpty()) {
                try {
                    FontFamily(me.lampu.lampcord.shared.utils.loadFont(customFontPath))
                } catch (e: Exception) {
                    FontFamily.Default
                }
            } else {
                FontFamily.Default
            }
        }
        else -> FontFamily.Default
    }
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
