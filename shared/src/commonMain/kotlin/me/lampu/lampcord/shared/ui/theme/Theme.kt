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
    val isRequiemExpressive = paletteStyle == ThemePaletteStyle.REQUIEM_EXPRESSIVE
    val finalSeedColor = remember(seedColor, dynamicSeed, useMaterialYou, isRequiemExpressive) {
        when {
            isRequiemExpressive -> RequiemColors.Brand
            useMaterialYou && dynamicSeed != null -> dynamicSeed
            else -> seedColor ?: dynamicSeed ?: ColorLightPrimary
        }
    }

    val materialPaletteStyle = when(paletteStyle) {
        ThemePaletteStyle.TONAL_SPOT -> PaletteStyle.TonalSpot
        ThemePaletteStyle.VIBRANT -> PaletteStyle.Vibrant
        ThemePaletteStyle.EXPRESSIVE,
        ThemePaletteStyle.REQUIEM_EXPRESSIVE -> PaletteStyle.Expressive
        ThemePaletteStyle.RAINBOW -> PaletteStyle.Rainbow
        ThemePaletteStyle.FRUIT_SALAD -> PaletteStyle.FruitSalad
        ThemePaletteStyle.CONTENT -> PaletteStyle.Content
        ThemePaletteStyle.MONOCHROME -> PaletteStyle.Monochrome
        ThemePaletteStyle.NEUTRAL -> PaletteStyle.Neutral
    }

    val baseColorScheme = rememberPlatformColorScheme(
        seedColor = finalSeedColor,
        isDark = useDarkTheme,
        paletteStyle = materialPaletteStyle,
        useMaterialYou = useMaterialYou && useDynamicColor && !isRequiemExpressive
    )

    val requiemScheme = if (isRequiemExpressive) {
        baseColorScheme.requiemExpressive(useDarkTheme)
    } else {
        baseColorScheme
    }

    val colorScheme = remember(requiemScheme, pureBlack, useDarkTheme, themeStore.activeThemes, Settings.shared.transparencyMode) {
        var scheme = if (useDarkTheme && pureBlack) {
            requiemScheme.pureBlack()
        } else {
            requiemScheme
        }

        if (themeStore.activeThemes.isNotEmpty()) {
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

        val transparencyMode = Settings.shared.transparencyMode
        if (transparencyMode != me.lampu.lampcord.shared.settings.TransparencyMode.NONE && themeStore.themeBackgroundUrl != null) {
            val transparentScheme = scheme.copy(
                background = Color.Transparent,
                surface = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surface.copy(alpha = 0.7f),
                surfaceContainer = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceContainer.copy(alpha = 0.7f),
                surfaceContainerLow = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceContainerLow.copy(alpha = 0.7f),
                surfaceContainerLowest = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceContainerLowest.copy(alpha = 0.7f),
                surfaceContainerHigh = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceContainerHigh.copy(alpha = 0.7f),
                surfaceContainerHighest = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceContainerHighest.copy(alpha = 0.7f),
                surfaceVariant = if (transparencyMode == me.lampu.lampcord.shared.settings.TransparencyMode.FULL) Color.Transparent else scheme.surfaceVariant.copy(alpha = 0.7f),
            )
            transparentScheme
        } else {
            scheme
        }
    }

    val animatedColorScheme = if (Settings.shared.reduceMotion) {
        colorScheme
    } else {
        animateColorScheme(colorScheme = colorScheme)
    }

    val themeFontPath = themeStore.themeFontPath
    val finalFontOption = if (!themeFontPath.isNullOrEmpty()) FontOption.CUSTOM else appFont
    val finalFontPath = if (!themeFontPath.isNullOrEmpty()) themeFontPath else customFontPath

    val font = rememberAppFontFamily(finalFontOption, finalFontPath)

    val typography = remember(font, fontScale) { LampcordTypography(font, scale = fontScale) }

    MaterialExpressiveTheme(
        colorScheme = animatedColorScheme,
        typography = typography,
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

/** Material 3 Expressive colors mapped onto Requiem's desktop token palette. */
private fun ColorScheme.requiemExpressive(isDark: Boolean): ColorScheme = if (isDark) {
    copy(
        primary = RequiemColors.Brand,
        onPrimary = Color.White,
        primaryContainer = Color(0xFF3138A0),
        onPrimaryContainer = Color(0xFFE0E1FF),
        secondary = RequiemColors.TextMuted,
        onSecondary = RequiemColors.BackgroundLowest,
        secondaryContainer = RequiemColors.SurfaceHighest,
        onSecondaryContainer = RequiemColors.TextDefault,
        tertiary = Color(0xFFC8A2C8),
        onTertiary = Color(0xFF33203A),
        tertiaryContainer = Color(0xFF4A3650),
        onTertiaryContainer = Color(0xFFE7D3E7),
        error = RequiemColors.Red,
        onError = Color.White,
        errorContainer = Color(0xFF5C1F20),
        onErrorContainer = Color(0xFFFFDAD6),
        background = RequiemColors.BackgroundLowest,
        onBackground = RequiemColors.TextDefault,
        surface = RequiemColors.BackgroundLower,
        onSurface = RequiemColors.TextDefault,
        surfaceVariant = RequiemColors.BackgroundLow,
        onSurfaceVariant = RequiemColors.ChannelDefault,
        surfaceContainerLowest = RequiemColors.BackgroundLowest,
        surfaceContainerLow = RequiemColors.BackgroundLower,
        surfaceContainer = RequiemColors.ChatBackground,
        surfaceContainerHigh = RequiemColors.SurfaceHigh,
        surfaceContainerHighest = RequiemColors.SurfaceHighest,
        outline = RequiemColors.InteractiveMuted,
        outlineVariant = RequiemColors.BorderSubtle,
        scrim = Color.Black,
        inverseSurface = Color(0xFFE4E1E6),
        inverseOnSurface = RequiemColors.BackgroundLowest,
        inversePrimary = Color(0xFFBEC2FF),
    )
} else {
    copy(
        primary = RequiemColors.Brand,
        onPrimary = Color.White,
        primaryContainer = Color(0xFFE0E1FF),
        onPrimaryContainer = Color(0xFF000569),
        secondary = Color(0xFF5C5D72),
        onSecondary = Color.White,
        secondaryContainer = Color(0xFFE1E0F9),
        onSecondaryContainer = Color(0xFF191A2C),
        tertiary = Color(0xFF78536B),
        onTertiary = Color.White,
        tertiaryContainer = Color(0xFFFFD7EE),
        onTertiaryContainer = Color(0xFF2E1126),
        error = Color(0xFFBA1A1A),
        onError = Color.White,
        errorContainer = Color(0xFFFFDAD6),
        onErrorContainer = Color(0xFF410002),
        background = Color(0xFFFFFBFF),
        onBackground = Color(0xFF1B1B1F),
        surface = Color(0xFFFFFBFF),
        onSurface = Color(0xFF1B1B1F),
        surfaceVariant = Color(0xFFE3E1EC),
        onSurfaceVariant = Color(0xFF46464F),
        surfaceContainerLowest = Color.White,
        surfaceContainerLow = Color(0xFFF7F2FA),
        surfaceContainer = Color(0xFFF3EDF7),
        surfaceContainerHigh = Color(0xFFECE6F0),
        surfaceContainerHighest = Color(0xFFE6E0E9),
        outline = Color(0xFF777680),
        outlineVariant = Color(0xFFC7C5D0),
        scrim = Color.Black,
        inverseSurface = Color(0xFF303034),
        inverseOnSurface = Color(0xFFF3F0F4),
        inversePrimary = Color(0xFFBEC2FF),
    )
}