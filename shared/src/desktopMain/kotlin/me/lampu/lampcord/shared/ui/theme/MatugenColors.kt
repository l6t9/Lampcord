package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject

/**
 * Snakes the Material You role names (as written by matugen) that map 1:1 onto
 * Material 3 [ColorScheme] slots.
 */
private val ROLE_KEYS = listOf(
    "primary", "on_primary", "primary_container", "on_primary_container", "inverse_primary",
    "secondary", "on_secondary", "secondary_container", "on_secondary_container",
    "tertiary", "on_tertiary", "tertiary_container", "on_tertiary_container",
    "error", "on_error", "error_container", "on_error_container",
    "background", "on_background",
    "surface", "on_surface", "surface_variant", "on_surface_variant", "surface_tint",
    "inverse_surface", "inverse_on_surface",
    "outline", "outline_variant", "scrim",
    "surface_dim", "surface_bright",
    "surface_container_lowest", "surface_container_low", "surface_container",
    "surface_container_high", "surface_container_highest",
)

private val MATUGEN_JSON = Json { ignoreUnknownKeys = true }

data class MatugenPalette(
    val light: Map<String, String>,
    val dark: Map<String, String>,
) {
    /** Builds the full Material 3 scheme for the requested mode from the palette roles. */
    fun schemeFor(isDark: Boolean): ColorScheme? {
        val roles = (if (isDark) dark else light).ifEmpty { if (isDark) light else dark }
        if (roles.isEmpty()) return null
        val base = if (isDark) darkColorScheme() else lightColorScheme()
        fun c(role: String, fallback: Color): Color = roles[role]?.toArgbColor() ?: fallback
        return base.copy(
            primary = c("primary", base.primary),
            onPrimary = c("on_primary", base.onPrimary),
            primaryContainer = c("primary_container", base.primaryContainer),
            onPrimaryContainer = c("on_primary_container", base.onPrimaryContainer),
            inversePrimary = c("inverse_primary", base.inversePrimary),
            secondary = c("secondary", base.secondary),
            onSecondary = c("on_secondary", base.onSecondary),
            secondaryContainer = c("secondary_container", base.secondaryContainer),
            onSecondaryContainer = c("on_secondary_container", base.onSecondaryContainer),
            tertiary = c("tertiary", base.tertiary),
            onTertiary = c("on_tertiary", base.onTertiary),
            tertiaryContainer = c("tertiary_container", base.tertiaryContainer),
            onTertiaryContainer = c("on_tertiary_container", base.onTertiaryContainer),
            error = c("error", base.error),
            onError = c("on_error", base.onError),
            errorContainer = c("error_container", base.errorContainer),
            onErrorContainer = c("on_error_container", base.onErrorContainer),
            background = c("background", base.background),
            onBackground = c("on_background", base.onBackground),
            surface = c("surface", base.surface),
            onSurface = c("on_surface", base.onSurface),
            surfaceVariant = c("surface_variant", base.surfaceVariant),
            onSurfaceVariant = c("on_surface_variant", base.onSurfaceVariant),
            surfaceTint = c("surface_tint", base.surfaceTint),
            inverseSurface = c("inverse_surface", base.inverseSurface),
            inverseOnSurface = c("inverse_on_surface", base.inverseOnSurface),
            outline = c("outline", base.outline),
            outlineVariant = c("outline_variant", base.outlineVariant),
            scrim = c("scrim", base.scrim),
            surfaceDim = c("surface_dim", base.surfaceDim),
            surfaceBright = c("surface_bright", base.surfaceBright),
            surfaceContainerLowest = c("surface_container_lowest", base.surfaceContainerLowest),
            surfaceContainerLow = c("surface_container_low", base.surfaceContainerLow),
            surfaceContainer = c("surface_container", base.surfaceContainer),
            surfaceContainerHigh = c("surface_container_high", base.surfaceContainerHigh),
            surfaceContainerHighest = c("surface_container_highest", base.surfaceContainerHighest),
        )
    }

    /** Single accent color for the requested mode, used as the seed fallback. */
    fun seedColor(isDark: Boolean): Color? {
        val roles = if (isDark) dark.ifEmpty { light } else light.ifEmpty { dark }
        return roles["primary"]?.toArgbColor()
    }
}

/**
 * Reads an exported Material You palette. Several shapes are accepted:
 *  - newer matugen: `{ "colors": { "dark": {...}, "light": {...} } }`
 *  - older matugen: `{ "dark": {...}, "light": {...} }`
 *  - flat role map: `{ "primary": "#...", "surface": "#...", ... }` (single active scheme,
 *    e.g. the `~/.local/state/quickshell/user/generated/colors.json` written by the
 *    dots' matugen `m3colors` template)
 * In all cases the role map uses snake_case Material 3 role names. Returns null when the
 * file is missing or unparseable.
 */
fun readMatugenPalette(file: File): MatugenPalette? {
    if (!file.exists()) return null
    val text = try {
        file.readText()
    } catch (e: Exception) {
        return null
    }
    val root = try {
        MATUGEN_JSON.parseToJsonElement(text).jsonObject
    } catch (e: Exception) {
        return null
    }

    // Flat role map at the top level: this setup's generated/colors.json has the active
    // scheme directly at the root (no dark/light split), apply it to both modes.
    if (isRoleMap(root)) {
        val roles = extractRoles(root)
        if (roles.isEmpty()) return null
        return MatugenPalette(light = roles, dark = roles)
    }

    val colors = root["colors"] as? JsonObject ?: root

    fun modeRoles(mode: String): Map<String, String> {
        val obj = colors[mode] as? JsonObject ?: return emptyMap()
        return extractRoles(obj)
    }

    val light = modeRoles("light")
    val dark = modeRoles("dark")
    if (light.isEmpty() && dark.isEmpty()) return null
    return MatugenPalette(light = light, dark = dark)
}

private fun isRoleMap(obj: JsonObject?): Boolean =
    obj != null &&
        (obj["primary"] != null || obj["background"] != null) &&
        (obj["surface"] != null || obj["primary_container"] != null)

private fun extractRoles(obj: JsonObject): Map<String, String> {
    val out = HashMap<String, String>()
    for (key in ROLE_KEYS) {
        val value = (obj[key] as? JsonPrimitive)?.contentOrNull ?: continue
        if (value.startsWith("#")) out[key] = value
    }
    return out
}

/** Parses "#RRGGBB" or "#AARRGGBB" into a color, or null when malformed. */
private fun String.toArgbColor(): Color? {
    val hex = trim().removePrefix("#")
    if (hex.length != 6 && hex.length != 8) return null
    val value = hex.toLongOrNull(16) ?: return null
    return if (hex.length == 6) Color(value or 0xFF000000) else Color(value)
}