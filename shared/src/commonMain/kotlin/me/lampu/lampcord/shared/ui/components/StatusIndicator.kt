package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.StatusIcons

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun StatusIndicator(
    status: String,
    modifier: Modifier = Modifier,
    size: Dp = 14.dp,
    borderColor: Color = Color.Transparent,
    borderWidth: Dp? = null,
) {
    val displayStatus = when (status) {
        "online" -> "Online"
        "idle" -> "Idle"
        "dnd" -> "DND"
        "streaming" -> "Streaming"
        "mobile" -> "Online"
        else -> "Offline"
    }

    val effectiveBorderWidth = borderWidth ?: size * 0.18f
    val isMobile = status == "mobile"
    val shape = if (isMobile) RoundedCornerShape(size * 0.2f) else CircleShape
    val boxHeight = if (isMobile) size * (12f / 8f) else size

    Box(
        modifier = modifier
            .size(width = size, height = boxHeight)
            .clip(shape)
            .background(borderColor)
            .padding(effectiveBorderWidth)
    ) {
        ExpressiveTooltip(
            anchorPosition = TooltipAnchorPosition.Above,
            content = tooltipText(displayStatus),
            anchor = {
                Image(
                    painter = rememberVectorPainter(StatusIcons.fromStatus(status)),
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Fit
                )
            }
        )
    }
}
