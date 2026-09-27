package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.utils.appFontFamily
import me.lampu.lampcord.shared.utils.getPlatformName
import kotlinx.coroutines.delay
import java.io.File
import kotlin.time.Duration.Companion.milliseconds

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    // These color files are Linux-specific. Do not keep a background polling
    // coroutine alive on Windows or macOS.
    if (remember { getPlatformName() != "linux" }) return null

    val home = remember { System.getProperty("user.home") }
    // The dots' matugen config writes the current scheme to the quickshell generated
    // folder; stock matugen uses ~/.cache/matugen/colors.json. Prefer the first.
    val generatedColorsFile = remember(home) { File("$home/.local/state/quickshell/user/generated/colors.json") }
    val matugenFile = remember(home) { File("$home/.cache/matugen/colors.json") }
    val colorFile = remember(home) { File("$home/.local/state/quickshell/user/generated/color.txt") }
    val alternativeColorFile = remember(home) { File("$home/.cache/wal/colors") } // common fallback for pywal

    var seedColor by remember { mutableStateOf<Color?>(null) }

    LaunchedEffect(generatedColorsFile, matugenFile, colorFile, alternativeColorFile) {
        while (true) {
            var foundColor: Color? = null

            // Matugen takes priority: it holds the full wallpaper-derived palette.
            if (foundColor == null && generatedColorsFile.exists()) {
                try {
                    foundColor = readMatugenPalette(generatedColorsFile)?.seedColor(isDark = false)
                } catch (_: Exception) {}
            }
            if (foundColor == null && matugenFile.exists()) {
                try {
                    foundColor = readMatugenPalette(matugenFile)?.seedColor(isDark = false)
                } catch (_: Exception) {}
            }

            if (foundColor == null && colorFile.exists()) {
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
            delay(2000.milliseconds)
        }
    }
    
    return seedColor
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
    // On Linux, when the dynamic (Material You) theme is enabled, use matugen's
    // generated palette directly so the app matches the wallpaper colors exactly.
    // Windows/macOS keep using the seed-derived scheme (Windows has its own
    // system accent color handling).
    val matugen = if (useMaterialYou) rememberMatugenColorScheme(isDark) else null
    if (matugen != null) return matugen

    return rememberDynamicColorScheme(
        seedColor = seedColor,
        isDark = isDark,
        style = paletteStyle
    )
}

@Composable
private fun rememberMatugenColorScheme(isDark: Boolean): ColorScheme? {
    if (remember { getPlatformName() != "linux" }) return null
    val home = remember { System.getProperty("user.home") }
    val candidateFiles = remember(home) {
        listOf(
            File("$home/.local/state/quickshell/user/generated/colors.json"),
            File("$home/.cache/matugen/colors.json")
        )
    }

    var palette by remember { mutableStateOf<MatugenPalette?>(null) }
    LaunchedEffect(candidateFiles) {
        while (true) {
            palette = candidateFiles.firstNotNullOfOrNull { file ->
                if (file.exists()) readMatugenPalette(file) else null
            }
            delay(2000.milliseconds)
        }
    }

    return palette?.schemeFor(isDark)
}
