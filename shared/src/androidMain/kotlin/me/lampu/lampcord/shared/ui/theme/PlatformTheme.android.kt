package me.lampu.lampcord.shared.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontFamily
import androidx.core.view.WindowCompat
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
    val scheme = rememberDynamicColorScheme(
        seedColor = seedColor,
        isDark = isDark,
        style = paletteStyle
    )

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            // Use surfaceContainerHigh (Guild Rail background) for status bar
            val statusBarColor = if (isDark) DiscordClassicGuildRail else scheme.surfaceContainerHigh
            window.statusBarColor = statusBarColor.toArgb()
            val navBarColor = if (isDark) DiscordClassicGuildRail else scheme.surfaceContainerHigh
            window.navigationBarColor = navBarColor.toArgb()
            
            WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = !isDark
            WindowCompat.getInsetsController(window, view).isAppearanceLightNavigationBars = !isDark
        }
    }
    
    return scheme
}
