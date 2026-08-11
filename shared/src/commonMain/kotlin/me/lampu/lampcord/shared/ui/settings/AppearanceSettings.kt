package me.lampu.lampcord.shared.ui.settings

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.interaction.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.settings.ThemePaletteStyle
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.utils.FilePicker
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

private enum class PaletteOption { DYNAMIC, CUSTOM }

data class DesktopThemePalette(
    val name: String,
    val seedColor: Color,
)

private val desktopPaletteColors =
    listOf(
        DesktopThemePalette("Dynamic", Color.Transparent),
        DesktopThemePalette("Baseline", Color(0xFF6750A4)),
        DesktopThemePalette("Teal", Color(0xFF4ECDC4)),
        DesktopThemePalette("Sage", Color(0xFF96E6A1)),
        DesktopThemePalette("Orange", Color(0xFFFFB347)),
        DesktopThemePalette("Crimson", Color(0xFFEC5464)),
        DesktopThemePalette("Rose", Color(0xFFD81B60)),
        DesktopThemePalette("Purple", Color(0xFF8E24AA)),
        DesktopThemePalette("Deep Purple", Color(0xFF5E35B1)),
        DesktopThemePalette("Indigo", Color(0xFF3949AB)),
        DesktopThemePalette("Blue", Color(0xFF1E88E5)),
        DesktopThemePalette("Sky Blue", Color(0xFF039BE5)),
        DesktopThemePalette("Cyan", Color(0xFF00ACC1)),
        DesktopThemePalette("Green", Color(0xFF43A047)),
        DesktopThemePalette("Light Green", Color(0xFF7CB342)),
        DesktopThemePalette("Lime", Color(0xFFC0CA33)),
        DesktopThemePalette("Yellow", Color(0xFFFDD835)),
        DesktopThemePalette("Amber", Color(0xFFFFB300)),
        DesktopThemePalette("Deep Orange", Color(0xFFF4511E)),
        DesktopThemePalette("Brown", Color(0xFF6D4C41)),
        DesktopThemePalette("Grey", Color(0xFF757575)),
        DesktopThemePalette("Blue Grey", Color(0xFF546E7A)),
    )

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun AppearanceSettings(chatState: ChatState) {
    val colorScheme = MaterialTheme.colorScheme
    val isSystemDark = isSystemInDarkTheme()
    val isDark = when (chatState.settingsStore.themeMode) {
        ThemeMode.DARK -> true
        ThemeMode.LIGHT -> false
        else -> isSystemDark
    }
    
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    fun updateTheme(theme: String) {
        chatState.updateUserSettings(UserSettings.Partial(theme = theme))
    }

    if (!isMobile) {
        DesktopAppearanceSettings(chatState, colorScheme, isDark, ::updateTheme)
    } else {
        MobileAppearanceSettings(chatState, colorScheme, ::updateTheme)
    }
}

