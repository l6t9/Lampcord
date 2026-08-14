package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.StatusIcons

@Composable
fun StatusIndicator(
    status: String,
    modifier: Modifier = Modifier,
    size: Dp = 14.dp,
    borderColor: Color = Color.Transparent,
    borderWidth: Dp? = null,
) {
    // Discord's StatusView draws a background circle (rounded rect for mobile)
    // in the app surface color, then insets the status glyph by a border,
    // which creates the "cutout" notch around the avatar.
    // The glyph stays a fixed fraction of the bubble, so the border must be
    // proportional to the size (a fixed dp would swallow the glyph on big
    // profile bubbles). 0.18 keeps the 2dp look on the 11.2dp member/DM bubble.
    // The mobile phone is 8x12, so its bubble is portrait, not square.
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
        Image(
            painter = rememberVectorPainter(StatusIcons.fromStatus(status)),
            contentDescription = null,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Fit
        )
    }
}
