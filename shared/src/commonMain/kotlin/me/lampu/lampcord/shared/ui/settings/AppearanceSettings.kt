package me.lampu.lampcord.shared.ui.settings

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.screenshare_source
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.settings.ThemePaletteStyle
import me.lampu.lampcord.shared.settings.PanelAnimation
import me.lampu.lampcord.shared.settings.PanelType
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.importCustomFont
import org.koin.compose.koinInject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

private enum class PaletteOption { DYNAMIC, CUSTOM }

private enum class ThemeModeOption { LIGHT, DARK, PURE_BLACK, AUTO }

@Composable
fun AppearanceSettings(
    onNavigateToTheming: () -> Unit,
    onNavigateToNavigation: () -> Unit,
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject(),
) {
    SettingsSubScreen(
        title = "Appearance",
        onNavigateBack = onBack
    ) {
        AppearanceSettingsContent(
            onNavigateToTheming = onNavigateToTheming,
            onNavigateToNavigation = onNavigateToNavigation,
            settingsStore = settingsStore,
        )
    }
}

@Composable
fun AppearanceSettingsContent(
    onNavigateToTheming: () -> Unit,
    onNavigateToNavigation: () -> Unit,
    settingsStore: SettingsStore = koinInject(),
) {
    val platform = remember { me.lampu.lampcord.shared.utils.getPlatformName() }
    val isDesktop = platform != "android" && platform != "ios"
    val isMobile = !isDesktop

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Theme",
            items = listOf(
                Material3SettingsItem(
                    icon = Icons.Rounded.Palette,
                    title = { Text("Theme") },
                    description = { Text("Theme mode, colors, and custom themes") },
                    onClick = onNavigateToTheming
                )
            )
        )

        Material3SettingsGroup(
            title = "Wallpaper",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Wallpaper") },
                    description = {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp), modifier = Modifier.padding(top = 8.dp)) {
                            AppearancePreview(settingsStore)
                            WallpaperControls(settingsStore)
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Animation",
            items = listOf(
                selectSettingsItem(
                    title = "Panel Animation",
                    options = PanelAnimation.entries.toList(),
                    selectedOption = settingsStore.panelAnimation,
                    onOptionSelected = { settingsStore.panelAnimation = it },
                    optionLabel = { it.name.lowercase().replaceFirstChar { char -> char.uppercase() } },
                    optionIcon = { animation, isSelected ->
                        val icon = when (animation) {
                            PanelAnimation.MINIMAL -> if (isSelected) Icons.Filled.Speed else Icons.Rounded.Speed
                            PanelAnimation.EXPRESSIVE -> if (isSelected) Icons.Filled.AutoAwesome else Icons.Rounded.AutoAwesome
                        }
                        Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp))
                    },
                ),
                selectSettingsItem(
                    title = "Panel Layout",
                    options = PanelType.entries.toList(),
                    selectedOption = settingsStore.panelType,
                    onOptionSelected = { settingsStore.panelType = it },
                    optionLabel = { type ->
                        when (type) {
                            PanelType.CENTER -> "Center Panel"
                            PanelType.OVERLAPPING -> "Overlapping"
                        }
                    },
                    optionIcon = { type, isSelected ->
                        val icon = when (type) {
                            PanelType.CENTER -> if (isSelected) Icons.Filled.List else Icons.Rounded.List
                            PanelType.OVERLAPPING -> if (isSelected) Icons.Filled.MenuOpen else Icons.Rounded.MenuOpen
                        }
                        Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp))
                    },
                )
            )
        )



        Material3SettingsGroup(
            title = "Display",
            items = buildList {
                add(selectSettingsItem(
                    title = "UI Density",
                    description = "Adjust the space between server, channel, and member lists.",
                    options = me.lampu.lampcord.shared.settings.MessageSpacingMode.entries.toList(),
                    selectedOption = settingsStore.messageSpacingMode,
                    onOptionSelected = { settingsStore.messageSpacingMode = it },
                    optionLabel = {
                        when (it) {
                            me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> "Compact"
                            me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> "Default"
                            me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> "Spacious"
                        }
                    },
                ))
                if (!isMobile) return@buildList
                add(Material3SettingsItem(
                    icon = Icons.Filled.BottomAppBar,
                    title = { Text("Navigation Tabs") },
                    description = { Text("Reorder and toggle visibility of navigation tabs") },
                    onClick = { onNavigateToNavigation() }
                ))
            }
        )

        Material3SettingsGroup(
            title = "Typography",
            items = buildList {
                add(selectSettingsItem(
                    title = "App Font",
                    options = FontOption.entries.toList(),
                    selectedOption = settingsStore.appFont,
                    onOptionSelected = { settingsStore.appFont = it },
                    optionLabel = {
                        when (it) {
                            FontOption.SYSTEM -> "System"
                            FontOption.INTER -> "Inter"
                            FontOption.MAPLE_MONO -> "Maple Mono"
                            FontOption.CUSTOM -> "Custom"
                        }
                    },
                ))
                if (settingsStore.appFont == FontOption.CUSTOM) {
                    add(Material3SettingsItem(
                        title = { Text("Custom Font File") },
                        description = { CustomFontPicker(settingsStore) }
                    ))
                }

                add(Material3SettingsItem(
                    title = { Text("Font Scale: ${(settingsStore.fontScale * 100).toInt()}%") },
                    description = {
                        Slider(
                            value = settingsStore.fontScale,
                            onValueChange = { settingsStore.fontScale = it },
                            valueRange = 0.5f..2.0f,
                            modifier = Modifier.fillMaxWidth().padding(top = 8.dp)
                        )
                    }
                ))
            }
        )
    }
}