@OptIn(ExperimentalEncodingApi::class)
@Composable
private fun DesktopAppearanceSettings(
    chatState: ChatState,
    colorScheme: ColorScheme,
    isDark: Boolean,
    updateTheme: (String) -> Unit
) {
    DesktopSettingsLayout {
        DesktopSettingsSection(
            title = "Wallpaper & Colors",
            icon = Icons.Filled.Monitor
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                // Main Row: Preview and Theme Controls
                Row(
                    modifier = Modifier.fillMaxWidth().height(220.dp),
                    horizontalArrangement = Arrangement.spacedBy(20.dp)
                ) {
                    // Preview Area
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight()
                            .clip(RoundedCornerShape(24.dp))
                            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
                        contentAlignment = Alignment.Center
                    ) {
                        if (chatState.settingsStore.chatBackground.isNotEmpty()) {
                             AsyncImage(
                                model = chatState.settingsStore.chatBackground,
                                contentDescription = "Chat Background Preview",
                                modifier = Modifier.fillMaxSize(),
                                contentScale = ContentScale.Crop
                            )
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

                    // Theme Controls
                    Column(
                        modifier = Modifier.width(280.dp).fillMaxHeight(),
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        // Auto / Pure Black row
                        DesktopButtonGroupSelection(
                            options = listOf(ThemeMode.AUTO, null),
                            selectedOption = if (chatState.settingsStore.themeMode == ThemeMode.AUTO) ThemeMode.AUTO else if (chatState.settingsStore.pureBlack) null else ThemeMode.AUTO,
                            onOptionSelected = { mode: ThemeMode? ->
                                if (mode == null) {
                                    chatState.settingsStore.themeMode = ThemeMode.DARK
                                    chatState.settingsStore.pureBlack = true
                                    if (chatState.settingsStore.syncAppearance) updateTheme("dark")
                                } else {
                                    chatState.settingsStore.themeMode = mode
                                    chatState.settingsStore.pureBlack = false
                                    if (chatState.settingsStore.syncAppearance) updateTheme("dark")
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

                        // Light / Dark buttons
                        DesktopLargeButtonGroupSelection(
                            options = listOf(ThemeMode.LIGHT, ThemeMode.DARK),
                            selectedOption = if (chatState.settingsStore.themeMode == ThemeMode.LIGHT) ThemeMode.LIGHT else ThemeMode.DARK,
                            onOptionSelected = { mode: ThemeMode ->
                                chatState.settingsStore.themeMode = mode
                                chatState.settingsStore.pureBlack = false
                                if (chatState.settingsStore.syncAppearance) updateTheme(if (mode == ThemeMode.LIGHT) "light" else "dark")
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
                                    chatState.settingsStore.chatBackground = "data:image/png;base64,$base64"
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
                            Text("Choose background", fontWeight = FontWeight.ExtraBold)
                        }
                    }
                }

                // Palette Styles
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Palette style", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
                    DesktopButtonGroupSelection(
                        options = ThemePaletteStyle.entries.toList(),
                        selectedOption = chatState.settingsStore.themePaletteStyle,
                        onOptionSelected = { chatState.settingsStore.themePaletteStyle = it },
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

                // Color Palette - Custom Picker
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Color palette", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val selectedOption = if (chatState.settingsStore.materialYou) PaletteOption.DYNAMIC else PaletteOption.CUSTOM
                        
                        DesktopButtonGroupSelectionCustomIcon(
                            options = PaletteOption.entries,
                            selectedOption = selectedOption,
                            onOptionSelected = { option ->
                                if (option == PaletteOption.DYNAMIC) {
                                    chatState.settingsStore.materialYou = true
                                } else {
                                    chatState.settingsStore.materialYou = false
                                }
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
                                        val accentColor = remember(chatState.settingsStore.accentColor) {
                                            try {
                                                Color(chatState.settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
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

                        // Custom Color Picker Trigger
                        var showPicker by remember { mutableStateOf(false) }
                        
                        if (!chatState.settingsStore.materialYou) {
                            Surface(
                                modifier = Modifier.weight(1f).height(44.dp),
                                shape = CircleShape,
                                color = MaterialTheme.colorScheme.surfaceVariant,
                                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.1f)),
                                onClick = { showPicker = true }
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Text(
                                        text = chatState.settingsStore.accentColor.uppercase(),
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
                        }

                        if (showPicker) {
                            Popup(
                                alignment = Alignment.TopStart,
                                onDismissRequest = { showPicker = false },
                                offset = IntOffset(0, 52)
                            ) {
                                val currentAccent = remember(chatState.settingsStore.accentColor) {
                                    try {
                                        Color(chatState.settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
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
                                        chatState.settingsStore.accentColor = "#$r$g$b"
                                        chatState.settingsStore.materialYou = false
                                        showPicker = false
                                    },
                                    onDismiss = { showPicker = false }
                                )
                            }
                        }
                    }
                }

                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                AppearanceToggle("Sync across clients", chatState.settingsStore.syncAppearance, "Sync your theme selection across all Discord clients.") {
                    chatState.settingsStore.syncAppearance = it
                }
            }
        }

        DesktopSettingsSection(
            title = "Typography",
            icon = Icons.Filled.TextFields
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("App Font", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    DesktopButtonGroupSelection(
                        options = FontOption.entries.toList(),
                        selectedOption = chatState.settingsStore.appFont,
                        onOptionSelected = { chatState.settingsStore.appFont = it },
                        labelProvider = {
                            when (it) {
                                FontOption.SYSTEM -> "System"
                                FontOption.INTER -> "Inter"
                                FontOption.GOOGLE_SANS -> "Google Sans"
                                FontOption.MAPLE_MONO -> "Maple Mono"
                            }
                        }
                    )
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Font Scale: ${(chatState.settingsStore.fontScale * 100).toInt()}%", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    Slider(
                        value = chatState.settingsStore.fontScale,
                        onValueChange = { chatState.settingsStore.fontScale = it },
                        valueRange = 0.5f..2.0f,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        }
    }
}

@Composable
private fun PaletteGridItem(
    palette: DesktopThemePalette,
    isDark: Boolean,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val colorScheme = rememberDynamicColorScheme(
        seedColor = if (palette.seedColor == Color.Transparent) Color(0xFF6750A4) else palette.seedColor,
        isDark = isDark,
        style = PaletteStyle.TonalSpot,
    )

    Surface(
        onClick = onClick,
        modifier = modifier.height(68.dp),
        shape = RoundedCornerShape(16.dp),
        color = if (isSelected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant,
        border = if (isSelected) BorderStroke(2.dp, MaterialTheme.colorScheme.primary) else null
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(44.dp)
                    .clip(CircleShape)
            ) {
                if (palette.seedColor == Color.Transparent) {
                    Box(
                        modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.primary),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.AutoAwesome,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimary,
                            modifier = Modifier.size(24.dp)
                        )
                    }
                } else {
                    Canvas(modifier = Modifier.fillMaxSize()) {
                        drawRect(color = colorScheme.primary, size = Size(size.width, size.height / 2))
                        drawRect(color = colorScheme.secondary, topLeft = Offset(0f, size.height / 2), size = Size(size.width / 2, size.height / 2))
                        drawRect(color = colorScheme.tertiary, topLeft = Offset(size.width / 2, size.height / 2), size = Size(size.width / 2, size.height / 2))
                    }
                }
            }
            
            Text(
                text = palette.name,
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                color = if (isSelected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun AppearanceToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Column(modifier = Modifier.weight(1f)) {
            Text(label, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
            if (description != null) {
                Text(description, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun MobileAppearanceSettings(
    chatState: ChatState,
    colorScheme: ColorScheme,
    updateTheme: (String) -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        var themeExpanded by remember { mutableStateOf(false) }
        var fontExpanded by remember { mutableStateOf(false) }
        var cleanChannelsExpanded by remember { mutableStateOf(false) }

        Material3SettingsGroup(
            title = "Display",
            items = listOf(
                switchSettingsItem(
                    title = "Material You",
                    description = "Use system wallpaper colors as the base for the app's theme (Android 12+).",
                    checked = chatState.settingsStore.materialYou,
                    onCheckedChange = { chatState.settingsStore.materialYou = it }
                ),
                switchSettingsItem(
                    title = "Sync across clients",
                    description = "Sync your theme selection across all Discord clients.",
                    checked = chatState.settingsStore.syncAppearance,
                    onCheckedChange = { chatState.settingsStore.syncAppearance = it }
                ),
                expandableSettingsItem(
                    title = "Theme",
                    description = "Color palette and theme customization",
                    expanded = themeExpanded,
                    onToggle = { themeExpanded = !themeExpanded },
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth().padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(20.dp),
                    ) {
                        Text(
                            text = "Theme mode",
                            style = MaterialTheme.typography.titleMedium,
                            color = colorScheme.onSurface,
                        )

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            ModeCircle(
                                mode = ThemeMode.AUTO,
                                pureBlack = chatState.settingsStore.pureBlack,
                                showIcon = true,
                                isSelected = chatState.settingsStore.themeMode == ThemeMode.AUTO,
                                onClick = { chatState.settingsStore.themeMode = ThemeMode.AUTO },
                            )

                            ModeCircle(
                                mode = ThemeMode.LIGHT,
                                pureBlack = false,
                                showIcon = false,
                                isSelected = chatState.settingsStore.themeMode == ThemeMode.LIGHT,
                                onClick = { 
                                    chatState.settingsStore.themeMode = ThemeMode.LIGHT
                                    if (chatState.settingsStore.syncAppearance) updateTheme("light")
                                },
                            )

                            ModeCircle(
                                mode = ThemeMode.DARK,
                                pureBlack = false,
                                showIcon = false,
                                isSelected = chatState.settingsStore.themeMode == ThemeMode.DARK && !chatState.settingsStore.pureBlack,
                                onClick = {
                                    chatState.settingsStore.themeMode = ThemeMode.DARK
                                    chatState.settingsStore.pureBlack = false
                                    if (chatState.settingsStore.syncAppearance) updateTheme("dark")
                                },
                            )

                            ModeCircle(
                                mode = ThemeMode.DARK,
                                pureBlack = true,
                                showIcon = false,
                                isSelected = chatState.settingsStore.pureBlack && chatState.settingsStore.themeMode != ThemeMode.LIGHT,
                                onClick = {
                                    chatState.settingsStore.themeMode = ThemeMode.DARK
                                    chatState.settingsStore.pureBlack = true
                                    if (chatState.settingsStore.syncAppearance) updateTheme("dark")
                                },
                            )
                        }

                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            Text(
                                text = "Color palette",
                                style = MaterialTheme.typography.titleMedium,
                                color = colorScheme.onSurface,
                            )

                            LazyRow(
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                                contentPadding = PaddingValues(horizontal = 4.dp),
                            ) {
                                items(desktopPaletteColors) { palette ->
                                    val isDynamicPalette = palette.seedColor == Color.Transparent
                                    val currentAccentHex = chatState.settingsStore.accentColor.lowercase()
                                    val paletteAccentHex = if (isDynamicPalette) {
                                        ""
                                    } else {
                                        "#" + palette.seedColor.value.toString(16).substring(2).lowercase()
                                    }

                                    val isSelected = if (isDynamicPalette) {
                                        chatState.settingsStore.materialYou
                                    } else {
                                        !chatState.settingsStore.materialYou && (currentAccentHex == paletteAccentHex || (currentAccentHex == "#6750a4" && palette.name == "Baseline"))
                                    }

                                    PalettePreviewItem(
                                        palette = palette,
                                        isSelected = isSelected,
                                        onClick = {
                                            if (isDynamicPalette) {
                                                chatState.settingsStore.materialYou = true
                                            } else {
                                                chatState.settingsStore.materialYou = false
                                                chatState.settingsStore.accentColor = "#" + palette.seedColor.value.toString(16).substring(2)
                                            }
                                        },
                                    )
                                }
                            }

                            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                                Text(
                                    text = "Palette style",
                                    style = MaterialTheme.typography.titleSmall,
                                    color = colorScheme.onSurface,
                                )

                                LazyRow(
                                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                                    contentPadding = PaddingValues(horizontal = 4.dp),
                                ) {
                                    items(ThemePaletteStyle.entries.toList()) { paletteStyle ->
                                        ThemePaletteStyleChip(
                                            paletteStyle = paletteStyle,
                                            isSelected = chatState.settingsStore.themePaletteStyle == paletteStyle,
                                            onClick = { chatState.settingsStore.themePaletteStyle = paletteStyle },
                                        )
                                    }
                                }
                            }
                        }
                    }
                },
                expandableSettingsItem(
                    title = "App Font",
                    description = when (chatState.settingsStore.appFont) {
                        FontOption.SYSTEM -> "System Default"
                        FontOption.INTER -> "Inter"
                        FontOption.GOOGLE_SANS -> "Google Sans"
                        FontOption.MAPLE_MONO -> "Maple Mono"
                    },
                    expanded = fontExpanded,
                    onToggle = { fontExpanded = !fontExpanded },
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth().padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        FontOption.entries.forEach { option ->
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(8.dp))
                                    .clickable {
                                        chatState.settingsStore.appFont = option
                                        fontExpanded = false
                                    }.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                RadioButton(
                                    selected = chatState.settingsStore.appFont == option,
                                    onClick = {
                                        chatState.settingsStore.appFont = option
                                        fontExpanded = false
                                    },
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    text = when (option) {
                                        FontOption.SYSTEM -> "System Default"
                                        FontOption.INTER -> "Inter"
                                        FontOption.GOOGLE_SANS -> "Google Sans"
                                        FontOption.MAPLE_MONO -> "Maple Mono"
                                    },
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = colorScheme.onSurface,
                                )
                            }
                        }
                    }
                },
                Material3SettingsItem(
                    title = { Text("Font Scale: ${(chatState.settingsStore.fontScale * 100).toInt()}%") },
                    description = {
                        Slider(
                            value = chatState.settingsStore.fontScale,
                            onValueChange = { chatState.settingsStore.fontScale = it },
                            valueRange = 0.5f..2.0f,
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Messages",
            items = listOf(
                switchSettingsItem(
                    title = "Compact Messages",
                    description = "Use a denser layout for chat messages.",
                    checked = Settings.shared.compactMode,
                    onCheckedChange = { Settings.shared.compactMode = it }
                )
            )
        )

        Material3SettingsGroup(
            title = "Sidebar",
            items = listOf(
                expandableSettingsItem(
                    title = "Clean Channels",
                    description = "Simplify channel names by removing symbols and emojis.",
                    expanded = cleanChannelsExpanded,
                    onToggle = { cleanChannelsExpanded = !cleanChannelsExpanded }
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        switchSettingsItem(
                            title = "Remove Emojis",
                            checked = Settings.shared.cleanChannelsRemoveEmojis,
                            onCheckedChange = { Settings.shared.cleanChannelsRemoveEmojis = it }
                        ).let { Material3SettingsItemRow(it, isFirst = true, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Hide Symbols",
                            checked = Settings.shared.cleanChannelsHideSymbols,
                            onCheckedChange = { Settings.shared.cleanChannelsHideSymbols = it }
                        ).let { Material3SettingsItemRow(it, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Normalize Letters",
                            checked = Settings.shared.cleanChannelsNormalizeLetters,
                            onCheckedChange = { Settings.shared.cleanChannelsNormalizeLetters = it }
                        ).let { Material3SettingsItemRow(it, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Capitalize Categories",
                            checked = Settings.shared.cleanChannelsCapitalizeCategories,
                            onCheckedChange = { Settings.shared.cleanChannelsCapitalizeCategories = it }
                        ).let { Material3SettingsItemRow(it, isLast = true, horizontalPadding = 0.dp) }
                    }
                }
            )
        )
    }
}

@Composable
private fun ModeCircle(
    mode: ThemeMode,
    pureBlack: Boolean,
    showIcon: Boolean,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    val isSystemDark = isSystemInDarkTheme()
    val effectiveDark =
        when (mode) {
            ThemeMode.AUTO -> isSystemDark
            ThemeMode.DARK -> true
            ThemeMode.LIGHT -> false
        }

    val modeColorScheme =
        rememberDynamicColorScheme(
            seedColor = Color(0xFF6750A4),
            isDark = effectiveDark,
            style = PaletteStyle.TonalSpot,
        )

    val fillColor =
        when {
            pureBlack && effectiveDark -> Color.Black
            else -> modeColorScheme.surface
        }

    val borderWidth by animateDpAsState(
        targetValue = if (isSelected) 3.dp else 0.dp,
        animationSpec = spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMedium),
        label = "borderWidth",
    )
    val scale by animateFloatAsState(
        targetValue = if (isSelected) 1.05f else 1f,
        animationSpec = spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMedium),
        label = "scale",
    )
    val interactionSource = remember { MutableInteractionSource() }

    Box(
        modifier =
            Modifier
                .size(48.dp)
                .graphicsLayer {
                    scaleX = scale
                    scaleY = scale
                }.clip(CircleShape)
                .background(fillColor)
                .then(
                    if (borderWidth > 0.dp) {
                        Modifier.border(borderWidth, MaterialTheme.colorScheme.inversePrimary, CircleShape)
                    } else {
                        Modifier
                    },
                ).clickable(
                    interactionSource = interactionSource,
                    indication = ripple(),
                    onClick = onClick,
                ),
        contentAlignment = Alignment.Center,
    ) {
        when {
            showIcon -> {
                Icon(
                    imageVector = Icons.Rounded.Sync,
                    contentDescription = null,
                    tint = modeColorScheme.onSurface,
                    modifier = Modifier.size(20.dp),
                )
            }

            isSelected -> {
                AnimatedVisibility(visible = true) {
                    Icon(
                        imageVector = Icons.Rounded.AutoAwesome,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.inversePrimary,
                        modifier = Modifier.size(20.dp),
                    )
                }
            }

            !effectiveDark -> {
                Icon(
                    imageVector = Icons.Rounded.LightMode,
                    contentDescription = null,
                    tint = modeColorScheme.onSurface,
                    modifier = Modifier.size(18.dp),
                )
            }

            pureBlack -> {
                Icon(
                    imageVector = Icons.Rounded.Brightness4,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(18.dp),
                )
            }

            else -> {
                Icon(
                    imageVector = Icons.Rounded.DarkMode,
                    contentDescription = null,
                    tint = modeColorScheme.onSurface,
                    modifier = Modifier.size(18.dp),
                )
            }
        }
    }
}

@Composable
private fun ThemePaletteStyleChip(
    paletteStyle: ThemePaletteStyle,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    Box(
        modifier =
            Modifier
                .clip(RoundedCornerShape(999.dp))
                .background(
                    if (isSelected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceVariant,
                ).clickable { onClick() }
                .padding(horizontal = 14.dp, vertical = 8.dp),
    ) {
        Text(
            text =
                paletteStyle.name
                    .lowercase()
                    .replace('_', ' ')
                    .split(' ')
                    .joinToString(" ") { it.replaceFirstChar(Char::uppercase) },
            style = MaterialTheme.typography.labelMedium,
            color = if (isSelected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun PalettePreviewItem(
    palette: DesktopThemePalette,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    val isSystemDark = isSystemInDarkTheme()
    val colorScheme =
        rememberDynamicColorScheme(
            seedColor = if (palette.seedColor == Color.Transparent) Color(0xFF6750A4) else palette.seedColor,
            isDark = isSystemDark,
            style = PaletteStyle.TonalSpot,
        )

    val cornerRadius by animateDpAsState(
        targetValue = if (isSelected) 12.dp else 24.dp,
        animationSpec = spring(dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessMedium),
        label = "cornerRadius",
    )
    val borderWidth by animateDpAsState(
        targetValue = if (isSelected) 3.dp else 0.dp,
        animationSpec = spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMedium),
        label = "borderWidth",
    )
    val scale by animateFloatAsState(
        targetValue = if (isSelected) 1.08f else 1f,
        animationSpec = spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMedium),
        label = "scale",
    )

    val shape = RoundedCornerShape(cornerRadius)
    val interactionSource = remember { MutableInteractionSource() }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(
            modifier =
                Modifier
                    .size(48.dp)
                    .graphicsLayer {
                        scaleX = scale
                        scaleY = scale
                    }.clip(shape)
                    .then(
                        if (borderWidth > 0.dp) {
                            Modifier.border(borderWidth, MaterialTheme.colorScheme.inversePrimary, shape)
                        } else {
                            Modifier
                        },
                    ).clickable(
                        interactionSource = interactionSource,
                        indication = ripple(),
                        onClick = onClick,
                    ),
        ) {
            if (palette.seedColor == Color.Transparent) {
                Box(
                    modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceVariant),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        imageVector = Icons.Rounded.AutoAwesome,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(22.dp),
                    )
                }
            } else {
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val width = size.width
                    val height = size.height
                    drawRect(
                        color = colorScheme.onPrimary,
                        topLeft = Offset(0f, 0f),
                        size = Size(width, height / 2),
                    )
                    drawRect(
                        color = colorScheme.secondary,
                        topLeft = Offset(0f, height / 2),
                        size = Size(width / 2, height / 2),
                    )
                    drawRect(
                        color = colorScheme.tertiary,
                        topLeft = Offset(width / 2, height / 2),
                        size = Size(width / 2, height / 2),
                    )
                }
            }
        }

        Text(
            text = palette.name,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
