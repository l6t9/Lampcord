package me.lampu.lampcord.shared.ui.settings

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
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.ThemeStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import org.koin.compose.koinInject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

private enum class PaletteOption { DYNAMIC, CUSTOM }

@Composable
fun AppearanceSettings(
    onNavigateToTheming: () -> Unit,
    onNavigateToNavigation: () -> Unit,
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject(),
    themeStore: ThemeStore = koinInject()
) {
    SettingsSubScreen(
        title = "Appearance",
        onNavigateBack = onBack
    ) {
        AppearanceSettingsContent(
            onNavigateToTheming = onNavigateToTheming,
            onNavigateToNavigation = onNavigateToNavigation,
            settingsStore = settingsStore,
            themeStore = themeStore
        )
    }
}

@Composable
fun AppearanceSettingsContent(
    onNavigateToTheming: () -> Unit,
    onNavigateToNavigation: () -> Unit,
    settingsStore: SettingsStore = koinInject(),
    themeStore: ThemeStore = koinInject()
) {
    val platform = remember { me.lampu.lampcord.shared.utils.getPlatformName() }
    val isDesktop = platform != "android" && platform != "ios"
    var currentSubTab by remember { mutableStateOf("main") }

    if (currentSubTab == "navigation") {
        NavigationSettings(onBack = { currentSubTab = "main" }, settingsStore = settingsStore)
        return
    }

    fun updateTheme(theme: String) {
        settingsStore.updateUserSettings(UserSettings.Partial(theme = theme))
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Themes",
            items = buildList {
                themeStore.activeThemes.forEach { theme ->
                    add(Material3SettingsItem(
                        icon = null,
                        title = { Text(theme.manifest.name) },
                        description = theme.manifest.author?.let { { Text("by $it") } },
                        trailingContent = {
                            IconButton(onClick = { 
                                themeStore.toggleTheme(theme, false)
                            }) {
                                Icon(Icons.Filled.Close, "Remove Theme")
                            }
                        }
                    ))
                }
                add(Material3SettingsItem(
                    icon = Icons.Rounded.Palette,
                    title = { Text("Themer") },
                    description = { Text("Manage and edit custom themes") },
                    onClick = onNavigateToTheming
                ))
            }
        )

        Material3SettingsGroup(
            title = "Wallpaper",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Wallpaper & Colors") },
                    description = {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp), modifier = Modifier.padding(top = 8.dp)) {
                            AppearancePreview(settingsStore)
                            ThemeControls(settingsStore, ::updateTheme)
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Color palette",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Visual Style") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp)) {
                            SettingsButtonGroup(
                                options = ThemePaletteStyle.entries.toList(),
                                selectedOption = settingsStore.themePaletteStyle,
                                onOptionSelected = { settingsStore.themePaletteStyle = it },
                                iconProvider = { style: ThemePaletteStyle, isSelected ->
                                    when (style) {
                                        ThemePaletteStyle.TONAL_SPOT -> if (isSelected) Icons.Filled.Palette else Icons.Rounded.Palette
                                        ThemePaletteStyle.EXPRESSIVE -> if (isSelected) Icons.Filled.FormatPaint else Icons.Rounded.FormatPaint
                                        ThemePaletteStyle.VIBRANT -> if (isSelected) Icons.Filled.AutoAwesome else Icons.Rounded.AutoAwesome
                                        ThemePaletteStyle.RAINBOW -> if (isSelected) Icons.Filled.Texture else Icons.Rounded.Texture
                                        ThemePaletteStyle.MONOCHROME -> if (isSelected) Icons.Filled.Rectangle else Icons.Rounded.Rectangle
                                        ThemePaletteStyle.FRUIT_SALAD -> if (isSelected) Icons.Filled.Fastfood else Icons.Rounded.Fastfood
                                        else -> if (isSelected) Icons.Filled.Palette else Icons.Rounded.Palette
                                    }
                                },
                                labelProvider = { it.name.lowercase().replace('_', ' ').replaceFirstChar { char -> char.uppercase() } }
                            )
                        }
                    }
                ),
                Material3SettingsItem(
                    title = { Text("Source") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(16.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                val selectedOption = if (settingsStore.materialYou) PaletteOption.DYNAMIC else PaletteOption.CUSTOM
                                
                                SettingsButtonGroupCustomIcon(
                                    options = PaletteOption.entries,
                                    selectedOption = selectedOption,
                                    onOptionSelected = { option ->
                                        settingsStore.materialYou = option == PaletteOption.DYNAMIC
                                    },
                                    iconProvider = { option, isSelected ->
                                        when (option) {
                                            PaletteOption.DYNAMIC -> {
                                                Icon(
                                                    Icons.Rounded.AutoAwesome, 
                                                    null, 
                                                    modifier = Modifier.size(18.dp),
                                                    tint = if (isSelected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.primary
                                                )
                                            }
                                            PaletteOption.CUSTOM -> {
                                                val accentColor = remember(settingsStore.accentColor) {
                                                    try {
                                                        Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
                                                    } catch (_: Exception) {
                                                        Color(0xFF6750A4)
                                                    }
                                                }
                                                Box(
                                                    modifier = Modifier
                                                        .size(18.dp)
                                                        .clip(RoundedCornerShape(4.dp))
                                                        .background(accentColor)
                                                        .border(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f), RoundedCornerShape(4.dp))
                                                )
                                            }
                                        }
                                    },
                                    labelProvider = { 
                                        when (it) {
                                            PaletteOption.DYNAMIC -> "Dynamic"
                                            PaletteOption.CUSTOM -> "Custom"
                                        }
                                    }
                                )

                                ColorHexPicker(settingsStore)
                            }
                        }
                    }
                ),
                switchSettingsItem(
                    title = "Sync across clients",
                    description = "Sync your theme selection across all Discord clients.",
                    checked = settingsStore.syncAppearance,
                    onCheckedChange = { settingsStore.syncAppearance = it }
                )
            )
        )

        Material3SettingsGroup(
            title = "Animation",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Panel Animation") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp)) {
                            SettingsButtonGroup(
                                options = PanelAnimation.entries.toList(),
                                selectedOption = settingsStore.panelAnimation,
                                onOptionSelected = { settingsStore.panelAnimation = it },
                                iconProvider = { animation, isSelected ->
                                    when (animation) {
                                        PanelAnimation.MINIMAL -> if (isSelected) Icons.Filled.Speed else Icons.Rounded.Speed
                                        PanelAnimation.EXPRESSIVE -> if (isSelected) Icons.Filled.AutoAwesome else Icons.Rounded.AutoAwesome
                                    }
                                },
                                labelProvider = { it.name.lowercase().replaceFirstChar { char -> char.uppercase() } }
                            )
                        }
                    }
                )
            )
        )



        Material3SettingsGroup(
            title = "Display",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("UI Density") },
                    description = { 
                        Column(modifier = Modifier.padding(top = 4.dp)) {
                            Text("Adjust the space between server, channel, and member lists.")
                            Spacer(Modifier.height(12.dp))
                            SettingsButtonGroup(
                                options = me.lampu.lampcord.shared.settings.MessageSpacingMode.entries.toList(),
                                selectedOption = settingsStore.messageSpacingMode,
                                onOptionSelected = { settingsStore.messageSpacingMode = it },
                                iconProvider = null,
                                labelProvider = {
                                    when (it) {
                                        me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> "Compact"
                                        me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> "Default"
                                        me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> "Spacious"
                                    }
                                }
                            )
                        }
                    }
                ),
                Material3SettingsItem(
                    icon = Icons.Filled.BottomAppBar,
                    title = { Text("Navigation Tabs") },
                    description = { Text("Reorder and toggle visibility of navigation tabs") },
                    onClick = {
                        if (me.lampu.lampcord.shared.utils.getPlatformName() != "android" && me.lampu.lampcord.shared.utils.getPlatformName() != "ios") {
                            currentSubTab = "navigation"
                        } else {
                            onNavigateToNavigation()
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Typography",
            items = buildList {
                add(Material3SettingsItem(
                    title = { Text("App Font") },
                    description = {
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.padding(top = 8.dp)) {
                            SettingsButtonGroup(
                                options = FontOption.entries.toList(),
                                selectedOption = settingsStore.appFont,
                                onOptionSelected = { settingsStore.appFont = it },
                                labelProvider = {
                                    when (it) {
                                        FontOption.SYSTEM -> "System"
                                        FontOption.INTER -> "Inter"
                                        FontOption.GOOGLE_SANS -> "Google Sans"
                                        FontOption.MAPLE_MONO -> "Maple Mono"
                                        FontOption.CUSTOM -> "Custom"
                                    }
                                }
                            )

                            if (settingsStore.appFont == FontOption.CUSTOM) {
                                var showFontPicker by remember { mutableStateOf(false) }

                                FilePicker(
                                    show = showFontPicker,
                                    onFileSelected = { files ->
                                        files.firstOrNull()?.let { (path, _) ->
                                            settingsStore.customFontPath = path
                                        }
                                        showFontPicker = false
                                    },
                                    onDismiss = { showFontPicker = false }
                                )

                                Surface(
                                    onClick = { showFontPicker = true },
                                    modifier = Modifier.fillMaxWidth(),
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
                                                if (settingsStore.customFontPath.isEmpty()) "Choose font file" else settingsStore.customFontPath.split("/").last(),
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
                                            IconButton(onClick = { settingsStore.customFontPath = "" }) {
                                                Icon(Icons.Rounded.Close, null)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                ))

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
private fun AppearancePreview(settingsStore: SettingsStore) {
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

@OptIn(ExperimentalEncodingApi::class)
@Composable
private fun ThemeControls(settingsStore: SettingsStore, updateTheme: (String) -> Unit) {
    Column(
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        SettingsButtonGroup(
            options = listOf(ThemeMode.AUTO, null),
            selectedOption = if (settingsStore.themeMode == ThemeMode.AUTO) ThemeMode.AUTO else if (settingsStore.pureBlack) null else ThemeMode.AUTO,
            onOptionSelected = { mode: ThemeMode? ->
                if (mode == null) {
                    settingsStore.themeMode = ThemeMode.DARK
                    settingsStore.pureBlack = true
                    if (settingsStore.syncAppearance) updateTheme("dark")
                } else {
                    settingsStore.themeMode = mode
                    settingsStore.pureBlack = false
                    if (settingsStore.syncAppearance) updateTheme("dark")
                }
            },
            iconProvider = { mode, isSelected ->
                when (mode) {
                    ThemeMode.AUTO -> if (isSelected) Icons.Filled.Sync else Icons.Rounded.Sync
                    null -> if (isSelected) Icons.Filled.Bedtime else Icons.Rounded.Bedtime
                    else -> null
                }
            },
            labelProvider = { mode: ThemeMode? -> if (mode == ThemeMode.AUTO) "Auto" else "Pure Black" }
        )

        SettingsLargeButtonGroup(
            options = listOf(ThemeMode.LIGHT, ThemeMode.DARK),
            selectedOption = if (settingsStore.themeMode == ThemeMode.LIGHT) ThemeMode.LIGHT else ThemeMode.DARK,
            onOptionSelected = { mode: ThemeMode ->
                settingsStore.themeMode = mode
                settingsStore.pureBlack = false
                if (settingsStore.syncAppearance) updateTheme(if (mode == ThemeMode.LIGHT) "light" else "dark")
            },
            iconProvider = { mode: ThemeMode, isSelected -> 
                when (mode) {
                    ThemeMode.LIGHT -> if (isSelected) Icons.Filled.LightMode else Icons.Rounded.LightMode
                    else -> if (isSelected) Icons.Filled.DarkMode else Icons.Rounded.DarkMode
                }
            },
            labelProvider = { mode: ThemeMode -> if (mode == ThemeMode.LIGHT) "Light" else "Dark" }
        )
        
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

    if (showPicker) {
        Popup(
            alignment = Alignment.TopStart,
            onDismissRequest = { showPicker = false },
            offset = IntOffset(0, 52)
        ) {
            val currentAccent = remember(settingsStore.accentColor) {
                try {
                    Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
                } catch (_: Exception) {
                    Color(0xFF6750A4)
                }
            }
            HsvColorPicker(
                initialColor = currentAccent,
                onColorSelected = {
                    val r = (it.red * 255).toInt().toString(16).padStart(2, '0')
                    val g = (it.green * 255).toInt().toString(16).padStart(2, '0')
                    val b = (it.blue * 255).toInt().toString(16).padStart(2, '0')
                    settingsStore.accentColor = "#$r$g$b"
                    settingsStore.materialYou = false
                    showPicker = false
                },
                onDismiss = { showPicker = false }
            )
        }
    }
}
