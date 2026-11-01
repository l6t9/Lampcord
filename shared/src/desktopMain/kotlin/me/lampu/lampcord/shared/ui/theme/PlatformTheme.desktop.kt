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
import com.materialkolor.quantize.QuantizerCelebi
import com.materialkolor.rememberDynamicColorScheme
import com.materialkolor.score.Score
import dev.nucleusframework.systemcolor.systemAccentColor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.utils.appFontFamily
import me.lampu.lampcord.shared.utils.getPlatformName
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.File
import java.net.URI
import java.nio.file.FileSystems
import java.nio.file.StandardWatchEventKinds
import java.util.concurrent.TimeUnit
import javax.imageio.ImageIO
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

private val KIITE_JSON = Json { ignoreUnknownKeys = true }
private const val GOOGLE_BLUE = 0xFF1B6EF3.toInt()
private const val MAX_EXTRACTION_SIZE = 128

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

    val targetNames = setOf("colors.json", "color.txt", "colors-json.json")
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
                readMatugenPaletteFile(file)?.seedColor(isDark = true)?.let { return@withContext it }
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

private fun normalizeWallpaperPath(raw: String?): File? {
    if (raw.isNullOrBlank()) return null
    var value = raw.trim().removeSurrounding("'").removeSurrounding("\"")
    if (value.startsWith("file:", ignoreCase = true)) {
        value = try {
            URI(value).path
        } catch (_: Exception) {
            value.removePrefix("file://").removePrefix("file:")
        }
    }
    if (value.isBlank()) return null
    val file = File(value)
    return file.takeIf { it.isFile }
}

private fun runCommand(vararg command: String, timeoutMs: Long = 1_500L): String? {
    return try {
        val process = ProcessBuilder(*command).redirectErrorStream(true).start()
        if (!process.waitFor(timeoutMs, TimeUnit.MILLISECONDS)) {
            process.destroyForcibly()
            return null
        }
        if (process.exitValue() != 0) return null
        process.inputStream.bufferedReader().use { it.readText().trim().takeIf(String::isNotEmpty) }
    } catch (_: Exception) {
        null
    }
}

