package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
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

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun AppearanceSettings(
    settingsStore: SettingsStore = koinInject(),
    themeStore: ThemeStore = koinInject()
) {
    val colorScheme = MaterialTheme.colorScheme

    fun updateTheme(theme: String) {
        settingsStore.updateUserSettings(UserSettings.Partial(theme = theme))
    }

    SettingsLayout {
        SettingsSection(
            title = "Themes",
            icon = Icons.Filled.Palette
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                var showThemePicker by remember { mutableStateOf(false) }
                FilePicker(
                    show = showThemePicker,
                    onFileSelected = { files ->
                        files.firstOrNull()?.let { (_, data) ->
                            val json = data.decodeToString()
                            themeStore.loadThemeFromJson(json)
                        }
                        showThemePicker = false
                    },
                    onDismiss = { showThemePicker = false }
                )

                themeStore.activeTheme?.let { theme ->
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f),
                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.2f))
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(theme.manifest.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                                    theme.manifest.author?.let { Text("by $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                                IconButton(onClick = { 
                                    themeStore.activeTheme = null
                                    me.lampu.lampcord.shared.settings.Settings.shared.activeThemeJson = ""
                                }) {
                                    Icon(Icons.Filled.Close, "Remove Theme")
                                }
                            }
                        }
                    }
                }

                Button(
                    onClick = { showThemePicker = true },
                    modifier = Modifier.fillMaxWidth(),
                    shape = CircleShape
                ) {
                    Icon(Icons.Rounded.FolderOpen, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(12.dp))
                    Text("Load Aliucord Theme")
                }
            }
        }

        SettingsSection(
            title = "Wallpaper & Colors",
            icon = Icons.Filled.Monitor
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                BoxWithConstraints {
                    val isCompact = maxWidth < 600.dp
                    
                    if (isCompact) {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                            AppearancePreview(settingsStore)
                            ThemeControls(settingsStore, { updateTheme(it) })
                        }
                    } else {
                        Row(
                            modifier = Modifier.fillMaxWidth().height(220.dp),
                            horizontalArrangement = Arrangement.spacedBy(20.dp)
                        ) {
                            Box(modifier = Modifier.weight(1f)) {
                                AppearancePreview(settingsStore)
                            }
                            ThemeControls(settingsStore, { updateTheme(it) }, modifier = Modifier.width(280.dp))
                        }
                    }
                }

                // Palette Styles
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Palette style", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
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

                // Animation Style
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Panel Animation", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
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

                // Color Palette
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Color palette", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
                    
                    BoxWithConstraints {
                        val isCompact = maxWidth < 600.dp
                        
                        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
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

                                if (!settingsStore.materialYou && !isCompact) {
                                    ColorHexPicker(settingsStore)
                                }
                            }

                            if (!settingsStore.materialYou && isCompact) {
                                ColorHexPicker(settingsStore)
                            }
                        }
                    }
                }

                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Sync across clients", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                        Text("Sync your theme selection across all Discord clients.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    ExpressiveSwitch(checked = settingsStore.syncAppearance, onCheckedChange = { settingsStore.syncAppearance = it })
                }
            }
        }

        SettingsSection(
            title = "Chatbox Customization",
            icon = Icons.Rounded.Forum
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Chatbox Font Scale: ${(settingsStore.chatboxFontSize * 100).toInt()}%", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    Slider(
                        value = settingsStore.chatboxFontSize,
                        onValueChange = { settingsStore.chatboxFontSize = it },
                        valueRange = 0.5f..2.0f,
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Background Opacity: ${(settingsStore.chatboxBackgroundOpacity * 100).toInt()}%", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    Slider(
                        value = settingsStore.chatboxBackgroundOpacity,
                        onValueChange = { settingsStore.chatboxBackgroundOpacity = it },
                        valueRange = 0.0f..1.0f,
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Border Radius: ${settingsStore.chatboxBorderRadius}dp", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    Slider(
                        value = settingsStore.chatboxBorderRadius.toFloat(),
                        onValueChange = { settingsStore.chatboxBorderRadius = it.toInt() },
                        valueRange = 0.0f..32.0f,
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Hide Upload Button", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                    }
                    ExpressiveSwitch(checked = settingsStore.chatboxHideUploadButton, onCheckedChange = { settingsStore.chatboxHideUploadButton = it })
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Hide Emoji Button", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                    }
                    ExpressiveSwitch(checked = settingsStore.chatboxHideEmojiButton, onCheckedChange = { settingsStore.chatboxHideEmojiButton = it })
                }
            }
        }

        SettingsSection(
            title = "Visual Density",
            icon = Icons.Rounded.Forum
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("UI Density", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
                    Text("Adjust the space between server, channel, and member lists.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
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
        }

        SettingsSection(
            title = "Message Display",
            icon = Icons.Rounded.Forum
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text("Chat Bubbles", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                        Text("Display messages inside rounded chat bubbles.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    ExpressiveSwitch(checked = settingsStore.chatBubbles, onCheckedChange = { settingsStore.chatBubbles = it })
                }
            }
        }


        SettingsSection(
            title = "Typography",
            icon = Icons.Filled.TextFields
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("App Font", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
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

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Font Scale: ${(settingsStore.fontScale * 100).toInt()}%", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurfaceVariant)
                    Slider(
                        value = settingsStore.fontScale,
                        onValueChange = { settingsStore.fontScale = it },
                        valueRange = 0.5f..2.0f,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        }
    }
}

@Composable
private fun AppearancePreview(settingsStore: SettingsStore) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .clip(RoundedCornerShape(24.dp))
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
            
            // Explicit Remove button in preview area for better visibility
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
private fun ThemeControls(settingsStore: SettingsStore, updateTheme: (String) -> Unit, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxHeight(),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        // Auto / Pure Black row
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

        // Light / Dark buttons
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
            Text("Choose background", fontWeight = FontWeight.ExtraBold)
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
        color = MaterialTheme.colorScheme.surfaceVariant,
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
