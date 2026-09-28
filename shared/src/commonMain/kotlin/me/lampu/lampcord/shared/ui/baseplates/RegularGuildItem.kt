package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@Composable
fun RegularGuildItem(
    isSelected: Boolean,
    isUnread: Boolean = false,
    isMuted: Boolean = false,
    isMonogram: Boolean = false,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    selectedColor: Color = Color.Transparent,
    unselectedColor: Color = Color.Transparent,
    monogramSelectedColor: Color = Color.Transparent,   // These monogram color variables are also responsible for the colors of the DM/home icon in the top left.
    monogramUnselectedColor: Color = Color.Transparent,
    content: @Composable BoxScope.() -> Unit,
) {
    val interactionSource: MutableInteractionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()

    val showHoverIndicator = isHovered && !isSelected
    val reduceMotion = Settings.shared.reduceMotion

    val indicatorFraction by animateFloatAsState(
        targetValue = when {
            isSelected -> 0.8f
            showHoverIndicator -> 0.4f
            isUnread && !isMuted -> 0.15f
            else -> 0f
        },
        label = "indicatorFraction",
        animationSpec = if (reduceMotion) snap() else spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessLow)
    )
    
    val indicatorAlpha by animateFloatAsState(
        targetValue = if (isSelected || (isUnread && !isMuted) || showHoverIndicator) 1f else 0f,
        label = "indicatorAlpha",
        animationSpec = if (reduceMotion) snap() else spring()
    )
    
    val backgroundColor by animateColorAsState(
        if (isSelected || isHovered) selectedColor else unselectedColor,
        label = "backgroundColor",
        animationSpec = if (reduceMotion) snap() else spring()
    )

    // Foreground color is for guild items that don't have a set icon, like the home/DMs button, servers without icons, etc.
    val monogramColor by animateColorAsState(
        if (isSelected || isHovered) monogramSelectedColor else monogramUnselectedColor,
        label = "monogramColor",
        animationSpec = if (reduceMotion) snap() else spring()
    )
    
    val cornerRadius by animateFloatAsState(
        targetValue = if (isSelected || isHovered) 16f else 24f,
        label = "cornerRadius",
        animationSpec = if (reduceMotion) snap() else spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessLow)
    )
    val shape = RoundedCornerShape(cornerRadius.dp)
    
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(48.dp)
            .hoverable(interactionSource)
            .clickableCursor(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick
            ),
        contentAlignment = Alignment.Center
    ) {
        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            // Indicator - Fixed to avoid clipping on mobile
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
                    .size(48.dp)
                    .clip(shape)
                    .background(backgroundColor),
                contentAlignment = Alignment.Center
            ) {
                if (isMonogram) {
                    CompositionLocalProvider(LocalContentColor provides monogramColor) {
                        content()
                    }
                } else {
                    content()
                }
            }
        }
    }
}
