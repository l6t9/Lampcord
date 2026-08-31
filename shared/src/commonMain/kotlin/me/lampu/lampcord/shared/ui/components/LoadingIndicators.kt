package me.lampu.lampcord.shared.ui.components

import androidx.compose.material3.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.draw.drawBehind
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import me.lampu.lampcord.shared.settings.Settings

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ContainedLoadingIndicator(
    modifier: Modifier = Modifier,
    containerColor: Color = LoadingIndicatorDefaults.containedContainerColor,
    indicatorColor: Color = LoadingIndicatorDefaults.containedIndicatorColor,
) {
    if (Settings.shared.reduceMotion) {
        StaticLoadingIndicator(modifier, containerColor, indicatorColor, 0.75f)
    } else {
        androidx.compose.material3.ContainedLoadingIndicator(
            modifier = modifier,
            containerColor = containerColor,
            indicatorColor = indicatorColor,
        )
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ContainedLoadingIndicator(
    progress: () -> Float,
    modifier: Modifier = Modifier,
    containerColor: Color = LoadingIndicatorDefaults.containedContainerColor,
    indicatorColor: Color = LoadingIndicatorDefaults.containedIndicatorColor,
) {
    if (Settings.shared.reduceMotion) {
        StaticLoadingIndicator(modifier, containerColor, indicatorColor, progress().coerceIn(0f, 1f))
    } else {
        androidx.compose.material3.ContainedLoadingIndicator(
            progress = progress,
            modifier = modifier,
            containerColor = containerColor,
            indicatorColor = indicatorColor,
        )
    }
}

@Composable
private fun StaticLoadingIndicator(
    modifier: Modifier,
    containerColor: Color,
    indicatorColor: Color,
    progress: Float
) {
    Box(
        modifier = modifier
            .background(containerColor, CircleShape)
            .drawBehind {
                val strokeWidth = (size.minDimension * 0.1f).coerceAtLeast(2f)
                drawArc(
                    color = indicatorColor,
                    startAngle = -90f,
                    sweepAngle = progress * 360f,
                    useCenter = false,
                    style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
                )
            }
    )
}

