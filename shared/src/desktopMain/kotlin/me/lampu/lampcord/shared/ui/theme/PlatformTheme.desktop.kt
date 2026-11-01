package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import dev.nucleusframework.systemcolor.systemAccentColor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.utils.appFontFamily
import me.lampu.lampcord.shared.utils.getPlatformName
import java.io.File
import java.nio.file.FileSystems
import java.nio.file.StandardWatchEventKinds
import java.util.concurrent.TimeUnit

private fun getXdgDir(envVar: String, defaultRelPath: String, home: String): File {
    val env = System.getenv(envVar)
    return if (!env.isNullOrBlank()) File(env) else File(home, defaultRelPath)
}

private fun findMatugenCandidateFiles(home: String): List<File> {
    val configHome = getXdgDir("XDG_CONFIG_HOME", ".config", home)
    val stateHome = getXdgDir("XDG_STATE_HOME", ".local/state", home)
    val cacheHome = getXdgDir("XDG_CACHE_HOME", ".cache", home)

    val searchRoots = listOf(
        File(configHome, "matugen"),
        File(stateHome, "matugen"),
        File(cacheHome, "matugen"),
        File(configHome, "quickshell"),
        File(stateHome, "quickshell"),
        File(cacheHome, "quickshell")
    )

    val targetNames = setOf("colors.json", "color.txt", "colors-json.json", "scheme.json")
    val found = mutableListOf<File>()

    for (root in searchRoots) {
        if (!root.isDirectory) continue
        try {
            root.walkTopDown()
                .maxDepth(4)
                .filter { it.isFile && it.name in targetNames }
                .forEach { found.add(it) }
        } catch (_: Exception) {
        }
    }

    return found.distinct().sortedByDescending { it.lastModified() }
}

private fun String.toThemeColor(): Color? {
    val hex = trim().removePrefix("#")
    if (hex.length != 6 && hex.length != 8) return null
    val value = hex.toLongOrNull(16) ?: return null
    return if (hex.length == 6) Color(value or 0xFF000000) else Color(value)
}

private fun readMatugenPaletteFile(file: File): MatugenPalette? {
    if (!file.isFile) return null
    return try {
        readMatugenPalette(file)
    } catch (_: Exception) {
        null
    }
}

private suspend fun loadMatugenSeed(candidateFiles: List<File>): Color? =
    withContext(Dispatchers.IO) {
        for (file in candidateFiles) {
            if (file.isFile) {
                readMatugenPaletteFile(file)?.seedColor(isDark = false)?.let { return@withContext it }
            }
        }
        for (file in candidateFiles) {
            if (file.isFile && (file.extension.isEmpty() || file.extension == "txt")) {
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
    val matugenSeed: Color? = null
)

private fun watchThemeFiles(
    dirs: List<File>,
    watchedNames: Set<String>,
    onChanged: suspend () -> Unit
): Thread? {
    val existing = dirs.filter { it.isDirectory }
    if (existing.isEmpty()) return null

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

    return Thread({
        try {
            while (!Thread.currentThread().isInterrupted) {

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
    }, "lampcord-theme-watch").apply { isDaemon = true }.also { it.start() }
}

@Composable
private fun rememberThemeSources(enabled: Boolean): ThemeSources {
    val home = remember { System.getProperty("user.home") }
    val matugenFiles = remember(home) { findMatugenCandidateFiles(home) }

    var sources by remember { mutableStateOf(ThemeSources()) }

    LaunchedEffect(enabled, matugenFiles) {
        if (!enabled) {
            sources = ThemeSources()
            return@LaunchedEffect
        }

        suspend fun reload() {
            sources = ThemeSources(
                matugenSeed = loadMatugenSeed(matugenFiles)
            )
        }

        reload()

        val watcher = watchThemeFiles(
            dirs = matugenFiles.mapNotNull { it.parentFile },
            watchedNames = setOf(
                "colors.json", "color.txt", "colors-json.json", "scheme.json"
            )
        ) {
            reload()
        }

        try {
            while (true) {
                kotlinx.coroutines.delay(30_000)
            }
        } finally {
            watcher?.interrupt()
            watcher?.join(1_000)
        }
    }

    return if (enabled) sources else ThemeSources()
}

@Composable
actual fun rememberDynamicSeedColor(): Color? {
    if (remember { getPlatformName() != "linux" }) return null

    val sources = rememberThemeSources(enabled = true)

    sources.matugenSeed?.let { return it }

    return systemAccentColor()
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
    return rememberDynamicColorScheme(
        seedColor = seedColor,
        isDark = isDark,
        style = paletteStyle
    )
}