@Composable
internal fun AppearancePreview(settingsStore: SettingsStore) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(180.dp)
            .clip(RoundedCornerShape(20.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
        contentAlignment = Alignment.Center
    ) {
        if (settingsStore.chatBackground.isNotEmpty()) {
            AsyncImage(
                model = settingsStore.chatBackground,
                contentDescription = "Chat Background Preview",
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            
            Surface(
                onClick = { settingsStore.chatBackground = "" },
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(12.dp),
                shape = CircleShape,
                color = Color.Black.copy(alpha = 0.4f),
                contentColor = Color.White
            ) {
                Icon(
                    Icons.Rounded.Close,
                    contentDescription = "Remove Wallpaper",
                    modifier = Modifier.padding(8.dp).size(20.dp)
                )
            }
        } else {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Icon(
                    Icons.Rounded.Image,
                    contentDescription = null,
                    modifier = Modifier.size(48.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
                )
                Spacer(Modifier.height(8.dp))
                Text("No background set", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f))
            }
        }
    }
}

@Composable
private fun CustomFontPicker(settingsStore: SettingsStore) {
    var showFontPicker by remember { mutableStateOf(false) }

    FilePicker(
        show = showFontPicker,
        onFileSelected = { files ->
            files.firstOrNull()?.let { (name, bytes) ->
                settingsStore.customFontPath = importCustomFont(name, bytes)
                settingsStore.customFontName = name
            }
            showFontPicker = false
        },
        onDismiss = { showFontPicker = false }
    )

    Surface(
        onClick = { showFontPicker = true },
        modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(Icons.Rounded.FolderOpen, null, tint = MaterialTheme.colorScheme.primary)
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    if (settingsStore.customFontPath.isEmpty()) {
                        "Choose font file"
                    } else {
                        settingsStore.customFontName.ifEmpty {
                            settingsStore.customFontPath.split("/").last()
                        }
                    },
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Bold
                )
                if (settingsStore.customFontPath.isNotEmpty()) {
                    Text(
                        settingsStore.customFontPath,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
            if (settingsStore.customFontPath.isNotEmpty()) {
                IconButton(onClick = {
                    settingsStore.customFontPath = ""
                    settingsStore.customFontName = ""
                }) {
                    Icon(Icons.Rounded.Close, null)
                }
            }
        }
    }
}

