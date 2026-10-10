package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.basicMarquee
import me.lampu.lampcord.shared.settings.Settings
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.DisplayNameStyles
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.loadFont

object DisplayNameCatalog {
    object Effect {
        const val SOLID = 1
        const val GRADIENT = 2
        const val NEON = 3
        const val TOON = 4
        const val POP = 5
        const val GLOW = 6
        const val TEST_1 = 1001
        const val TEST_2 = 1002
        const val TEST_3 = 1003
        const val TEST_4 = 1004
    }

    object Fonts {
        const val BANGERS = 1
        const val BIO_RHYME = 2
        const val CHERRY_BOMB = 3
        const val CHICLE = 4
        const val COMPAGNON = 5
        const val MUSEO_MODERNO = 6
        const val NEO_CASTEL = 7
        const val PIXELIFY = 8
        const val RIBES = 9
        const val SINISTRE = 10
        const val DEFAULT = 11
        const val ZILLA_SLAB = 12
    }

    fun getFontFamily(fontId: Int?): FontFamily? = when (fontId) {
        Fonts.BANGERS -> FontFamily(loadFont("font/Bangers-Regular.ttf"))
        Fonts.BIO_RHYME -> FontFamily(loadFont("font/BioRhyme-Regular.ttf"))
        Fonts.CHERRY_BOMB -> FontFamily(loadFont("font/CherryBombOne-Regular.ttf"))
        Fonts.CHICLE -> FontFamily(loadFont("font/Chicle-Regular.ttf"))
        Fonts.COMPAGNON -> FontFamily(loadFont("font/Compagnon-Medium.otf"))
        Fonts.MUSEO_MODERNO -> FontFamily(loadFont("font/MuseoModerno-Regular.ttf"))
        Fonts.NEO_CASTEL -> FontFamily(loadFont("font/NeoCastel.otf"))
        Fonts.PIXELIFY -> FontFamily(loadFont("font/PixelifySans-Regular.otf"))
        Fonts.RIBES -> FontFamily(loadFont("font/Ribes-Black.otf"))
        Fonts.SINISTRE -> FontFamily(loadFont("font/Sinistre-Bold.otf"))
        Fonts.ZILLA_SLAB -> FontFamily(loadFont("font/ZillaSlab-SemiBold.ttf"))
        else -> null
    }

    fun getLetterSpacing(fontId: Int?): TextUnit = when (fontId) {
        Fonts.CHERRY_BOMB -> 0.04.sp
        Fonts.MUSEO_MODERNO -> 0.01.sp
        Fonts.NEO_CASTEL -> 0.02.sp
        Fonts.PIXELIFY -> 0.02.sp
        Fonts.SINISTRE -> 0.01.sp
        Fonts.ZILLA_SLAB -> 0.03.sp
        else -> TextUnit.Unspecified
    }

    fun isProfileEffect(effectId: Int): Boolean = when (effectId) {
        Effect.SOLID,
        Effect.GRADIENT,
        Effect.NEON,
        Effect.TOON,
        Effect.POP,
        Effect.GLOW,
        Effect.TEST_1,
        Effect.TEST_2,
        Effect.TEST_3,
        Effect.TEST_4 -> true
        else -> false
    }
}

