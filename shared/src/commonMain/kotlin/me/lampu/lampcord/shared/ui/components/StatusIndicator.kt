package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Composable
fun StatusIndicator(
    status: String,
    modifier: Modifier = Modifier,
    size: Dp = 14.dp,
    borderColor: Color = Color.Transparent,
    borderWidth: Dp = 2.dp,
    backgroundColor: Color = Color.Black
) {
    val statusColor = when (status) {
        "online" -> Color(0xFF23A559)
        "idle" -> Color(0xFFF0B232)
        "dnd" -> Color(0xFFF23F43)
        else -> Color(0xFF80848E)
    }

    Box(
        modifier = modifier
            .size(size)
            .background(borderColor, CircleShape)
            .padding(borderWidth)
            .clip(CircleShape)
            .background(statusColor),
        contentAlignment = Alignment.Center
    ) {
        when (status) {
            "dnd" -> {
                // Modern Discord DND dash
                Box(
                    modifier = Modifier
                        .fillMaxWidth(0.65f)
                        .height(size * 0.18f)
                        .background(backgroundColor)
                )
            }
            "idle" -> {
                // Modern Discord Idle Moon cutout
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .offset(x = (-size * 0.25f), y = (-size * 0.25f))
                        .background(backgroundColor, CircleShape)
                )
            }
            "offline", "invisible" -> {
                // Hollow ring for offline
                Box(
                    modifier = Modifier
                        .fillMaxSize(0.45f)
                        .background(backgroundColor, CircleShape)
                )
            }
        }
    }
}
