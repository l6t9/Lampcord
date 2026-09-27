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

private fun findMatugenCandidateFiles(home: String): List<File> {
    return listOf(
        File("$home/.local/state/quickshell/user/generated/colors.json"),
        File("$home/.config/matugen/colors.json"),
        File("$home/.config/matugen/generated/colors.json"),
        File("$home/.config/matugen/templates/colors.json"),
        File("$home/.cache/matugen/colors.json"),
        File("$home/.cache/matugen/colors-json.json"),
        File("$home/.local/state/matugen/colors.json"),
        File("$home/.local/share/matugen/colors.json"),
        File("$home/.config/hypr/colors.json"),
        File("$home/.cache/wal/colors.json")
    )
}

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    if (remember { getPlatformName() != "linux" }) return null

    val home = remember { System.getProperty("user.home") }
    val candidateFiles = remember(home) { findMatugenCandidateFiles(home) }
    val colorFile = remember(home) { File("$home/.local/state/quickshell/user/generated/color.txt") }
    val alternativeColorFile = remember(home) { File("$home/.cache/wal/colors") }

    fun loadSeed(): Color? {
        for (file in candidateFiles) {
            if (file.exists()) {
                val seed = readMatugenPalette(file)?.seedColor(isDark = false)
                if (seed != null) return seed
            }
        }
        if (colorFile.exists()) {
            try {
                val hex = colorFile.readText().trim().removePrefix("#")
                return Color(hex.toLong(16) or 0xFF000000)
            } catch (_: Exception) {}
        }
        if (alternativeColorFile.exists()) {
            try {
                val firstLine = alternativeColorFile.readLines().firstOrNull()?.removePrefix("#")
                if (firstLine != null) {
                    return Color(firstLine.toLong(16) or 0xFF000000)
                }
            } catch (_: Exception) {}
        }
        return null
    }

    var seedColor by remember { mutableStateOf(loadSeed()) }

    LaunchedEffect(candidateFiles, colorFile, alternativeColorFile) {
        while (true) {
            val foundColor = loadSeed()
            if (seedColor != foundColor) {
                seedColor = foundColor
            }
            delay(1500.milliseconds)
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
    val candidateFiles = remember(home) { findMatugenCandidateFiles(home) }

    fun loadPalette(): MatugenPalette? {
        return candidateFiles.firstNotNullOfOrNull { file ->
            if (file.exists()) readMatugenPalette(file) else null
        }
    }

    var palette by remember { mutableStateOf(loadPalette()) }

    LaunchedEffect(candidateFiles) {
        while (true) {
            val current = loadPalette()
            if (current != palette) {
                palette = current
            }
            delay(1500.milliseconds)
        }
    }

    return palette?.schemeFor(isDark)
}
