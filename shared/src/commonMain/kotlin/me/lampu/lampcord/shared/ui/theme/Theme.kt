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
import me.lampu.lampcord.shared.state.ThemeStore
import org.koin.compose.koinInject

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
    fontScale: Float = Settings.shared.fontScale,
    customFontPath: String = Settings.shared.customFontPath,
    themeStore: ThemeStore = koinInject(),
    content: @Composable () -> Unit
) {
    val dynamicSeed = rememberDynamicSeedColor()
    val finalSeedColor = remember(seedColor, dynamicSeed, useMaterialYou) {
        if (useMaterialYou && dynamicSeed != null) {
            dynamicSeed
        } else {
            seedColor ?: dynamicSeed ?: ColorLightPrimary
        }
    }

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
    
    val colorScheme = remember(baseColorScheme, pureBlack, useDarkTheme, themeStore.activeTheme) {
        var scheme = if (useDarkTheme && pureBlack) {
            baseColorScheme.pureBlack()
        } else {
            baseColorScheme
        }
        
        // Apply theme overrides
        themeStore.activeTheme?.let {
            scheme = scheme.copy(
                primary = themeStore.resolveColor("primary", scheme) ?: scheme.primary,
                onPrimary = themeStore.resolveColor("onPrimary", scheme) ?: scheme.onPrimary,
                primaryContainer = themeStore.resolveColor("primaryContainer", scheme) ?: scheme.primaryContainer,
                onPrimaryContainer = themeStore.resolveColor("onPrimaryContainer", scheme) ?: scheme.onPrimaryContainer,
                secondary = themeStore.resolveColor("secondary", scheme) ?: scheme.secondary,
                onSecondary = themeStore.resolveColor("onSecondary", scheme) ?: scheme.onSecondary,
                secondaryContainer = themeStore.resolveColor("secondaryContainer", scheme) ?: scheme.secondaryContainer,
                onSecondaryContainer = themeStore.resolveColor("onSecondaryContainer", scheme) ?: scheme.onSecondaryContainer,
                tertiary = themeStore.resolveColor("tertiary", scheme) ?: scheme.tertiary,
                onTertiary = themeStore.resolveColor("onTertiary", scheme) ?: scheme.onTertiary,
                tertiaryContainer = themeStore.resolveColor("tertiaryContainer", scheme) ?: scheme.tertiaryContainer,
                onTertiaryContainer = themeStore.resolveColor("onTertiaryContainer", scheme) ?: scheme.onTertiaryContainer,
                error = themeStore.resolveColor("error", scheme) ?: scheme.error,
                onError = themeStore.resolveColor("onError", scheme) ?: scheme.onError,
                errorContainer = themeStore.resolveColor("errorContainer", scheme) ?: scheme.errorContainer,
                onErrorContainer = themeStore.resolveColor("onErrorContainer", scheme) ?: scheme.onErrorContainer,
                background = themeStore.resolveColor("background", scheme) ?: scheme.background,
                onBackground = themeStore.resolveColor("onBackground", scheme) ?: scheme.onBackground,
                surface = themeStore.resolveColor("surface", scheme) ?: scheme.surface,
                onSurface = themeStore.resolveColor("onSurface", scheme) ?: scheme.onSurface,
                surfaceVariant = themeStore.resolveColor("surfaceVariant", scheme) ?: scheme.surfaceVariant,
                onSurfaceVariant = themeStore.resolveColor("onSurfaceVariant", scheme) ?: scheme.onSurfaceVariant,
                outline = themeStore.resolveColor("outline", scheme) ?: scheme.outline,
                outlineVariant = themeStore.resolveColor("outlineVariant", scheme) ?: scheme.outlineVariant,
                scrim = themeStore.resolveColor("scrim", scheme) ?: scheme.scrim,
                inverseSurface = themeStore.resolveColor("inverseSurface", scheme) ?: scheme.inverseSurface,
                inverseOnSurface = themeStore.resolveColor("inverseOnSurface", scheme) ?: scheme.inverseOnSurface,
                inversePrimary = themeStore.resolveColor("inversePrimary", scheme) ?: scheme.inversePrimary,
                surfaceContainer = themeStore.resolveColor("surfaceContainer", scheme) ?: themeStore.resolveColor("surface", scheme) ?: scheme.surfaceContainer,
                surfaceContainerLow = themeStore.resolveColor("surfaceContainerLow", scheme) ?: themeStore.resolveColor("surface", scheme) ?: scheme.surfaceContainerLow,
                surfaceContainerLowest = themeStore.resolveColor("surfaceContainerLowest", scheme) ?: themeStore.resolveColor("surface", scheme) ?: scheme.surfaceContainerLowest,
                surfaceContainerHigh = themeStore.resolveColor("surfaceContainerHigh", scheme) ?: themeStore.resolveColor("surface", scheme) ?: scheme.surfaceContainerHigh,
                surfaceContainerHighest = themeStore.resolveColor("surfaceContainerHighest", scheme) ?: themeStore.resolveColor("surface", scheme) ?: scheme.surfaceContainerHighest,
            )
        }
        scheme
    }

    val animatedColorScheme = animateColorScheme(colorScheme = colorScheme)
    val font = rememberAppFontFamily(appFont, customFontPath)

    MaterialExpressiveTheme(
        colorScheme = animatedColorScheme,
        typography = LampcordTypography(font, scale = fontScale),
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
