package me.lampu.lampcord.shared.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

fun LampcordTypography(
    font: FontFamily = FontFamily.Default,
    scale: Float = 1.0f
): Typography {
    fun TextStyle.scaled() = copy(
        fontSize = fontSize * scale,
        lineHeight = lineHeight * scale,
        letterSpacing = letterSpacing * scale
    )

    return Typography(
        displayLarge = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 57.sp,
            lineHeight = 64.sp,
            letterSpacing = (-0.25).sp,
        ).scaled(),
        displayMedium = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 45.sp,
            lineHeight = 52.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        displaySmall = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 36.sp,
            lineHeight = 44.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        headlineLarge = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 32.sp,
            lineHeight = 40.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        headlineMedium = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 28.sp,
            lineHeight = 36.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        headlineSmall = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Bold,
            fontSize = 24.sp,
            lineHeight = 32.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        titleLarge = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.SemiBold,
            fontSize = 22.sp,
            lineHeight = 28.sp,
            letterSpacing = 0.sp,
        ).scaled(),
        titleMedium = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.SemiBold,
            fontSize = 16.sp,
            lineHeight = 24.sp,
            letterSpacing = 0.15.sp,
        ).scaled(),
        titleSmall = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.SemiBold,
            fontSize = 14.sp,
            lineHeight = 20.sp,
            letterSpacing = 0.1.sp,
        ).scaled(),
        bodyLarge = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Normal,
            fontSize = 16.sp,
            lineHeight = 24.sp,
            letterSpacing = 0.5.sp,
        ).scaled(),
        bodyMedium = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Normal,
            fontSize = 14.sp,
            lineHeight = 20.sp,
            letterSpacing = 0.25.sp,
        ).scaled(),
        bodySmall = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Normal,
            fontSize = 12.sp,
            lineHeight = 16.sp,
            letterSpacing = 0.4.sp,
        ).scaled(),
        labelLarge = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Medium,
            fontSize = 14.sp,
            lineHeight = 20.sp,
            letterSpacing = 0.1.sp,
        ).scaled(),
        labelMedium = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Medium,
            fontSize = 12.sp,
            lineHeight = 16.sp,
            letterSpacing = 0.5.sp,
        ).scaled(),
        labelSmall = TextStyle(
            fontFamily = font,
            fontWeight = FontWeight.Medium,
            fontSize = 11.sp,
            lineHeight = 16.sp,
            letterSpacing = 0.5.sp,
        ).scaled(),
    )
}