private fun readCurrentWallpaper(home: String): File? {
    val configHome = getXdgDir("XDG_CONFIG_HOME", ".config", home)
    val cacheHome = getXdgDir("XDG_CACHE_HOME", ".cache", home)

    val kiite = File(cacheHome, "kiite/active_wallpaper.json")
    if (kiite.isFile) {
        runCatching {
            val root = KIITE_JSON.parseToJsonElement(kiite.readText()).jsonObject
            normalizeWallpaperPath(root["path"]?.jsonPrimitive?.contentOrNull)?.let { return it }
        }
    }

    normalizeWallpaperPath(File(cacheHome, "wal/wal").takeIf { it.isFile }?.readText())?.let { return it }

    val gnomeDark = runCommand("gsettings", "get", "org.gnome.desktop.interface", "color-scheme")
        ?.contains("dark", ignoreCase = true) == true
    val gnomeKeys = if (gnomeDark) {
        listOf("picture-uri-dark", "picture-uri")
    } else {
        listOf("picture-uri")
    }
    for (key in gnomeKeys) {
        normalizeWallpaperPath(
            runCommand("gsettings", "get", "org.gnome.desktop.background", key)
        )?.let { return it }
    }
    normalizeWallpaperPath(
        runCommand("gsettings", "get", "org.cinnamon.desktop.background", "picture-uri")
    )?.let { return it }
    normalizeWallpaperPath(
        runCommand("gsettings", "get", "org.mate.background", "picture-filename")
    )?.let { return it }

    val plasmaConfig = File(configHome, "plasma-org.kde.plasma.desktop-appletsrc")
    if (plasmaConfig.isFile) {
        runCatching {
            Regex("""(?m)^Image=file:(.+)$""")
                .find(plasmaConfig.readText())
                ?.groupValues
                ?.getOrNull(1)
                ?.let(::normalizeWallpaperPath)
                ?.let { return it }
        }
    }
    normalizeWallpaperPath(
        runCommand("kreadconfig6", "--file", "plasma-org.kde.plasma.desktop-appletsrc",
            "--group", "Containments", "--group", "1", "--group", "Wallpaper",
            "--group", "org.kde.image", "--group", "General", "--key", "Image")
    )?.let { return it }
    normalizeWallpaperPath(
        runCommand("kreadconfig5", "--file", "plasma-org.kde.plasma.desktop-appletsrc",
            "--group", "Containments", "--group", "1", "--group", "Wallpaper",
            "--group", "org.kde.image", "--group", "General", "--key", "Image")
    )?.let { return it }

    val xfceConfig = File(configHome, "xfce4/xfconf/xfce-perchannel-xml/xfce4-desktop.xml")
    if (xfceConfig.isFile) {
        runCatching {
            Regex("""<property name="(?:last-image|image-path)"[^>]*value="([^"]+)"""")
                .find(xfceConfig.readText())
                ?.groupValues
                ?.getOrNull(1)
                ?.let(::normalizeWallpaperPath)
                ?.let { return it }
        }
    }
    normalizeWallpaperPath(runCommand("xfconf-query", "-c", "xfce4-desktop", "-p", "/backdrop/screen0/monitor0/workspace0/last-image"))
        ?.let { return it }

    val hyprpaperConfig = File(configHome, "hypr/hyprpaper.conf")
    if (hyprpaperConfig.isFile) {
        runCatching {
            hyprpaperConfig.readLines()
                .asReversed()
                .firstNotNullOfOrNull { line ->
                    val value = line.substringAfter('=', "").trim()
                    if (value.isBlank()) null else normalizeWallpaperPath(value.substringAfterLast(',').trim())
                }
                ?.let { return it }
        }
    }
    normalizeWallpaperPath(runCommand("hyprctl", "hyprpaper", "listloaded")?.lineSequence()?.firstOrNull())
        ?.let { return it }
    runCommand("swww", "query")?.let { output ->
        Regex("""(?m)^.*?image:\s*(.+)$""")
            .find(output)
            ?.groupValues
            ?.getOrNull(1)
            ?.let(::normalizeWallpaperPath)
            ?.let { return it }
    }

    val swayConfig = File(configHome, "sway/config")
    if (swayConfig.isFile) {
        runCatching {
            Regex("""(?m)^\s*output\s+\S+\s+bg\s+(\S+)""")
                .find(swayConfig.readText())
                ?.groupValues
                ?.getOrNull(1)
                ?.let(::normalizeWallpaperPath)
                ?.let { return it }
        }
    }

    val nitrogenConfig = File(configHome, "nitrogen/bg-saved.cfg")
    if (nitrogenConfig.isFile) {
        runCatching {
            nitrogenConfig.readLines()
                .firstOrNull { it.startsWith("file=") }
                ?.substringAfter('=')
                ?.let(::normalizeWallpaperPath)
                ?.let { return it }
        }
    }

    return null
}

private fun extractWallpaperSeed(wallpaper: File): Color? {
    val source = try {
        ImageIO.read(wallpaper)
    } catch (_: Exception) {
        null
    } ?: return null
    if (source.width <= 0 || source.height <= 0) return null

    val scale = min(1f, MAX_EXTRACTION_SIZE.toFloat() / max(source.width, source.height))
    val width = max(1, (source.width * scale).roundToInt())
    val height = max(1, (source.height * scale).roundToInt())
    val scaled = BufferedImage(width, height, BufferedImage.TYPE_INT_ARGB)
    val graphics = scaled.createGraphics()
    try {
        graphics.setRenderingHint(
            RenderingHints.KEY_INTERPOLATION,
            RenderingHints.VALUE_INTERPOLATION_BILINEAR
        )
        graphics.drawImage(source, 0, 0, width, height, null)
    } finally {
        graphics.dispose()
    }

    val pixels = IntArray(width * height)
    scaled.getRGB(0, 0, width, height, pixels, 0, width)
    val opaque = pixels.filter { (it ushr 24) and 0xFF >= 128 }.toIntArray()
    if (opaque.isEmpty()) return null

    val quantized = QuantizerCelebi.quantize(opaque, 128)
    var ranked = Score.score(
        colorsToPopulation = quantized,
        desired = 4,
        fallbackColorArgb = GOOGLE_BLUE,
        filter = true
    )
    if (ranked.size == 1 && ranked[0] == GOOGLE_BLUE) {
        val unfiltered = Score.score(
            colorsToPopulation = quantized,
            desired = 4,
            fallbackColorArgb = GOOGLE_BLUE,
            filter = false
        )
        if (unfiltered.isNotEmpty() && (unfiltered.size > 1 || unfiltered[0] != GOOGLE_BLUE)) {
            ranked = unfiltered
        } else {
            quantized.maxByOrNull { it.value }?.let { ranked = listOf(it.key) }
        }
    }

    return Color(ranked.firstOrNull() ?: GOOGLE_BLUE)
}

private class ThemeSources(
    val wallpaperSeed: Color? = null,
    val matugenSeed: Color? = null,
    val wallpaperDir: File? = null,
    val wallpaperName: String? = null
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
    val kiiteCacheDir = remember(home) {
        File(getXdgDir("XDG_CACHE_HOME", ".cache", home), "kiite")
    }

    var sources by remember { mutableStateOf(ThemeSources()) }

    LaunchedEffect(enabled, matugenFiles, kiiteCacheDir) {
        if (!enabled) {
            sources = ThemeSources()
            return@LaunchedEffect
        }

        suspend fun reload() {
            val wallpaper = withContext(Dispatchers.IO) { readCurrentWallpaper(home) }
            val wallpaperSeed = withContext(Dispatchers.IO) {
                wallpaper?.let(::extractWallpaperSeed)
            }
            sources = ThemeSources(
                wallpaperSeed = wallpaperSeed,
                matugenSeed = if (wallpaperSeed == null) loadMatugenSeed(matugenFiles) else null,
                wallpaperDir = wallpaper?.parentFile,
                wallpaperName = wallpaper?.name
            )
        }

        reload()

        val watcher = watchThemeFiles(
            dirs = matugenFiles.mapNotNull { it.parentFile } + kiiteCacheDir +
                listOfNotNull(sources.wallpaperDir),
            watchedNames = setOf(
                "colors.json", "color.txt", "colors-json.json",
                "active_wallpaper.json"
            ) + listOfNotNull(sources.wallpaperName)
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

    sources.wallpaperSeed?.let { return it }
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
