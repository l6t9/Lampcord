package me.lampu.lampcord.shared.ui.theme

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.os.Build
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontFamily
import androidx.core.view.WindowCompat
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption

private fun Context.findActivity(): Activity? {
    var context = this
    while (context is ContextWrapper) {
        if (context is Activity) return context
        context = context.baseContext
    }
    return null
}

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
            val activity = view.context.findActivity()
            if (activity != null) {
                val window = activity.window
                WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = !isDark
                WindowCompat.getInsetsController(window, view).isAppearanceLightNavigationBars = !isDark
            }
        }
    }
    
    return scheme
}
