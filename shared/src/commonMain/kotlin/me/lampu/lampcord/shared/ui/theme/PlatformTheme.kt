package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.materialkolor.PaletteStyle

import me.lampu.lampcord.shared.settings.FontOption
import androidx.compose.ui.text.font.FontFamily

@Composable
expect fun rememberDynamicSeedColor(): Color?

@Composable
expect fun rememberAppFontFamily(option: FontOption): FontFamily

@Composable
expect fun rememberPlatformColorScheme(
    seedColor: Color,
    isDark: Boolean,
    paletteStyle: PaletteStyle,
    useMaterialYou: Boolean
): ColorScheme
