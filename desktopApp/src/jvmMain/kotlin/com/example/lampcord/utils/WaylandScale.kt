package com.example.lampcord.utils

import com.sun.jna.Library
import com.sun.jna.Native
import java.io.File

object WaylandScale {
    private var applied = false

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

        // Fix for oversized mouse cursors on Linux/Wayland HiDPI
        // Force Java's underlying windowing toolkits to evaluate layout scales at 1:1
        // which prevents the cursor from being scaled twice (once by Java, once by the compositor)
        System.setProperty("sun.java2d.uiScale", "1")

        // Ensure GDK doesn't multiply the cursor size
        if (System.getenv("XCURSOR_SIZE") == null) {
            System.setProperty("GDK_SCALE", "1")
        }

        // WebView fix for black screen / GBM buffer errors on Linux (WebKitGTK)
        try {
            val libc = Native.load("c", LibC::class.java) as LibC
            libc.setenv("WEBKIT_DISABLE_DMABUF_RENDERER", "1", 1)
            libc.setenv("WEBKIT_DISABLE_COMPOSITING_MODE", "1", 1)
        } catch (_: Throwable) {
        }

        // Try to fix cursor theme mismatch for XWayland
        applyCursorTheme()

        val scale = detectScale()
        if (scale in 1.0f..6.0f) {
            System.setProperty("skiko.uiScale", scale.toString())
            applied = true
        }
    }

    fun isWayland(): Boolean {
        val sessionType = System.getenv("XDG_SESSION_TYPE")
        val waylandDisplay = System.getenv("WAYLAND_DISPLAY")
        return sessionType == "wayland" || waylandDisplay != null
    }

    private fun applyCursorTheme() {
        try {
            var theme = runCommand("gsettings", "get", "org.gnome.desktop.interface", "cursor-theme")
                ?.trim()?.removePrefix("'")?.removeSuffix("'")

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
                System.setProperty("XCURSOR_THEME", theme)
            }
            if (size != null && size > 0 && System.getenv("XCURSOR_SIZE") == null) {
                libc.setenv("XCURSOR_SIZE", size.toString(), 1)
                System.setProperty("XCURSOR_SIZE", size.toString())
            }
        } catch (_: Throwable) {}
    }

    private fun detectScale(): Float {
        // Environment variables first
        val envVars = listOf("GDK_SCALE", "XDG_SCALE", "QT_SCALE_FACTOR")
        for (env in envVars) {
            System.getenv(env)?.toFloatOrNull()?.let { if (it > 0) return it }
        }

        // Compositor specific detection
        detectScaleFromKScreen()?.let { return it }
        detectScaleFromWlrRandr()?.let { return it }
        detectScaleFromSway()?.let { return it }
        detectScaleFromHyprctl()?.let { return it }
        detectScaleFromGnome()?.let { return it }

        return 1.0f
    }

    private fun detectScaleFromKScreen(): Float? = runCommand("kscreen-doctor")?.let {
        Regex("""Scale:\s*(\d+)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.minOrNull()
    }

    private fun detectScaleFromWlrRandr(): Float? = runCommand("wlr-randr")?.let {
        Regex("""scale:\s*(\d+\.?\d*)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.minOrNull()
    }

    private fun detectScaleFromSway(): Float? = runCommand("swaymsg", "-t", "get_outputs")?.let {
        Regex(""""scale"\s*:\s*(\d+\.?\d*)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.minOrNull()
    }

    private fun detectScaleFromHyprctl(): Float? = runCommand("hyprctl", "monitors", "-j")?.let {
        Regex(""""scale"\s*:\s*([\d.]+)""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.minOrNull()
    }

    private fun detectScaleFromGnome(): Float? = runCommand(
        "gdbus", "call", "--session", "--dest", "org.gnome.Mutter.DisplayConfig",
        "--object-path", "/org/gnome/Mutter/DisplayConfig",
        "--method", "org.gnome.Mutter.DisplayConfig.GetCurrentState"
    )?.let {
        Regex("""'scale',\s*<(\d+)>""").findAll(it).mapNotNull { m -> m.groupValues[1].toFloatOrNull() }.maxOrNull()
    }

    private fun runCommand(vararg cmd: String): String? = try {
        val process = ProcessBuilder(*cmd).redirectErrorStream(true).start()
        val output = process.inputStream.bufferedReader().use { it.readText() }
        process.waitFor()
        output
    } catch (_: Exception) { null }
}
