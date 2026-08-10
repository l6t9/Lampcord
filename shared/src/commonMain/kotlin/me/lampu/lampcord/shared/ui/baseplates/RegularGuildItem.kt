package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

@Composable
fun RegularGuildItem(
    isSelected: Boolean,
    isUnread: Boolean = false,
    isMuted: Boolean = false,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    selectedColor: Color = Color.Transparent,
    unselectedColor: Color = Color.Transparent,
    content: @Composable BoxScope.() -> Unit
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()

    val showHoverIndicator = isHovered && !isSelected

    val indicatorFraction by animateFloatAsState(
        targetValue = when {
            isSelected -> 0.8f
            showHoverIndicator -> 0.4f
            isUnread && !isMuted -> 0.15f
            else -> 0f
        },
        label = "indicatorFraction"
    )
    
    val indicatorAlpha by animateFloatAsState(
        targetValue = if (isSelected || (isUnread && !isMuted) || showHoverIndicator) 1f else 0f,
        label = "indicatorAlpha"
    )

    val imageCornerRadius by animateDpAsState(if (isSelected || isHovered) 12.dp else 24.dp)
    val backgroundColor by animateColorAsState(if (isSelected) selectedColor else unselectedColor)
    
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(48.dp)
            .hoverable(interactionSource)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick
            ),
    ) {
        // Indicator
        Box(
            modifier = Modifier
                .align(Alignment.CenterStart)
                .width(4.dp)
                .fillMaxHeight(indicatorFraction)
                .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                .background(MaterialTheme.colorScheme.onSurface)
                .alpha(indicatorAlpha)
        )

        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .size(48.dp)
                .clip(RoundedCornerShape(imageCornerRadius))
                .background(backgroundColor),
            contentAlignment = Alignment.Center
        ) {
            content()
        }
    }
}