@Composable
internal fun ColorPaletteSettings(settingsStore: SettingsStore) {
    Material3SettingsGroup(
        title = "Color palette",
        items = listOf(
            selectSettingsItem(
                title = "Visual Style",
                options = ThemePaletteStyle.entries.toList(),
                selectedOption = settingsStore.themePaletteStyle,
                onOptionSelected = { settingsStore.themePaletteStyle = it },
                optionLabel = { it.name.lowercase().replace('_', ' ').replaceFirstChar { char -> char.uppercase() } },
                optionIcon = { style: ThemePaletteStyle, isSelected ->
                    val icon = when (style) {
                        ThemePaletteStyle.TONAL_SPOT -> if (isSelected) Icons.Filled.Palette else Icons.Rounded.Palette
                        ThemePaletteStyle.EXPRESSIVE -> if (isSelected) Icons.Filled.FormatPaint else Icons.Rounded.FormatPaint
                        ThemePaletteStyle.VIBRANT -> if (isSelected) Icons.Filled.AutoAwesome else Icons.Rounded.AutoAwesome
                        ThemePaletteStyle.RAINBOW -> if (isSelected) Icons.Filled.Texture else Icons.Rounded.Texture
                        ThemePaletteStyle.MONOCHROME -> if (isSelected) Icons.Filled.Rectangle else Icons.Rounded.Rectangle
                        ThemePaletteStyle.FRUIT_SALAD -> if (isSelected) Icons.Filled.Fastfood else Icons.Rounded.Fastfood
                        else -> if (isSelected) Icons.Filled.Palette else Icons.Rounded.Palette
                    }
                    Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp))
                },
            ),
            selectSettingsItem(
                title = stringResource(Res.string.screenshare_source),
                options = PaletteOption.entries,
                selectedOption = if (settingsStore.materialYou) PaletteOption.DYNAMIC else PaletteOption.CUSTOM,
                onOptionSelected = { option ->
                    settingsStore.materialYou = option == PaletteOption.DYNAMIC
                },
                optionLabel = {
                    when (it) {
                        PaletteOption.DYNAMIC -> "Dynamic"
                        PaletteOption.CUSTOM -> "Custom"
                    }
                },
                optionIcon = { option, _ ->
                    when (option) {
                        PaletteOption.DYNAMIC -> Icon(
                            Icons.Rounded.AutoAwesome,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                        )
                        PaletteOption.CUSTOM -> EditableAccentSwatch(settingsStore)
                    }
                },
            ),
            Material3SettingsItem(
                title = { Text("Accent Color") },
                description = { ColorHexPicker(settingsStore) },
            ),
            switchSettingsItem(
                title = "Sync across clients",
                description = "Sync your theme selection across all Discord clients.",
                checked = settingsStore.syncAppearance,
                onCheckedChange = { settingsStore.syncAppearance = it }
            )
        )
    )
}

@Composable
internal fun ThemeModeControls(settingsStore: SettingsStore, updateTheme: (String) -> Unit) {
    val selectedOption = when {
        settingsStore.themeMode == ThemeMode.AUTO -> ThemeModeOption.AUTO
        settingsStore.pureBlack -> ThemeModeOption.PURE_BLACK
        settingsStore.themeMode == ThemeMode.LIGHT -> ThemeModeOption.LIGHT
        else -> ThemeModeOption.DARK
    }
    Material3SettingsGroup(
        title = "Theme Mode",
        items = listOf(
            selectSettingsItem(
                title = "Theme Mode",
                options = ThemeModeOption.entries.toList(),
                selectedOption = selectedOption,
                onOptionSelected = { option ->
                    when (option) {
                        ThemeModeOption.AUTO -> {
                            settingsStore.themeMode = ThemeMode.AUTO
                            settingsStore.pureBlack = false
                        }
                        ThemeModeOption.LIGHT -> {
                            settingsStore.themeMode = ThemeMode.LIGHT
                            settingsStore.pureBlack = false
                        }
                        ThemeModeOption.DARK -> {
                            settingsStore.themeMode = ThemeMode.DARK
                            settingsStore.pureBlack = false
                        }
                        ThemeModeOption.PURE_BLACK -> {
                            settingsStore.themeMode = ThemeMode.DARK
                            settingsStore.pureBlack = true
                        }
                    }
                    if (settingsStore.syncAppearance) {
                        updateTheme(
                            SettingsStore.discordThemeRawValue(settingsStore.themeMode, settingsStore.pureBlack)
                        )
                    }
                },
                optionLabel = {
                    when (it) {
                        ThemeModeOption.AUTO -> "Auto"
                        ThemeModeOption.LIGHT -> "Light"
                        ThemeModeOption.DARK -> "Dark"
                        ThemeModeOption.PURE_BLACK -> "Pure Black"
                    }
                },
                optionIcon = { option, isSelected ->
                    val icon = when (option) {
                        ThemeModeOption.AUTO -> if (isSelected) Icons.Filled.Sync else Icons.Rounded.Sync
                        ThemeModeOption.LIGHT -> if (isSelected) Icons.Filled.LightMode else Icons.Rounded.LightMode
                        ThemeModeOption.DARK -> if (isSelected) Icons.Filled.DarkMode else Icons.Rounded.DarkMode
                        ThemeModeOption.PURE_BLACK -> if (isSelected) Icons.Filled.Bedtime else Icons.Rounded.Bedtime
                    }
                    Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp))
                },
            )
        )
    )
}