@Composable
fun UsernameView(
    name: String,
    style: DisplayNameStyles?,
    modifier: Modifier = Modifier,
    baseStyle: TextStyle = MaterialTheme.typography.bodyMedium,
    color: Color = Color.Unspecified,
    roleGradient: List<Color>? = null,
    fontWeight: FontWeight? = null,
    maxLines: Int = 1,
    overflow: TextOverflow = TextOverflow.Ellipsis,
    marquee: Boolean = false,
    ignoreEffects: Boolean = false,
    ignoreColors: Boolean = false
) {
    val styleColors = style?.colors?.map { Color(it or 0xFF000000.toInt()) }
    val effectId = if (ignoreEffects) DisplayNameCatalog.Effect.SOLID else (style?.effect_id ?: if (!styleColors.isNullOrEmpty() && styleColors.size > 1) DisplayNameCatalog.Effect.GRADIENT else DisplayNameCatalog.Effect.SOLID)
    val fontFamily = remember(style?.font_id) {
        DisplayNameCatalog.getFontFamily(style?.font_id)
    }
    val letterSpacing = DisplayNameCatalog.getLetterSpacing(style?.font_id)

    val useStyleColors = !ignoreColors && !styleColors.isNullOrEmpty()
    
    val animateGradient = !Settings.shared.reduceMotion && getPlatformName() != "windows" && roleGradient != null && roleGradient.size > 1
    val animValue = if (animateGradient) {
        val infiniteTransition = rememberInfiniteTransition(label = "usernameGradient")
        val value by infiniteTransition.animateFloat(
            initialValue = 0f,
            targetValue = 1f,
            animationSpec = infiniteRepeatable(
                animation = tween(4000, easing = LinearEasing),
                repeatMode = RepeatMode.Restart
            ),
            label = "gradientOffset"
        )
        value
    } else {
        0f
    }

    val brush = remember(useStyleColors, styleColors, roleGradient, animValue) {
        if (roleGradient != null && roleGradient.size > 1) {
            val offset = animValue * 200f // 2x the period
            
            val animatedColors = when {
                roleGradient.size >= 3 -> listOf(roleGradient[0], roleGradient[1], roleGradient[2], roleGradient[0])
                roleGradient.size == 2 -> listOf(roleGradient[0], roleGradient[1], roleGradient[0])
                else -> roleGradient
            }
            
            Brush.linearGradient(
                colors = animatedColors,
                start = Offset(offset - 200f, 0f),
                end = Offset(offset, 0f),
                tileMode = TileMode.Repeated
            )
        } else if (useStyleColors && styleColors.size > 1) {
            Brush.linearGradient(colors = styleColors)
        } else if (useStyleColors && styleColors.size == 1 && effectId == DisplayNameCatalog.Effect.GRADIENT) {
            null
        } else null
    }

    val finalColor = if (roleGradient != null && roleGradient.isNotEmpty()) {
        roleGradient.first()
    } else if (color != Color.Unspecified) {
        color
    } else if (useStyleColors && !styleColors.isNullOrEmpty()) {
        styleColors.first()
    } else {
        color
    }
    
    val effectBaseColor = if (roleGradient != null && roleGradient.isNotEmpty()) {
        roleGradient.first()
    } else if (useStyleColors && !styleColors.isNullOrEmpty()) {
        styleColors.first()
    } else if (color != Color.Unspecified) {
        color
    } else {
        MaterialTheme.colorScheme.onSurface
    }

    val textModifier = if (marquee && Settings.shared.marqueeEnabled) {
        Modifier.basicMarquee(
            iterations = if (getPlatformName() == "windows") 1 else Int.MAX_VALUE,
            initialDelayMillis = 3000,
            velocity = 30.dp
        )
    } else Modifier

    Box(modifier = modifier, contentAlignment = Alignment.CenterStart) {
        if (effectId == DisplayNameCatalog.Effect.NEON) {
            val neonColor = effectBaseColor.copy(alpha = 0.6f)
            Text(
                text = name,
                style = baseStyle.copy(
                    color = Color.Transparent,
                    fontFamily = fontFamily ?: baseStyle.fontFamily,
                    letterSpacing = if (letterSpacing != TextUnit.Unspecified) letterSpacing else baseStyle.letterSpacing,
                    shadow = Shadow(color = neonColor, blurRadius = 16f, offset = androidx.compose.ui.geometry.Offset.Zero)
                ),
                maxLines = maxLines,
                overflow = overflow,
                modifier = textModifier
            )
        }

        Text(
            text = name,
            style = if (brush != null) {
                baseStyle.copy(
                    brush = brush,
                    fontFamily = fontFamily ?: baseStyle.fontFamily,
                    letterSpacing = if (letterSpacing != TextUnit.Unspecified) letterSpacing else baseStyle.letterSpacing,
                    fontWeight = fontWeight ?: baseStyle.fontWeight,
                    shadow = if (effectId == DisplayNameCatalog.Effect.GLOW) Shadow(
                        color = effectBaseColor.copy(alpha = 0.5f),
                        blurRadius = 8f
                    ) else baseStyle.shadow
                )
            } else {
                baseStyle.copy(
                    color = if (finalColor != Color.Unspecified) finalColor else MaterialTheme.colorScheme.onSurface,
                    fontFamily = fontFamily ?: baseStyle.fontFamily,
                    letterSpacing = if (letterSpacing != TextUnit.Unspecified) letterSpacing else baseStyle.letterSpacing,
                    fontWeight = fontWeight ?: baseStyle.fontWeight,
                    shadow = if (effectId == DisplayNameCatalog.Effect.NEON) Shadow(
                        color = finalColor.copy(alpha = 0.8f),
                        blurRadius = 12f
                    ) else baseStyle.shadow
                )
            },
            maxLines = maxLines,
            overflow = overflow,
            modifier = textModifier
        )
    }
}
