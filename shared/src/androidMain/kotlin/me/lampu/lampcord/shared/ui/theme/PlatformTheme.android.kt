package me.lampu.lampcord.shared.ui.theme

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.os.Build
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontFamily
import androidx.core.view.WindowCompat
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.utils.appFontFamily

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
actual fun rememberAppFontFamily(option: FontOption, customFontPath: String): FontFamily =
    remember(option, customFontPath) { appFontFamily(option, customFontPath) }

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
    val navigationStore: me.lampu.lampcord.shared.state.NavigationStore = org.koin.compose.koinInject()
    val isOverlapping = me.lampu.lampcord.shared.settings.Settings.shared.panelType == me.lampu.lampcord.shared.settings.PanelType.OVERLAPPING

    if (!view.isInEditMode) {
        SideEffect {
            val activity = view.context.findActivity()
            if (activity != null) {
                val window = activity.window
                val isAppearanceLight = !isDark
                
                val chatCentered = !navigationStore.isSettingsVisible &&
                    !navigationStore.isChannelsAndRolesVisible

                val statusBarColor = when {
                    navigationStore.isSettingsVisible -> scheme.surface
                    navigationStore.isChannelsAndRolesVisible -> scheme.surfaceContainerHigh
                    chatCentered && !isOverlapping -> scheme.surfaceContainer
                    else -> scheme.background
                }
                
                @Suppress("DEPRECATION")
                window.statusBarColor = statusBarColor.toArgb()
                WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = isAppearanceLight
                WindowCompat.getInsetsController(window, view).isAppearanceLightNavigationBars = isAppearanceLight
            }
        }
    }
    
    return scheme
}

private fun Color.toArgb(): Int {
    return (this.alpha * 255.0f + 0.5f).toInt() shl 24 or
           ((this.red * 255.0f + 0.5f).toInt() shl 16) or
           ((this.green * 255.0f + 0.5f).toInt() shl 8) or
           (this.blue * 255.0f + 0.5f).toInt()
}
