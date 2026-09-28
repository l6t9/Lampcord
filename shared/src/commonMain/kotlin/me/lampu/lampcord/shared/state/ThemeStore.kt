package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.model.LampcordTheme
import me.lampu.lampcord.shared.model.ThemeManifest
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.checkInternalFileExists
import me.lampu.lampcord.shared.utils.getInternalFilePath
import me.lampu.lampcord.shared.utils.writeInternalBytes

class ThemeStore(
    private val scope: CoroutineScope
) {
    var enabledThemeNames by mutableStateOf<Set<String>>(emptySet())
        private set

    var availableThemes by mutableStateOf<List<LampcordTheme>>(emptyList())
        private set

    var activeThemes by mutableStateOf<List<LampcordTheme>>(emptyList())
        private set
    
    var themeFontPath by mutableStateOf<String?>(null)

    private val json = Json { 
        ignoreUnknownKeys = true
        prettyPrint = true
        isLenient = true
    }
    private val httpClient = HttpClient()

    init {
        loadInstalledThemes()
        
        val savedEnabled = Settings.shared.activeThemeJson
        if (savedEnabled.isNotBlank()) {
            try {
                enabledThemeNames = json.decodeFromString<Set<String>>(savedEnabled)
            } catch (e: Exception) {
                // Fallback for old format (single theme JSON)
                try {
                    val oldTheme = json.decodeFromString<LampcordTheme>(savedEnabled)
                    enabledThemeNames = setOf(oldTheme.manifest.name)
                } catch (e2: Exception) {
                    enabledThemeNames = emptySet()
                }
            }
        }
        updateActiveThemes()
    }

    private fun updateActiveThemes() {
        activeThemes = availableThemes.filter { it.manifest.name in enabledThemeNames }
        activeThemes.lastOrNull()?.let { checkAndDownloadThemeFont(it) } ?: run { themeFontPath = null }
    }

    fun toggleTheme(theme: LampcordTheme, enabled: Boolean) {
        val newEnabled = enabledThemeNames.toMutableSet()
        if (enabled) newEnabled.add(theme.manifest.name) else newEnabled.remove(theme.manifest.name)
        enabledThemeNames = newEnabled
        Settings.shared.activeThemeJson = json.encodeToString(newEnabled)
        updateActiveThemes()
    }

    private fun checkAndDownloadThemeFont(theme: LampcordTheme) {
        val url = (theme.fonts["*"] as? JsonPrimitive)?.contentOrNull
            ?: (theme.fonts["whitney_medium"] as? JsonPrimitive)?.contentOrNull
        
        if (url == null) {
            themeFontPath = null
            return
        }

        if (!url.startsWith("http")) {
            themeFontPath = url
            return
        }

        val fileName = "theme_font_${url.hashCode()}.ttf"
        if (checkInternalFileExists(fileName)) {
            themeFontPath = getInternalFilePath(fileName)
            return
        }

        scope.launch {
            try {
                val response = httpClient.get(url)
                val bytes = response.body<ByteArray>()
                writeInternalBytes(fileName, bytes)
                themeFontPath = getInternalFilePath(fileName)
            } catch (e: Exception) {
                e.printStackTrace()
                themeFontPath = null
            }
        }
    }

    fun refreshThemes() {
        loadInstalledThemes()
    }

    private fun loadInstalledThemes() {
        try {
            val list = json.decodeFromString<List<LampcordTheme>>(Settings.shared.installedThemesJson)
            availableThemes = list
            updateActiveThemes()
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun saveInstalledThemes() {
        try {
            Settings.shared.installedThemesJson = json.encodeToString(availableThemes)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun installTheme(jsonString: String) {
        scope.launch {
            try {
                val rootElement = try {
                    json.parseToJsonElement(jsonString)
                } catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        me.lampu.lampcord.shared.utils.Logging.e("Theme", "Failed to parse JSON: ${e.message}")
                    }
                    return@launch
                }

                val root = when (rootElement) {
                    is JsonObject -> rootElement
                    is JsonArray -> if (rootElement.size > 0) rootElement[0] as? JsonObject else null
                    else -> null
                } ?: throw Exception("Invalid theme format: Root is not an object")

                val isLegacy = listOf("manifest", "simple_colors", "colors", "drawable_tints", "background", "fonts").none { root.containsKey(it) }
                
                val theme = if (isLegacy) {
                    me.lampu.lampcord.shared.utils.Logging.i("Theme", "Converting legacy theme...")
                    val manifest = mutableMapOf<String, JsonElement>()
                    val background = mutableMapOf<String, JsonElement>()
                    val fonts = mutableMapOf<String, JsonElement>()
                    val simpleColors = mutableMapOf<String, JsonElement>()
                    val colors = mutableMapOf<String, JsonElement>()
                    val drawableTints = mutableMapOf<String, JsonElement>()

                    for ((key, value) in root) {
                        when (key) {
                            "author", "version", "name", "license", "updater" -> manifest[key] = value
                            "background_url" -> background["url"] = value
                            "background_transparency" -> background["overlay_alpha"] = value
                            "font" -> fonts["*"] = value
                            "simple_accent_color" -> simpleColors["accent"] = value
                            "simple_bg_color" -> simpleColors["background"] = value
                            "simple_bg_secondary_color" -> simpleColors["background_secondary"] = value
                            "mention_highlight" -> simpleColors[key] = value
                            "active_channel_color", "statusbar_color", "input_background_color" -> 
                                simpleColors[key.removeSuffix("_color")] = value
                            else -> {
                                when {
                                    key.startsWith("color_") -> colors[key.substring(6)] = value
                                    key.startsWith("drawablecolor_") -> drawableTints[key.substring(14)] = value
                                    key.startsWith("font_") -> fonts[key.substring(5)] = value
                                    else -> { /* Unknown key */ }
                                }
                            }
                        }
                    }
                    LampcordTheme(
                        manifest = ThemeManifest(
                            name = (manifest["name"] as? JsonPrimitive)?.content ?: "Unknown Theme",
                            author = (manifest["author"] as? JsonPrimitive)?.content,
                            version = (manifest["version"] as? JsonPrimitive)?.content
                        ),
                        simple_colors = simpleColors,
                        colors = colors,
                        drawable_tints = drawableTints,
                        background = background,
                        fonts = fonts,
                        raws = emptyMap()
                    )
                } else {
                    val manifestObj = root["manifest"] as? JsonObject
                    val name = root["name"]?.jsonPrimitive?.content 
                        ?: manifestObj?.get("name")?.jsonPrimitive?.content
                        ?: "Unknown Theme"
                    val author = root["author"]?.jsonPrimitive?.content
                        ?: manifestObj?.get("author")?.jsonPrimitive?.content
                    val version = root["version"]?.jsonPrimitive?.content
                        ?: manifestObj?.get("version")?.jsonPrimitive?.content
                    
                    LampcordTheme(
                        manifest = ThemeManifest(
                            name = name,
                            author = author,
                            version = version
                        ),
                        simple_colors = (root["simple_colors"] ?: root["simpleColors"]) as? JsonObject ?: emptyMap(),
                        colors = root["colors"] as? JsonObject ?: emptyMap(),
                        drawable_tints = (root["drawable_tints"] ?: root["drawableTints"]) as? JsonObject ?: emptyMap(),
                        background = root["background"] as? JsonObject ?: emptyMap(),
                        fonts = root["fonts"] as? JsonObject ?: emptyMap(),
                        raws = root["raws"] as? JsonObject ?: emptyMap()
                    )
                }

                withContext(Dispatchers.Main) {
                    val newList = availableThemes.toMutableList()
                    val existingIndex = newList.indexOfFirst { it.manifest.name == theme.manifest.name }
                    if (existingIndex != -1) {
                        newList[existingIndex] = theme
                    } else {
                        newList.add(theme)
                    }
                    availableThemes = newList
                    updateActiveThemes()
                    saveInstalledThemes()
                    me.lampu.lampcord.shared.utils.Logging.i("Theme", "Successfully installed theme: ${theme.manifest.name}. Total: ${availableThemes.size}")
                }
            } catch (e: Exception) {
                withContext(Dispatchers.Main) {
                    me.lampu.lampcord.shared.utils.Logging.e("Theme", "Failed to install theme: ${e.message}")
                }
                e.printStackTrace()
            }
        }
    }

    fun deleteTheme(theme: LampcordTheme) {
        if (theme.manifest.name in enabledThemeNames) {
            val newEnabled = enabledThemeNames.toMutableSet()
            newEnabled.remove(theme.manifest.name)
            enabledThemeNames = newEnabled
            Settings.shared.activeThemeJson = json.encodeToString(newEnabled)
        }
        availableThemes = availableThemes.filter { it.manifest.name != theme.manifest.name }
        updateActiveThemes()
        saveInstalledThemes()
    }

    fun updateTheme(theme: LampcordTheme) {
        availableThemes = availableThemes.map { 
            if (it.manifest.name == theme.manifest.name) theme else it
        }
        if (theme.manifest.name in enabledThemeNames) {
            updateActiveThemes()
        }
        saveInstalledThemes()
    }

    fun loadThemeFromJson(jsonString: String, persist: Boolean = true) {
        try {
            val theme = json.decodeFromString<LampcordTheme>(jsonString)
            toggleTheme(theme, true)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    val themeBackgroundUrl: String?
        get() = (activeThemes.lastOrNull()?.background?.get("url") as? JsonPrimitive)?.contentOrNull
        
    val themeBackgroundAlpha: Float
        get() {
            val element = activeThemes.lastOrNull()?.background?.get("overlay_alpha") as? JsonPrimitive
            val value = element?.contentOrNull?.toFloatOrNull() ?: return 1.0f
            return if (value > 1.0f) value / 255f else value
        }

    val themeFontUrl: String?
        get() = (activeThemes.lastOrNull()?.fonts?.get("*") as? JsonPrimitive)?.contentOrNull
            ?: (activeThemes.lastOrNull()?.fonts?.get("whitney_medium") as? JsonPrimitive)?.contentOrNull

    fun resolveColor(colorName: String, colorScheme: androidx.compose.material3.ColorScheme): Color? {
        val themes = activeThemes
        if (themes.isEmpty()) return null
        
        val keys = when (colorName) {
            "primary" -> listOf("brand_new", "brand_500", "brand_new_500", "accent", "link", "colorControlBrandForeground", "colorControlActivated", "color_brand", "color_brand_500")
            "onPrimary" -> listOf("white_500", "white", "primary_100", "primary_light_100", "primary_dark_100")
            "primaryContainer" -> listOf("brand_500_alpha_20", "mention_highlight", "brand_new_160", "theme_chat_mention_background", "theme_chat_mentioned_me")
            "onPrimaryContainer" -> listOf("white_500", "brand_new_500", "theme_chat_mention_foreground", "primary_dark_100")
            
            "secondary" -> listOf("brand_new_360", "accent2", "brand_360", "colorControlActivated", "brand_new_530")
            "onSecondary" -> listOf("white_500", "primary_dark_100")
            "secondaryContainer" -> listOf("brand_new_360_alpha_20", "brand_360_alpha_20")
            "onSecondaryContainer" -> listOf("white_500", "primary_dark_100")

            "tertiary" -> listOf("uikit_btn_bg_color_selector_green", "green_360")
            "onTertiary" -> listOf("white_500", "primary_dark_100")

            "error" -> listOf("uikit_btn_bg_color_selector_red", "red_400")
            "onError" -> listOf("white_500")

            "background" -> listOf("primary_dark_600", "background", "colorBackgroundPrimary", "primary_600")
            "onBackground" -> listOf("primary_dark_200", "white_500", "primary_dark_100", "primary_100", "primary_200")
            
            "surface" -> listOf("primary_dark_800", "background_secondary", "colorSurface", "colorBackgroundFloating", "colorTabsBackground")
            "onSurface" -> listOf("primary_dark_200", "white_500", "primary_dark_100", "primary_100")
            
            "surfaceVariant" -> listOf("primary_dark_630", "background_secondary", "primary_630", "theme_chat_code", "input_background")
            "onSurfaceVariant" -> listOf("primary_dark_330", "primary_dark_400", "primary_300")
            
            "surfaceContainer" -> listOf("primary_dark_630", "background_secondary", "primary_700", "colorBackgroundTertiary", "colorBackgroundSecondary")
            "surfaceContainerLow" -> listOf("primary_dark_600", "background", "primary_600")
            "surfaceContainerLowest" -> listOf("primary_dark_700", "background_tertiary", "primary_700", "statusbar")
            "surfaceContainerHigh" -> listOf("primary_dark_630", "background_secondary", "primary_630")
            "surfaceContainerHighest" -> listOf("primary_dark_630", "background_secondary")

            "outline" -> listOf("primary_dark_400", "primary_400", "theme_chat_codeblock_border", "primary_660")
            "outlineVariant" -> listOf("primary_dark_500", "primary_500")
            
            "scrim" -> listOf("background_floating", "black_alpha_10")
            "inverseSurface" -> listOf("white_500", "white")
            "inverseOnSurface" -> listOf("primary_dark_800", "primary_800")
            "inversePrimary" -> listOf("brand_new")
            
            else -> listOf(colorName)
        }

        for (theme in themes.asReversed()) {
            for (key in keys) {
                theme.colors[key]?.let { element ->
                    val colorValue = if (element is JsonPrimitive) element.content else element.toString()
                    parseColorValue(colorValue, colorScheme)?.let { return it }
                }
            }

            for (key in keys) {
                theme.simple_colors[key]?.let { element ->
                    val colorValue = if (element is JsonPrimitive) element.content else element.toString()
                    parseColorValue(colorValue, colorScheme)?.let { return it }
                }
            }
        }
        
        return null
    }

    private fun parseColorValue(value: String, colorScheme: androidx.compose.material3.ColorScheme): Color? {
        if (value.startsWith("#")) {
            return try {
                val hex = value.removePrefix("#")
                if (hex.length == 6) {
                    Color(hex.toLong(16) or 0xFF000000)
                } else {
                    Color(hex.toLong(16))
                }
            } catch (e: Exception) {
                null
            }
        }
        
        if (value.startsWith("system_")) {
            return when {
                value.contains("accent1") -> colorScheme.primary
                value.contains("accent2") -> colorScheme.secondary
                value.contains("accent3") -> colorScheme.tertiary
                value.contains("neutral1") -> colorScheme.surface
                value.contains("neutral2") -> colorScheme.surfaceVariant
                else -> null
            }
        }
        
        // Also support integer colors (Aliucord format)
        return try {
            // Aliucord uses 32-bit signed integers for colors
            val intColor = value.toInt()
            Color(intColor)
        } catch (e: Exception) {
            try {
                val longColor = value.toLong()
                if (longColor > 0xFFFFFFFFL) Color(longColor) else Color(longColor.toInt())
            } catch (e2: Exception) {
                null
            }
        }
    }
}
