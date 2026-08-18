package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class LampcordTheme(
    val manifest: ThemeManifest,
    val simple_colors: Map<String, JsonElement> = emptyMap(),
    val colors: Map<String, JsonElement> = emptyMap(),
    val drawable_tints: Map<String, JsonElement> = emptyMap(),
    val background: Map<String, JsonElement> = emptyMap(),
    val fonts: Map<String, JsonElement> = emptyMap()
)

@Serializable
data class ThemeManifest(
    val name: String,
    val author: String? = null,
    val version: String? = null,
    val license: String? = null,
    val updater: String? = null
)
