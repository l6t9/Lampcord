package me.lampu.lampcord.shared.ui.settings

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.interaction.*
import androidx.compose.foundation.layout.*
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

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun AppearanceSettings(chatState: ChatState) {
    val colorScheme = MaterialTheme.colorScheme

    fun updateTheme(theme: String) {
        chatState.updateUserSettings(UserSettings.Partial(theme = theme))
    }

    SettingsLayout {
        SettingsSection(
            title = "Wallpaper & Colors",
            icon = Icons.Filled.Monitor
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                BoxWithConstraints {
                    val isCompact = maxWidth < 600.dp
                    
                    if (isCompact) {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                            AppearancePreview(chatState)
                            ThemeControls(chatState, { updateTheme(it) })
                        }
                    } else {
                        Row(
                            modifier = Modifier.fillMaxWidth().height(220.dp),
                            horizontalArrangement = Arrangement.spacedBy(20.dp)
                        ) {
                            Box(modifier = Modifier.weight(1f)) {
                                AppearancePreview(chatState)
                            }
                            ThemeControls(chatState, { updateTheme(it) }, modifier = Modifier.width(280.dp))
                        }
                    }
                }

                // Palette Styles
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Palette style", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold, color = colorScheme.onSurface)
                    SettingsButtonGroup(
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
                                val selectedOption = if (chatState.settingsStore.materialYou) PaletteOption.DYNAMIC else PaletteOption.CUSTOM
                                
                                SettingsButtonGroupCustomIcon(
                                    options = PaletteOption.entries,
                                    selectedOption = selectedOption,
                                    onOptionSelected = { option ->
                                        chatState.settingsStore.materialYou = option == PaletteOption.DYNAMIC
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

                                if (!chatState.settingsStore.materialYou && !isCompact) {
                                    ColorHexPicker(chatState)
                                }
                            }

                            if (!chatState.settingsStore.materialYou && isCompact) {
                                ColorHexPicker(chatState)
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
                    ExpressiveSwitch(checked = chatState.settingsStore.syncAppearance, onCheckedChange = { chatState.settingsStore.syncAppearance = it })
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
                        selectedOption = chatState.settingsStore.appFont,
                        onOptionSelected = { chatState.settingsStore.appFont = it },
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

                    if (chatState.settingsStore.appFont == FontOption.CUSTOM) {
                        var showFontPicker by remember { mutableStateOf(false) }

                        FilePicker(
                            show = showFontPicker,
                            onFileSelected = { files ->
                                files.firstOrNull()?.let { (path, _) ->
                                    chatState.settingsStore.customFontPath = path
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
                                        if (chatState.settingsStore.customFontPath.isEmpty()) "Choose font file" else chatState.settingsStore.customFontPath.split("/").last(),
                                        style = MaterialTheme.typography.bodyLarge,
                                        fontWeight = FontWeight.Bold
                                    )
                                    if (chatState.settingsStore.customFontPath.isNotEmpty()) {
                                        Text(
                                            chatState.settingsStore.customFontPath,
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                }
                                if (chatState.settingsStore.customFontPath.isNotEmpty()) {
                                    IconButton(onClick = { chatState.settingsStore.customFontPath = "" }) {
                                        Icon(Icons.Rounded.Close, null)
                                    }
                                }
                            }
                        }
                    }
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
private fun AppearancePreview(chatState: ChatState) {
    Box(
        modifier = Modifier
            .fillMaxSize()
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
}

@OptIn(ExperimentalEncodingApi::class)
@Composable
private fun ThemeControls(chatState: ChatState, updateTheme: (String) -> Unit, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxHeight(),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        // Auto / Pure Black row
        SettingsButtonGroup(
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
        SettingsLargeButtonGroup(
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

        if (chatState.settingsStore.chatBackground.isNotEmpty()) {
            OutlinedButton(
                onClick = { chatState.settingsStore.chatBackground = "" },
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
private fun ColorHexPicker(chatState: ChatState) {
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



