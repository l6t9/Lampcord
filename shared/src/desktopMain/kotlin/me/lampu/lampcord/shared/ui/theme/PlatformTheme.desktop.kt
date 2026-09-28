package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import dev.nucleusframework.systemcolor.systemAccentColor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.utils.appFontFamily
import me.lampu.lampcord.shared.utils.getPlatformName
import java.io.File
import java.nio.file.FileSystems
import java.nio.file.StandardWatchEventKinds
import java.util.concurrent.TimeUnit

private val THEME_JSON = Json { ignoreUnknownKeys = true }

private fun findMatugenCandidateFiles(home: String): List<File> = listOf(
    File("$home/.config/matugen/colors.json"),
    File("$home/.config/matugen/generated/colors.json"),
    File("$home/.config/matugen/templates/colors.json"),
    File("$home/.local/state/matugen/colors.json"),
    File("$home/.cache/matugen/colors-json.json"),
    File("$home/.local/state/quickshell/user/generated/colors.json"),
    File("$home/.local/state/quickshell/user/generated/color.txt")
)

private fun camelToSnake(key: String): String = buildString {
    key.forEach { ch ->
        if (ch.isUpperCase()) {
            append('_')
            append(ch.lowercaseChar())
        } else {
            append(ch)
        }
    }
}

private fun String.toThemeColor(): Color? {
    val hex = trim().removePrefix("#")
    if (hex.length != 6 && hex.length != 8) return null
    val value = hex.toLongOrNull(16) ?: return null
    return if (hex.length == 6) Color(value or 0xFF000000) else Color(value)
}

/**
 * Caelestia derives its palette from the wallpaper and keeps the active scheme in
 * `~/.local/state/caelestia/scheme.json`, which is rewritten whenever the wallpaper changes.
 */
private class CaelestiaScheme(val palette: MatugenPalette, val isDark: Boolean)

private fun readCaelestiaScheme(stateDir: File): CaelestiaScheme? {
    val file = File(stateDir, "scheme.json")
    if (!file.isFile) return null
    val text = try {
        file.readText()
    } catch (_: Exception) {
        return null
    }
    val root = try {
        THEME_JSON.parseToJsonElement(text).jsonObject
    } catch (_: Exception) {
        return null
    }
    val colours = root["colours"]?.jsonObject ?: return null

    val roles = HashMap<String, String>()
    for ((key, element) in colours) {
        val hex = element.jsonPrimitive.contentOrNullSafe() ?: continue
        roles[camelToSnake(key)] = hex
    }
    if (roles.isEmpty()) return null

    val isDark = root["mode"]?.jsonPrimitive?.contentOrNullSafe() != "light"
    return CaelestiaScheme(
        palette = MatugenPalette(light = roles, dark = roles),
        isDark = isDark
    )
}

private fun kotlinx.serialization.json.JsonPrimitive.contentOrNullSafe(): String? =
    try {
        content
    } catch (_: Exception) {
        null
    }

private suspend fun loadCaelestiaScheme(stateDir: File): CaelestiaScheme? =
    withContext(Dispatchers.IO) { readCaelestiaScheme(stateDir) }

private fun readMatugenPaletteFile(file: File): MatugenPalette? {
    if (!file.isFile) return null
    return try {
        readMatugenPalette(file)
    } catch (_: Exception) {
        null
    }
}

private suspend fun loadMatugenPalette(candidateFiles: List<File>): MatugenPalette? =
    withContext(Dispatchers.IO) {
        candidateFiles.firstNotNullOfOrNull { readMatugenPaletteFile(it) }
    }

private suspend fun loadMatugenSeed(candidateFiles: List<File>): Color? =
    withContext(Dispatchers.IO) {
        for (file in candidateFiles) {
            if (file.isFile) {
                readMatugenPaletteFile(file)?.seedColor(isDark = false)?.let { return@withContext it }
            }
        }
        for (file in candidateFiles) {
            if (file.isFile && file.extension.isEmpty()) {
                try {
                    val hex = file.readLines().firstOrNull()?.trim()
                    hex?.toThemeColor()?.let { return@withContext it }
                } catch (_: Exception) {
                }
            }
        }
        null
    }

private class ThemeSources(
    val caelestia: CaelestiaScheme? = null,
    val matugen: MatugenPalette? = null,
    val caelestiaSeed: Color? = null,
    val matugenSeed: Color? = null
)

