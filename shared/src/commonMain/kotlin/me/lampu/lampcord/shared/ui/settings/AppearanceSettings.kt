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
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.materialkolor.PaletteStyle
import com.materialkolor.rememberDynamicColorScheme
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.settings.ThemePaletteStyle
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings

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

@Composable
fun AppearanceSettings(chatState: ChatState) {
    val colorScheme = MaterialTheme.colorScheme
    
    fun updateTheme(theme: String) {
        chatState.updateUserSettings(UserSettings.Partial(theme = theme))
    }

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
                                    val paletteAccentHex = if (isDynamicPalette) "" else "#" + palette.seedColor.value.toString(16).substring(2).lowercase()

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
                    imageVector = Icons.Outlined.Sync,
                    contentDescription = null,
                    tint = modeColorScheme.onSurface,
                    modifier = Modifier.size(20.dp),
                )
            }

            isSelected -> {
                AnimatedVisibility(visible = true) {
                    Icon(
                        imageVector = Icons.Outlined.AutoAwesome,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.inversePrimary,
                        modifier = Modifier.size(20.dp),
                    )
                }
            }

            !effectiveDark -> {
                Icon(
                    imageVector = Icons.Outlined.LightMode,
                    contentDescription = null,
                    tint = modeColorScheme.onSurface,
                    modifier = Modifier.size(18.dp),
                )
            }

            pureBlack -> {
                Icon(
                    imageVector = Icons.Outlined.Brightness4,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(18.dp),
                )
            }

            else -> {
                Icon(
                    imageVector = Icons.Outlined.DarkMode,
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
                        imageVector = Icons.Outlined.AutoAwesome,
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
