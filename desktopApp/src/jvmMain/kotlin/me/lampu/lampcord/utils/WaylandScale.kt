package me.lampu.lampcord.utils

import com.sun.jna.Library
import com.sun.jna.Native
import java.io.File
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import me.lampu.lampcord.shared.settings.Settings

object WaylandScale {
    private const val MAX_COMMAND_OUTPUT_CHARS = 64 * 1_024
    private val TRUSTED_EXECUTABLE_DIRECTORIES = listOf("/usr/bin", "/bin", "/usr/local/bin")
    private val LOG_FILE: File = File(System.getProperty("user.home"), ".cache/lampcord/wayland_scale.log")
    private var applied = false
    private val monitorCache = AtomicReference<List<Monitor>>(emptyList())
    private val fallbackScale = AtomicReference(1.0f)
    private var watcherThread: Thread? = null

    interface LibC : Library {
        fun setenv(
            name: String,
            value: String,
            overwrite: Int,
        ): Int
    }

    fun detectAndApply() {
        if (applied) return
        if (!isWayland() || Settings.shared.disableWaylandScaling) return

        val forceScale =
            System.getProperty("lampcord.forceScale")?.toFloatOrNull()
                ?: System.getenv("LAMPCORD_FORCE_SCALE")?.toFloatOrNull()

        // Wakefield (Native Wayland) support - Experimental
        val useWakefield =
            System.getProperty("lampcord.useWakefield") == "true" || System.getenv("LAMPCORD_USE_WAKEFIELD") == "1"
        if (useWakefield) {
            System.setProperty("awt.toolkit.name", "WLToolkit")
            System.setProperty("jdk.gtk.version", "3")
            System.setProperty("sun.java2d.uiScale.enabled", "true")
            if (forceScale != null) System.setProperty("sun.java2d.uiScale", forceScale.toString())
            applied = true
            return
        }

        // Standard Wayland/XWayland fix
        if (forceScale == null) {
            System.setProperty("sun.java2d.uiScale", "1")
        } else {
            System.setProperty("sun.java2d.uiScale", forceScale.toString())
        }

        if (System.getenv("XCURSOR_SIZE") == null) {
            System.setProperty("GDK_SCALE", "1")
        }

        try {
            val libc = Native.load("c", LibC::class.java) as LibC
            libc.setenv("WEBKIT_DISABLE_DMABUF_RENDERER", "1", 1)
            libc.setenv("WEBKIT_DISABLE_COMPOSITING_MODE", "1", 1)
        } catch (_: Throwable) {
        }

        applyCursorTheme()

        val scale = forceScale ?: detectScale()
        if (scale in 0.1f..10.0f) {
            fallbackScale.set(scale)
            System.setProperty("skiko.uiScale", scale.toString())
            applied = true
        }
        log("Applied Wayland scale: $scale")

        // Keep monitor discovery off the AWT event thread.
        startMonitorWatcher()
    }

    fun isWayland(): Boolean {
        val sessionType = System.getenv("XDG_SESSION_TYPE")
        val waylandDisplay = System.getenv("WAYLAND_DISPLAY")
        return sessionType == "wayland" || waylandDisplay != null
    }

    fun ensureDisplay(): Boolean {
        val currentDisplay = System.getenv("DISPLAY")
        if (!currentDisplay.isNullOrBlank()) {
            System.setProperty("DISPLAY", currentDisplay)
            return true
        }

        val detectedDisplay = findDisplay() ?: ":0"
        try {
            val libc = Native.load("c", LibC::class.java) as LibC
            libc.setenv("DISPLAY", detectedDisplay, 1)
        } catch (e: Throwable) {
            log("Failed to set native DISPLAY: ${e.message}")
        }
        System.setProperty("DISPLAY", detectedDisplay)
        updateJavaEnv("DISPLAY", detectedDisplay)
        log("Ensured DISPLAY=$detectedDisplay")
        return true
    }

