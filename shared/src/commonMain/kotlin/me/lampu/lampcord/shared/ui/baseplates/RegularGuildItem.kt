package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.animateIntAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

@Composable
fun RegularGuildItem(
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    selectedColor: Color = Color.Transparent,
    unselectedColor: Color = Color.Transparent,
    content: @Composable BoxScope.() -> Unit
) {
    val indicatorFraction by animateFloatAsState(if (isSelected) 0.8f else 0.15f)
    val imageCornerRadius by animateIntAsState(if (isSelected) 25 else 50)
    val backgroundColor by animateColorAsState(if (isSelected) selectedColor else unselectedColor)
    
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(48.dp),
    ) {
        // Indicator
        val indicatorColor = MaterialTheme.colorScheme.onSurface
        Box(
            modifier = Modifier
                .align(Alignment.CenterStart)
                .width(4.dp)
                .fillMaxHeight(indicatorFraction)
                .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                .background(indicatorColor)
                .alpha(if (isSelected) 1f else 0f)
        )

        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .size(48.dp)
                .clip(RoundedCornerShape(imageCornerRadius))
                .background(backgroundColor)
                .clickable(onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            content()
        }
    }
}
