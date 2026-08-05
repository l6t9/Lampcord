package me.lampu.lampcord.shared.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Color
import com.materialkolor.DynamicMaterialTheme
import com.materialkolor.PaletteStyle
import com.materialkolor.ktx.animateColorScheme

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun LampcordTheme(
    seedColor: Color? = null,
    useDarkTheme: Boolean = isSystemInDarkTheme(),
    pureBlack: Boolean = false,
    useDynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
    val dynamicSeed = rememberDynamicSeedColor()
    val finalSeedColor = seedColor ?: dynamicSeed ?: Blurple

    DynamicMaterialTheme(
        primary = finalSeedColor,
        isDark = useDarkTheme,
        style = PaletteStyle.TonalSpot,
        animate = true,
    ) {
        val baseColorScheme = MaterialTheme.colorScheme
        
        val colorScheme = remember(baseColorScheme, pureBlack, useDarkTheme) {
            if (useDarkTheme && pureBlack) {
                baseColorScheme.pureBlack()
            } else {
                baseColorScheme
            }
        }

        val animatedColorScheme = animateColorScheme(colorScheme = colorScheme)

        MaterialExpressiveTheme(
            colorScheme = animatedColorScheme,
            typography = LampcordTypography(),
            shapes = LampcordShapes,
            content = content
        )
    }
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