    private fun findDisplay(): String? {
        val x11Dir = File("/tmp/.X11-unix")
        if (x11Dir.isDirectory) {
            val sockets =
                x11Dir
                    .listFiles()
                    ?.filter { it.name.startsWith("X") }
                    ?.mapNotNull { it.name.removePrefix("X").toIntOrNull() }
                    ?.sorted()
            if (!sockets.isNullOrEmpty()) {
                return ":${sockets.first()}"
            }
        }

        val xdgRuntime = System.getenv("XDG_RUNTIME_DIR")
        if (!xdgRuntime.isNullOrBlank()) {
            val xdgX11Dir = File(xdgRuntime, "X11")
            if (xdgX11Dir.isDirectory) {
                val sockets =
                    xdgX11Dir
                        .listFiles()
                        ?.filter { it.name.startsWith("X") }
                        ?.mapNotNull { it.name.removePrefix("X").toIntOrNull() }
                        ?.sorted()
                if (!sockets.isNullOrEmpty()) {
                    return ":${sockets.first()}"
                }
            }
            val x11DisplayFile = File(xdgRuntime, "x11-display")
            if (x11DisplayFile.isFile) {
                val text = runCatching { x11DisplayFile.readText().trim() }.getOrNull()
                if (!text.isNullOrBlank()) {
                    return text
                }
            }
        }

        return ":0"
    }

    private fun updateJavaEnv(
        name: String,
        value: String,
    ) {
        try {
            val processEnvClass = Class.forName("java.lang.ProcessEnvironment")
            val theEnvironmentField = processEnvClass.getDeclaredField("theEnvironment")
            theEnvironmentField.isAccessible = true
            @Suppress("UNCHECKED_CAST")
            val env = theEnvironmentField.get(null) as MutableMap<String, String>
            env[name] = value

            val theCaseInsensitiveEnvironmentField =
                runCatching { processEnvClass.getDeclaredField("theCaseInsensitiveEnvironment") }.getOrNull()
            if (theCaseInsensitiveEnvironmentField != null) {
                theCaseInsensitiveEnvironmentField.isAccessible = true
                @Suppress("UNCHECKED_CAST")
                val cienv = theCaseInsensitiveEnvironmentField.get(null) as MutableMap<String, String>
                cienv[name] = value
            }
        } catch (_: Throwable) {
        }
    }

    internal data class Monitor(
        val x: Int,
        val y: Int,
        val w: Int,
        val h: Int,
        val scale: Float,
    )

    fun getWindowScale(
        windowX: Int,
        windowY: Int,
    ): Float {
        val forceScale =
            System.getProperty("lampcord.forceScale")?.toFloatOrNull()
                ?: System.getenv("LAMPCORD_FORCE_SCALE")?.toFloatOrNull()
        if (forceScale != null) return forceScale

        if (!isWayland()) return 1.0f

        val monitors = monitorCache.get()
        monitors
            .find { m ->
                windowX >= m.x && windowX < m.x + m.w && windowY >= m.y && windowY < m.y + m.h
            }?.let { return it.scale }

        return fallbackScale.get()
    }

    private fun startMonitorWatcher() {
        if (watcherThread != null) return

        watcherThread =
            Thread({
                while (!Thread.interrupted()) {
                    try {
                        refreshMonitorCache()
                        Thread.sleep(2000)
                    } catch (e: InterruptedException) {
                        break
                    } catch (_: Exception) {
                    }
                }
            }, "WaylandMonitorWatcher").apply {
                isDaemon = true
                start()
            }
    }

