package me.lampu.lampcord.shared.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Color
import com.materialkolor.PaletteStyle
import com.materialkolor.ktx.animateColorScheme
import me.lampu.lampcord.shared.settings.ThemePaletteStyle
import me.lampu.lampcord.shared.settings.FontOption
import me.lampu.lampcord.shared.settings.Settings

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun LampcordTheme(
    seedColor: Color? = null,
    useDarkTheme: Boolean = isSystemInDarkTheme(),
    pureBlack: Boolean = false,
    useDynamicColor: Boolean = true,
    paletteStyle: ThemePaletteStyle = Settings.shared.themePaletteStyle,
    useMaterialYou: Boolean = Settings.shared.materialYou,
    appFont: FontOption = Settings.shared.appFont,
    content: @Composable () -> Unit
) {
    val dynamicSeed = rememberDynamicSeedColor()
    val finalSeedColor = seedColor ?: dynamicSeed ?: ColorLightPrimary

    val materialPaletteStyle = when(paletteStyle) {
        ThemePaletteStyle.TONAL_SPOT -> PaletteStyle.TonalSpot
        ThemePaletteStyle.VIBRANT -> PaletteStyle.Vibrant
        ThemePaletteStyle.EXPRESSIVE -> PaletteStyle.Expressive
        ThemePaletteStyle.RAINBOW -> PaletteStyle.Rainbow
        ThemePaletteStyle.FRUIT_SALAD -> PaletteStyle.FruitSalad
        ThemePaletteStyle.MONOCHROME -> PaletteStyle.Monochrome
        ThemePaletteStyle.NEUTRAL -> PaletteStyle.Neutral
    }

    val baseColorScheme = rememberPlatformColorScheme(
        seedColor = finalSeedColor,
        isDark = useDarkTheme,
        paletteStyle = materialPaletteStyle,
        useMaterialYou = useMaterialYou && useDynamicColor
    )
    
    val colorScheme = remember(baseColorScheme, pureBlack, useDarkTheme) {
        if (useDarkTheme && pureBlack) {
            baseColorScheme.pureBlack()
        } else {
            baseColorScheme
        }
    }

    val animatedColorScheme = animateColorScheme(colorScheme = colorScheme)
    val font = rememberAppFontFamily(appFont)

    MaterialExpressiveTheme(
        colorScheme = animatedColorScheme,
        typography = LampcordTypography(font),
        shapes = LampcordShapes,
        content = content
    )
}

fun ColorScheme.pureBlack() = copy(
    surface = Color.Black,
    background = Color.Black,
    surfaceContainer = Color.Black,
    surfaceContainerHigh = Color.Black,
    surfaceContainerLow = Color.Black,
    surfaceContainerLowest = Color.Black,
    surfaceContainerHighest = Color.Black,
    surfaceVariant = Color.Black,
)
