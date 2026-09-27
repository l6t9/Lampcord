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

private val ROLE_NAME_MAPPINGS = mapOf(
    "primary" to "primary",
    "on_primary" to "on_primary", "onPrimary" to "on_primary",
    "primary_container" to "primary_container", "primaryContainer" to "primary_container",
    "on_primary_container" to "on_primary_container", "onPrimaryContainer" to "on_primary_container",
    "inverse_primary" to "inverse_primary", "inversePrimary" to "inverse_primary",
    "secondary" to "secondary",
    "on_secondary" to "on_secondary", "onSecondary" to "on_secondary",
    "secondary_container" to "secondary_container", "secondaryContainer" to "secondary_container",
    "on_secondary_container" to "on_secondary_container", "onSecondaryContainer" to "on_secondary_container",
    "tertiary" to "tertiary",
    "on_tertiary" to "on_tertiary", "onTertiary" to "on_tertiary",
    "tertiary_container" to "tertiary_container", "tertiaryContainer" to "tertiary_container",
    "on_tertiary_container" to "on_tertiary_container", "onTertiaryContainer" to "on_tertiary_container",
    "error" to "error",
    "on_error" to "on_error", "onError" to "on_error",
    "error_container" to "error_container", "errorContainer" to "error_container",
    "on_error_container" to "on_error_container", "onErrorContainer" to "on_error_container",
    "background" to "background",
    "on_background" to "on_background", "onBackground" to "on_background",
    "surface" to "surface",
    "on_surface" to "on_surface", "onSurface" to "on_surface",
    "surface_variant" to "surface_variant", "surfaceVariant" to "surface_variant",
    "on_surface_variant" to "on_surface_variant", "onSurfaceVariant" to "on_surface_variant",
    "surface_tint" to "surface_tint", "surfaceTint" to "surface_tint",
    "inverse_surface" to "inverse_surface", "inverseSurface" to "inverse_surface",
    "inverse_on_surface" to "inverse_on_surface", "inverseOnSurface" to "inverse_on_surface",
    "outline" to "outline",
    "outline_variant" to "outline_variant", "outlineVariant" to "outline_variant",
    "scrim" to "scrim",
    "surface_dim" to "surface_dim", "surfaceDim" to "surface_dim",
    "surface_bright" to "surface_bright", "surfaceBright" to "surface_bright",
    "surface_container_lowest" to "surface_container_lowest", "surfaceContainerLowest" to "surface_container_lowest",
    "surface_container_low" to "surface_container_low", "surfaceContainerLow" to "surface_container_low",
    "surface_container" to "surface_container", "surfaceContainer" to "surface_container",
    "surface_container_high" to "surface_container_high", "surfaceContainerHigh" to "surface_container_high",
    "surface_container_highest" to "surface_container_highest", "surfaceContainerHighest" to "surface_container_highest",
)

private val MATUGEN_JSON = Json { ignoreUnknownKeys = true }

data class MatugenPalette(
    val light: Map<String, String>,
    val dark: Map<String, String>,
) {
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

    fun seedColor(isDark: Boolean): Color? {
        val roles = if (isDark) dark.ifEmpty { light } else light.ifEmpty { dark }
        return roles["primary"]?.toArgbColor()
    }
}

fun readMatugenPalette(file: File): MatugenPalette? {
    if (!file.exists()) return null
    val text = try {
        file.readText()
    } catch (_: Exception) {
        return null
    }
    val root = try {
        MATUGEN_JSON.parseToJsonElement(text).jsonObject
    } catch (_: Exception) {
        return null
    }

    val colorsObj = (root["colors"] as? JsonObject) ?: root
    val lightObj = colorsObj["light"] as? JsonObject
    val darkObj = colorsObj["dark"] as? JsonObject

    val lightRoles = if (lightObj != null) extractRoles(lightObj) else emptyMap()
    val darkRoles = if (darkObj != null) extractRoles(darkObj) else emptyMap()

    if (lightRoles.isNotEmpty() || darkRoles.isNotEmpty()) {
        return MatugenPalette(
            light = lightRoles.ifEmpty { darkRoles },
            dark = darkRoles.ifEmpty { lightRoles }
        )
    }

    val flatRoles = extractRoles(colorsObj)
    if (flatRoles.isNotEmpty()) {
        return MatugenPalette(light = flatRoles, dark = flatRoles)
    }

    return null
}

private fun extractRoles(obj: JsonObject): Map<String, String> {
    val out = HashMap<String, String>()
    val targetObj = (obj["colors"] as? JsonObject) ?: obj
    for ((key, primitive) in targetObj) {
        val canonicalKey = ROLE_NAME_MAPPINGS[key] ?: continue
        val content = (primitive as? JsonPrimitive)?.contentOrNull ?: continue
        if (content.startsWith("#")) {
            out[canonicalKey] = content
        }
    }
    return out
}

private fun String.toArgbColor(): Color? {
    val hex = trim().removePrefix("#")
    if (hex.length != 6 && hex.length != 8) return null
    val value = hex.toLongOrNull(16) ?: return null
    return if (hex.length == 6) Color(value or 0xFF000000) else Color(value)
}
