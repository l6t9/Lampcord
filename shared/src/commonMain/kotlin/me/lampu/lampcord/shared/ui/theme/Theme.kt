package me.lampu.lampcord.shared.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.materialkolor.DynamicMaterialTheme
import com.materialkolor.PaletteStyle

private val LightColors = lightColorScheme(
    primary = ColorLightPrimary,
    onPrimary = ColorLightOnPrimary,
    primaryContainer = ColorLightPrimaryContainer,
    onPrimaryContainer = ColorLightOnPrimaryContainer,
    secondary = ColorLightSecondary,
    onSecondary = ColorLightOnSecondary,
    secondaryContainer = ColorLightSecondaryContainer,
    onSecondaryContainer = ColorLightOnSecondaryContainer,
    tertiary = ColorLightTertiary,
    onTertiary = ColorLightOnTertiary,
    tertiaryContainer = ColorLightTertiaryContainer,
    onTertiaryContainer = ColorLightOnTertiaryContainer,
    error = ColorLightError,
    errorContainer = ColorLightErrorContainer,
    onError = ColorLightOnError,
    onErrorContainer = ColorLightOnErrorContainer,
    background = ColorLightBackground,
    onBackground = ColorLightOnBackground,
    surface = ColorLightSurface,
    onSurface = ColorLightOnSurface,
    surfaceVariant = ColorLightSurfaceVariant,
    onSurfaceVariant = ColorLightOnSurfaceVariant,
    outline = ColorLightOutline,
    inverseOnSurface = ColorLightInverseOnSurface,
    inverseSurface = ColorLightInverseSurface,
    inversePrimary = ColorLightInversePrimary,
    surfaceTint = ColorLightSurfaceTint,
    outlineVariant = ColorLightOutlineVariant,
    scrim = ColorLightScrim,
)

private val DarkColors = darkColorScheme(
    primary = ColorDarkPrimary,
    onPrimary = ColorDarkOnPrimary,
    primaryContainer = ColorDarkPrimaryContainer,
    onPrimaryContainer = ColorDarkOnPrimaryContainer,
    secondary = ColorDarkSecondary,
    onSecondary = ColorDarkOnSecondary,
    secondaryContainer = ColorDarkSecondaryContainer,
    onSecondaryContainer = ColorDarkOnSecondaryContainer,
    tertiary = ColorDarkTertiary,
    onTertiary = ColorDarkOnTertiary,
    tertiaryContainer = ColorDarkTertiaryContainer,
    onTertiaryContainer = ColorDarkOnTertiaryContainer,
    error = ColorDarkError,
    errorContainer = ColorDarkErrorContainer,
    onError = ColorDarkOnError,
    onErrorContainer = ColorDarkOnErrorContainer,
    background = ColorDarkBackground,
    onBackground = ColorDarkOnBackground,
    surface = ColorDarkSurface,
    onSurface = ColorDarkOnSurface,
    surfaceVariant = ColorDarkSurfaceVariant,
    onSurfaceVariant = ColorDarkOnSurfaceVariant,
    outline = ColorDarkOutline,
    inverseOnSurface = ColorDarkInverseOnSurface,
    inverseSurface = ColorDarkInverseSurface,
    inversePrimary = ColorDarkInversePrimary,
    surfaceTint = ColorDarkSurfaceTint,
    outlineVariant = ColorDarkOutlineVariant,
    scrim = ColorDarkScrim,
)

val LampcordShapes = Shapes()

@Composable
fun LampcordTheme(
    seedColor: Color? = null,
    useDarkTheme: Boolean = isSystemInDarkTheme(),
    useDynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val dynamicSeed = rememberDynamicSeedColor()
    
    // If useDynamicColor is true and we have a seed, use DynamicMaterialTheme (from materialkolor)
    // Otherwise use standard MaterialTheme with OpenCord's fixed colors.
    val finalSeedColor = seedColor ?: dynamicSeed

    if (useDynamicColor && finalSeedColor != null) {
        DynamicMaterialTheme(
            primary = finalSeedColor,
            isDark = useDarkTheme,
            style = PaletteStyle.TonalSpot,
            animate = true,
        ) {
            MaterialTheme(
                typography = Typography,
                shapes = LampcordShapes,
                content = content
            )
        }
    } else {
        val colorScheme = if (useDarkTheme) DarkColors else LightColors
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            shapes = LampcordShapes,
            content = content
        )
    }
}
