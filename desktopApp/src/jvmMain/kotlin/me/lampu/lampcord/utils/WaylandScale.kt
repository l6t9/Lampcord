package me.lampu.lampcord.utils

import com.sun.jna.Library
import com.sun.jna.Native
import java.io.File
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

object WaylandScale {
    private var applied = false
    private val monitorCache = AtomicReference<List<Monitor>>(emptyList())
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
        if (!isWayland()) return

        val forceScale = System.getProperty("lampcord.forceScale")?.toFloatOrNull()
            ?: System.getenv("LAMPCORD_FORCE_SCALE")?.toFloatOrNull()

        // Wakefield (Native Wayland) support - Experimental
        val useWakefield = System.getProperty("lampcord.useWakefield") == "true" || System.getenv("LAMPCORD_USE_WAKEFIELD") == "1"
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
        } catch (_: Throwable) {}

        applyCursorTheme()

        // Sync refresh once before starting the async watcher to ensure detectScale() has cache
        refreshMonitorCache()

        // Start async monitor watcher for dynamic scaling
        startMonitorWatcher()

        val scale = forceScale ?: detectScale()
        if (scale in 0.1f..10.0f) {
            System.setProperty("skiko.uiScale", scale.toString())
            applied = true
        }
    }

    fun isWayland(): Boolean {
        val sessionType = System.getenv("XDG_SESSION_TYPE")
        val waylandDisplay = System.getenv("WAYLAND_DISPLAY")
        return sessionType == "wayland" || waylandDisplay != null
    }

    private data class Monitor(val x: Int, val y: Int, val w: Int, val h: Int, val scale: Float)

    /**
     * Instantly returns the scale for the given coordinates from the background cache.
     */
    fun getWindowScale(windowX: Int, windowY: Int): Float {
        val forceScale = System.getProperty("lampcord.forceScale")?.toFloatOrNull()
            ?: System.getenv("LAMPCORD_FORCE_SCALE")?.toFloatOrNull()
        if (forceScale != null) return forceScale

        if (!isWayland()) return 1.0f

        val monitors = monitorCache.get()
        monitors.find { m -> 
            windowX >= m.x && windowX < m.x + m.w && windowY >= m.y && windowY < m.y + m.h 
        }?.let { return it.scale }

        // Fallback to primary detection if not in any known monitor bounds
        return detectScale()
    }

    private fun startMonitorWatcher() {
        if (watcherThread != null) return
        
        watcherThread = Thread({
            while (!Thread.interrupted()) {
                try {
                    refreshMonitorCache()
                    // Sleep for a while. We can afford 2 seconds here because 
                    // the Provider also triggers checks on window move.
                    Thread.sleep(2000) 
                } catch (e: InterruptedException) {
                    break
                } catch (_: Exception) {}
            }
        }, "WaylandMonitorWatcher").apply {
            isDaemon = true
            start()
        }
    }

    fun refreshMonitorCache() {
        val desktop = System.getenv("XDG_CURRENT_DESKTOP")?.lowercase() ?: ""
        val json = if (desktop.contains("hyprland")) {
            runCommand("hyprctl", "monitors", "-j")
        } else if (desktop.contains("sway")) {
            runCommand("swaymsg", "-t", "get_outputs")
        } else null

        if (json != null) {
            try {
                val monitors = mutableListOf<Monitor>()
                val blocks = json.split("},{", "}, {", "[{", "}]")
                for (block in blocks) {
                    if (block.isBlank()) continue
                    val x = Regex(""""x"\s*:\s*(-?\d+)""").find(block)?.groupValues?.get(1)?.toIntOrNull() ?: 0
                    val y = Regex(""""y"\s*:\s*(-?\d+)""").find(block)?.groupValues?.get(1)?.toIntOrNull() ?: 0
                    val w = Regex(""""width"\s*:\s*(\d+)""").find(block)?.groupValues?.get(1)?.toIntOrNull() ?: 0
                    val h = Regex(""""height"\s*:\s*(\d+)""").find(block)?.groupValues?.get(1)?.toIntOrNull() ?: 0
                    val scale = Regex(""""scale"\s*:\s*([\d.]+)""").find(block)?.groupValues?.get(1)?.toFloatOrNull() ?: 1.0f
                    monitors.add(Monitor(x, y, w, h, scale))
                }
                if (monitors.isNotEmpty()) {
                    monitorCache.set(monitors)
                }
            } catch (_: Exception) {}
        }
    }

    private fun applyCursorTheme() {
        try {
            var theme = runCommand("gsettings", "get", "org.gnome.desktop.interface", "cursor-theme")?.trim()?.removePrefix("'")?.removeSuffix("'")
            if (theme.isNullOrBlank()) {
                val kcminputrc = File(System.getProperty("user.home"), ".config/kcminputrc")
                if (kcminputrc.exists()) {
                    theme = Regex("""cursorTheme\s*=\s*(.*)""").find(kcminputrc.readText())?.groupValues?.get(1)?.trim()
                }
            }
            var size = runCommand("gsettings", "get", "org.gnome.desktop.interface", "cursor-size")?.trim()?.toIntOrNull()
            if (size == null) {
                val kcminputrc = File(System.getProperty("user.home"), ".config/kcminputrc")
                if (kcminputrc.exists()) {
                    size = Regex("""cursorSize\s*=\s*(.*)""").find(kcminputrc.readText())?.groupValues?.get(1)?.trim()?.toIntOrNull()
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
        } catch (_: Throwable) {}
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

    private fun detectScaleFromGsettings(): Float? = try {
        val factor = runCommand("gsettings", "get", "org.gnome.desktop.interface", "scaling-factor")
            ?.trim()?.removePrefix("uint32 ")?.toFloatOrNull()
        if (factor != null && factor > 1f) factor
        else runCommand("gsettings", "get", "org.gnome.desktop.interface", "text-scaling-factor")
            ?.trim()?.toFloatOrNull()
    } catch (_: Exception) { null }

    private fun detectScaleFromKScreen(): Float? = runCommand("kscreen-doctor", "-o")?.let {
        Regex("""Scale:\s*([\d.]+)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.maxOrNull()
    }

    private fun detectScaleFromWlrRandr(): Float? = runCommand("wlr-randr")?.let {
        Regex("""scale:\s*([\d.]+)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.maxOrNull()
    }

    private fun detectScaleFromSway(): Float? = runCommand("swaymsg", "-t", "get_outputs")?.let {
        Regex(""""scale"\s*:\s*([\d.]+)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.maxOrNull()
    }

    private fun detectScaleFromGnome(): Float? = runCommand(
        "gdbus", "call", "--session", "--dest", "org.gnome.Mutter.DisplayConfig",
        "--object-path", "/org/org.gnome.Mutter.DisplayConfig",
        "--method", "org.gnome.Mutter.DisplayConfig.GetCurrentState"
    )?.let {
        Regex("""'scale',\s*<([\d.]+)>""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.maxOrNull()
    }

    private fun runCommand(vararg cmd: String): String? = try {
        val process = ProcessBuilder(*cmd).redirectErrorStream(true).start()
        
        val output = StringBuilder()
        val readerThread = Thread {
            try {
                process.inputStream.bufferedReader().use { reader ->
                    var line: String?
                    while (reader.readLine().also { line = it } != null) {
                        output.append(line).append("\n")
                    }
                }
            } catch (_: Exception) {}
        }
        readerThread.start()

        if (process.waitFor(2, TimeUnit.SECONDS)) {
            readerThread.join(500)
            output.toString()
        } else {
            process.destroyForcibly()
            null
        }
    } catch (_: Exception) { null }
}