/**
 * Watches the directories holding the palette sources and fires when one of [watchedNames]
 * is created, modified or removed. Used instead of polling so theme changes are picked up
 * as soon as the shell writes them.
 */
private fun watchThemeFiles(
    dirs: List<File>,
    watchedNames: Set<String>,
    onChanged: suspend () -> Unit
) {
    val existing = dirs.filter { it.isDirectory }
    if (existing.isEmpty()) return

    val watchService = FileSystems.getDefault().newWatchService()
    existing.forEach { dir ->
        try {
            dir.toPath().register(
                watchService,
                StandardWatchEventKinds.ENTRY_CREATE,
                StandardWatchEventKinds.ENTRY_MODIFY,
                StandardWatchEventKinds.ENTRY_DELETE
            )
        } catch (_: Exception) {
        }
    }

    Thread({
        try {
            while (!Thread.currentThread().isInterrupted) {
                // The timeout only bounds the blocking wait so the thread can notice interruption.
                val key = watchService.poll(30, TimeUnit.SECONDS) ?: continue
                var changed = false
                for (event in key.pollEvents()) {
                    val context = event.context()
                    if (context is File && context.name in watchedNames) changed = true
                }
                key.reset()
                if (changed) {
                    kotlinx.coroutines.runBlocking { onChanged() }
                }
            }
        } catch (_: InterruptedException) {
        } catch (_: Exception) {
        } finally {
            try {
                watchService.close()
            } catch (_: Exception) {
            }
        }
    }, "lampcord-theme-watch").apply { isDaemon = true }.start()
}

@Composable
private fun rememberThemeSources(enabled: Boolean): ThemeSources {
    val home = remember { System.getProperty("user.home") }
    val caelestiaStateDir = remember(home) { File("$home/.local/state/caelestia") }
    val matugenFiles = remember(home) { findMatugenCandidateFiles(home) }

    var sources by remember { mutableStateOf(ThemeSources()) }

    LaunchedEffect(enabled, caelestiaStateDir, matugenFiles) {
        if (!enabled) {
            sources = ThemeSources()
            return@LaunchedEffect
        }

        suspend fun reload() {
            val caelestia = loadCaelestiaScheme(caelestiaStateDir)
            sources = ThemeSources(
                caelestia = caelestia,
                matugen = loadMatugenPalette(matugenFiles),
                caelestiaSeed = caelestia?.palette?.seedColor(caelestia.isDark),
                matugenSeed = loadMatugenSeed(matugenFiles)
            )
        }

        reload()

        watchThemeFiles(
            dirs = listOf(
                caelestiaStateDir,
                File(caelestiaStateDir, "wallpaper")
            ) + matugenFiles.mapNotNull { it.parentFile },
            watchedNames = setOf(
                "scheme.json", "current", "path.txt", "thumbnail.jpg",
                "colors.json", "color.txt", "colors-json.json"
            )
        ) {
            reload()
        }
    }

    return if (enabled) sources else ThemeSources()
}

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    if (remember { getPlatformName() != "linux" }) return null

    val sources = rememberThemeSources(enabled = true)

    // Caelestia already derives a full palette from the wallpaper, so it is the best source.
    sources.caelestiaSeed?.let { return it }

    // Next best: the OS accent, which Nucleus observes natively and pushes as a change
    // notification, so it recomposes without any polling. Null when unavailable (e.g. a
    // Wayland session without a Settings portal, which is the common Linux case).
    systemAccentColor()?.let { return it }

    // Last resort: a wallpaper-generated palette file, kept live by a file watcher.
    return sources.matugenSeed
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
    val sources = rememberThemeSources(enabled = useMaterialYou)

    // Prefer an exact match with the shell palette when its mode matches the app's.
    // A full palette carries every role, so it beats deriving one from a single seed.
    sources.caelestia
        ?.takeIf { it.isDark == isDark }
        ?.palette
        ?.schemeFor(isDark)
        ?.let { return it }

    if (useMaterialYou) {
        sources.matugen?.schemeFor(isDark)?.let { return it }
    }

    return rememberDynamicColorScheme(
        seedColor = seedColor,
        isDark = isDark,
        style = paletteStyle
    )
}
