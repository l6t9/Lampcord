package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption
import kotlinx.coroutines.delay
import java.io.File

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    val home = remember { System.getProperty("user.home") }
    // These paths are based on the illogical-impulse dotfiles on CachyOS
    val colorFile = remember(home) { File("$home/.local/state/quickshell/user/generated/color.txt") }
    val alternativeColorFile = remember(home) { File("$home/.cache/wal/colors") } // common fallback for pywal
    
    var seedColor by remember { mutableStateOf<Color?>(null) }
    
    LaunchedEffect(colorFile, alternativeColorFile) {
        while (true) {
            var foundColor: Color? = null
            
            if (colorFile.exists()) {
                try {
                    val hex = colorFile.readText().trim().removePrefix("#")
                    foundColor = Color(hex.toLong(16) or 0xFF000000)
                } catch (_: Exception) {}
            }
            
            if (foundColor == null && alternativeColorFile.exists()) {
                try {
                    val firstLine = alternativeColorFile.readLines().firstOrNull()?.removePrefix("#")
                    if (firstLine != null) {
                        foundColor = Color(firstLine.toLong(16) or 0xFF000000)
                    }
                } catch (_: Exception) {}
            }
            
            // TODO: Add GNOME/KDE wallpaper extraction if needed
            
            if (seedColor != foundColor) {
                seedColor = foundColor
            }
            delay(2000)
        }
    }
    
    return seedColor
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