@OptIn(ExperimentalEncodingApi::class)
@Composable
internal fun WallpaperControls(settingsStore: SettingsStore) {
    Column(
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        var showBackgroundPicker by remember { mutableStateOf(false) }
        
        FilePicker(
            show = showBackgroundPicker,
            onFileSelected = { files ->
                files.firstOrNull()?.let { (_, data) ->
                    val base64 = Base64.encode(data)
                    settingsStore.chatBackground = "data:image/png;base64,$base64"
                }
                showBackgroundPicker = false
            },
            onDismiss = { showBackgroundPicker = false }
        )
        
        Button(
            onClick = { showBackgroundPicker = true },
            modifier = Modifier.fillMaxWidth(),
            shape = CircleShape,
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
            contentPadding = PaddingValues(12.dp)
        ) {
            Icon(Icons.Rounded.Image, null, modifier = Modifier.size(18.dp))
            Spacer(Modifier.width(12.dp))
            Text("Choose background")
        }

        if (settingsStore.chatBackground.isNotEmpty()) {
            OutlinedButton(
                onClick = { settingsStore.chatBackground = "" },
                modifier = Modifier.fillMaxWidth(),
                shape = CircleShape,
                colors = ButtonDefaults.outlinedButtonColors(
                    contentColor = MaterialTheme.colorScheme.error,
                ),
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.5f)),
                contentPadding = PaddingValues(12.dp)
            ) {
                Icon(Icons.Rounded.Close, null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(12.dp))
                Text("Remove background", fontWeight = FontWeight.ExtraBold)
            }
        }
    }
}

@Composable
private fun accentColorValue(settingsStore: SettingsStore): Color =
    remember(settingsStore.accentColor) {
        try {
            Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
        } catch (_: Exception) {
            Color(0xFF6750A4)
        }
    }

@Composable
private fun AccentColorPickerPopup(
    show: Boolean,
    settingsStore: SettingsStore,
    onDismiss: () -> Unit,
    offset: IntOffset,
) {
    if (!show) return

    Popup(
        alignment = Alignment.TopStart,
        onDismissRequest = onDismiss,
        offset = offset
    ) {
        HsvColorPicker(
            initialColor = accentColorValue(settingsStore),
            onColorSelected = {
                val r = (it.red * 255).toInt().toString(16).padStart(2, '0')
                val g = (it.green * 255).toInt().toString(16).padStart(2, '0')
                val b = (it.blue * 255).toInt().toString(16).padStart(2, '0')
                settingsStore.accentColor = "#$r$g$b"
                settingsStore.materialYou = false
                onDismiss()
            },
            onDismiss = onDismiss
        )
    }
}

@Composable
private fun EditableAccentSwatch(settingsStore: SettingsStore) {
    var showPicker by remember { mutableStateOf(false) }

    Box(
        modifier = Modifier
            .size(24.dp)
            .clickableCursor { showPicker = true },
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier
                .size(18.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(accentColorValue(settingsStore))
                .border(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f), RoundedCornerShape(4.dp))
        )
        Box(
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .size(14.dp)
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.surface)
                .border(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f), CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                Icons.Rounded.Edit,
                contentDescription = "Edit accent color",
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(10.dp)
            )
        }
    }

    AccentColorPickerPopup(
        show = showPicker,
        settingsStore = settingsStore,
        onDismiss = { showPicker = false },
        offset = IntOffset(0, 28)
    )
}

@Composable
private fun ColorHexPicker(settingsStore: SettingsStore) {
    var showPicker by remember { mutableStateOf(false) }

    Surface(
        modifier = Modifier.fillMaxWidth().height(44.dp),
        shape = CircleShape,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.1f)),
        onClick = { showPicker = true }
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = settingsStore.accentColor.uppercase(),
                style = MaterialTheme.typography.bodyLarge.copy(
                    color = MaterialTheme.colorScheme.onSurface,
                    fontWeight = FontWeight.Bold,
                    fontFamily = FontFamily.Monospace
                )
            )

            Spacer(Modifier.weight(1f))

            Text(
                "Pick color",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.Bold
            )
        }
    }

    AccentColorPickerPopup(
        show = showPicker,
        settingsStore = settingsStore,
        onDismiss = { showPicker = false },
        offset = IntOffset(0, 52)
    )
}
