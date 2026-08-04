package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import kotlin.math.max

/**
 * A Mesh Gradient-like background component for Lampcord.
 * Uses rotating radial gradients and blur to mimic Paicord's MeshGradient.
 */
@Composable
fun MeshGradientBackground(
    modifier: Modifier = Modifier,
    colors: List<Color> = listOf(
        Color(0xFF5865F2), // Blurple
        Color(0xFF3A3EAC), // Darker Blurple
        Color(0xFF282C54), // Indigo
        Color(0xFF1E193B)  // Midnight
    )
) {
    val infiniteTransition = rememberInfiniteTransition(label = "MeshGradient")
    
    val angle1 by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(tween(20000, easing = LinearEasing)),
        label = "angle1"
    )
    
    val angle2 by infiniteTransition.animateFloat(
        initialValue = 360f,
        targetValue = 0f,
        animationSpec = infiniteRepeatable(tween(25000, easing = LinearEasing)),
        label = "angle2"
    )

    val density = LocalDensity.current
    val blurRadius = with(density) { 80.dp.toPx() }

    Box(modifier = modifier.fillMaxSize().background(colors.last())) {
        Canvas(modifier = Modifier.fillMaxSize().graphicsLayer {
            // Apply a heavy blur to smooth out the gradients into a mesh look
            renderEffect = BlurEffect(blurRadius, blurRadius, TileMode.Clamp)
            compositingStrategy = CompositingStrategy.Offscreen
        }) {
            val canvasWidth = size.width
            val canvasHeight = size.height
            val maxDim = max(canvasWidth, canvasHeight)

            // Layer 1
            withTransform({
                rotate(angle1, Offset(canvasWidth / 2f, canvasHeight / 2f))
            }) {
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(colors[0], Color.Transparent),
                        center = Offset(canvasWidth * 0.2f, canvasHeight * 0.2f),
                        radius = maxDim * 0.8f
                    ),
                    radius = maxDim * 0.8f,
                    center = Offset(canvasWidth * 0.2f, canvasHeight * 0.2f)
                )
            }

            // Layer 2
            withTransform({
                rotate(angle2, Offset(canvasWidth / 2f, canvasHeight / 2f))
            }) {
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(colors[1], Color.Transparent),
                        center = Offset(canvasWidth * 0.8f, canvasHeight * 0.3f),
                        radius = maxDim * 0.7f
                    ),
                    radius = maxDim * 0.7f,
                    center = Offset(canvasWidth * 0.8f, canvasHeight * 0.3f)
                )
            }

            // Layer 3
            withTransform({
                rotate(angle1 * 0.5f, Offset(canvasWidth / 2f, canvasHeight / 2f))
            }) {
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(colors[2], Color.Transparent),
                        center = Offset(canvasWidth * 0.5f, canvasHeight * 0.8f),
                        radius = maxDim * 0.9f
                    ),
                    radius = maxDim * 0.9f,
                    center = Offset(canvasWidth * 0.5f, canvasHeight * 0.8f)
                )
            }
        }
        
        // Slight dark overlay to ensure readability
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.2f)))
    }
}