    fun refreshMonitorCache() {
        val desktop = System.getenv("XDG_CURRENT_DESKTOP")?.lowercase() ?: ""
        if (desktop.contains("kde") || desktop.contains("plasma")) {
            updateMonitorCache(parseKScreenMonitors(runCommand("kscreen-doctor", "-o").orEmpty()))
            return
        }
        val json =
            if (desktop.contains("hyprland")) {
                runCommand("hyprctl", "monitors", "-j")
            } else if (desktop.contains("sway")) {
                runCommand("swaymsg", "-t", "get_outputs")
            } else {
                null
            }

        if (json != null) {
            try {
                val monitors = mutableListOf<Monitor>()
                val blocks = json.split("},{", "}, {", "[{", "}]")
                for (block in blocks) {
                    if (block.isBlank()) continue
                    val x =
                        Regex(""""x"\s*:\s*(-?\d+)""")
                            .find(block)
                            ?.groupValues
                            ?.get(1)
                            ?.toIntOrNull() ?: 0
                    val y =
                        Regex(""""y"\s*:\s*(-?\d+)""")
                            .find(block)
                            ?.groupValues
                            ?.get(1)
                            ?.toIntOrNull() ?: 0
                    val w =
                        Regex(""""width"\s*:\s*(\d+)""")
                            .find(block)
                            ?.groupValues
                            ?.get(1)
                            ?.toIntOrNull() ?: 0
                    val h =
                        Regex(""""height"\s*:\s*(\d+)""")
                            .find(block)
                            ?.groupValues
                            ?.get(1)
                            ?.toIntOrNull() ?: 0
                    val scale =
                        Regex(""""scale"\s*:\s*([\d.]+)""")
                            .find(block)
                            ?.groupValues
                            ?.get(1)
                            ?.toFloatOrNull() ?: 1.0f
                    monitors.add(Monitor(x, y, w, h, scale))
                }
                if (monitors.isNotEmpty()) {
                    updateMonitorCache(monitors)
                }
            } catch (_: Exception) {
            }
        }
    }

    internal fun parseKScreenMonitors(output: String): List<Monitor> =
        Regex("""Output:.*?(?=Output:|\z)""", RegexOption.DOT_MATCHES_ALL)
            .findAll(output)
            .mapNotNull { match ->
                val geometry =
                    Regex("""Geometry:\s*(-?\d+),(-?\d+)\s+(\d+)x(\d+)""")
                        .find(match.value)
                        ?.groupValues
                        ?: return@mapNotNull null
                val scale =
                    Regex("""Scale:\s*([\d.]+)""")
                        .find(match.value)
                        ?.groupValues
                        ?.get(1)
                        ?.toFloatOrNull()
                        ?: return@mapNotNull null
                Monitor(
                    x = geometry[1].toInt(),
                    y = geometry[2].toInt(),
                    w = geometry[3].toInt(),
                    h = geometry[4].toInt(),
                    scale = scale,
                )
            }.toList()

    private fun updateMonitorCache(monitors: List<Monitor>) {
        if (monitors.isEmpty()) return
        monitorCache.set(monitors)
        fallbackScale.set(monitors.maxOf { it.scale })
    }

    private fun applyCursorTheme() {
        try {
            var theme =
                runCommand("gsettings", "get", "org.gnome.desktop.interface", "cursor-theme")
                    ?.trim()
                    ?.removePrefix("'")
                    ?.removeSuffix("'")
            if (theme.isNullOrBlank()) {
                val kcminputrc = File(System.getProperty("user.home"), ".config/kcminputrc")
                if (kcminputrc.exists()) {
                    theme =
                        Regex("""cursorTheme\s*=\s*(.*)""")
                            .find(kcminputrc.readText())
                            ?.groupValues
                            ?.get(1)
                            ?.trim()
                }
            }
            var size =
                runCommand("gsettings", "get", "org.gnome.desktop.interface", "cursor-size")
                    ?.trim()
                    ?.toIntOrNull()
            if (size == null) {
                val kcminputrc = File(System.getProperty("user.home"), ".config/kcminputrc")
                if (kcminputrc.exists()) {
                    size =
                        Regex("""cursorSize\s*=\s*(.*)""")
                            .find(kcminputrc.readText())
                            ?.groupValues
                            ?.get(1)
                            ?.trim()
                            ?.toIntOrNull()
                }
            }
            val libc = Native.load("c", LibC::class.java) as LibC
            if (!theme.isNullOrBlank() && System.getenv("XCURSOR_THEME") == null) {
                libc.setenv("XCURSOR_THEME", theme, 1)
                libc.setenv("GDK_CURSOR_THEME", theme, 1)
            }
            if (size != null && size > 0 && System.getenv("XCURSOR_SIZE") == null) {
                libc.setenv("XCURSOR_SIZE", size.toString(), 1)
            }
        } catch (_: Throwable) {
        }
    }

    private fun detectScale(): Float {
        val envVars = listOf("GDK_SCALE", "GDK_DPI_SCALE", "XDG_SCALE", "QT_SCALE_FACTOR")
        for (env in envVars) {
            System.getenv(env)?.toFloatOrNull()?.let { if (it > 0) return it }
        }

        val desktop = System.getenv("XDG_CURRENT_DESKTOP")?.lowercase() ?: ""

        if (desktop.contains("hyprland")) {
            monitorCache.get().maxOfOrNull { it.scale }?.let { return it }
        } else if (desktop.contains("kde") || desktop.contains("plasma")) {
            detectScaleFromKScreen()?.let { return it }
        } else if (desktop.contains("gnome")) {
            detectScaleFromGnome()?.let { return it }
        } else if (desktop.contains("sway")) {
            detectScaleFromSway()?.let { return it }
        }

        detectScaleFromGsettings()?.let { return it }
        detectScaleFromWlrRandr()?.let { return it }

        return 1.0f
    }

    private fun detectScaleFromGsettings(): Float? =
        try {
            val factor =
                runCommand("gsettings", "get", "org.gnome.desktop.interface", "scaling-factor")
                    ?.trim()
                    ?.removePrefix("uint32 ")
                    ?.toFloatOrNull()
            if (factor != null && factor > 1f) {
                factor
            } else {
                runCommand("gsettings", "get", "org.gnome.desktop.interface", "text-scaling-factor")
                    ?.trim()
                    ?.toFloatOrNull()
            }
        } catch (_: Exception) {
            null
        }

    private fun detectScaleFromKScreen(): Float? =
        runCommand("kscreen-doctor", "-o")?.let {
            Regex("""Scale:\s*([\d.]+)""")
                .findAll(it)
                .mapNotNull { m -> m.groupValues[1].toFloatOrNull() }
                .maxOrNull()
        }

    private fun detectScaleFromWlrRandr(): Float? =
        runCommand("wlr-randr")?.let {
            Regex("""scale:\s*([\d.]+)""")
                .findAll(it)
                .mapNotNull { m -> m.groupValues[1].toFloatOrNull() }
                .maxOrNull()
        }

    private fun detectScaleFromSway(): Float? =
        runCommand("swaymsg", "-t", "get_outputs")?.let {
            Regex(""""scale"\s*:\s*([\d.]+)""")
                .findAll(it)
                .mapNotNull { m -> m.groupValues[1].toFloatOrNull() }
                .maxOrNull()
        }

    private fun detectScaleFromGnome(): Float? =
        runCommand(
            "gdbus",
            "call",
            "--session",
            "--dest",
            "org.gnome.Mutter.DisplayConfig",
            "--object-path",
            "/org/org.gnome.Mutter.DisplayConfig",
            "--method",
            "org.gnome.Mutter.DisplayConfig.GetCurrentState",
        )?.let {
            Regex("""'scale',\s*<([\d.]+)>""")
                .findAll(it)
                .mapNotNull { m -> m.groupValues[1].toFloatOrNull() }
                .maxOrNull()
        }

    private fun runCommand(
        command: String,
        vararg arguments: String,
    ): String? {
        val executable =
            TRUSTED_EXECUTABLE_DIRECTORIES
                .asSequence()
                .map { File(it, command) }
                .firstOrNull { it.isFile && it.canExecute() } ?: return null
        return try {
            val process = ProcessBuilder(executable.absolutePath, *arguments).redirectErrorStream(true).start()
            val output = StringBuilder()
            val readerThread =
                Thread {
                    runCatching {
                        process.inputStream.bufferedReader().use { reader ->
                            val buffer = CharArray(1_024)
                            while (true) {
                                val count = reader.read(buffer)
                                if (count < 0) break
                                val remaining = MAX_COMMAND_OUTPUT_CHARS - output.length
                                if (remaining > 0) output.append(buffer, 0, minOf(count, remaining))
                            }
                        }
                    }
                }.apply { start() }

            if (!process.waitFor(2, TimeUnit.SECONDS)) {
                process.destroyForcibly()
                process.inputStream.close()
                readerThread.join(500)
                null
            } else {
                readerThread.join()
                output.toString()
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun log(message: String) {
        try {
            LOG_FILE.parentFile?.mkdirs()
            LOG_FILE.appendText("[${System.currentTimeMillis()}] $message\n")
        } catch (_: Exception) {
        }
    }
}
