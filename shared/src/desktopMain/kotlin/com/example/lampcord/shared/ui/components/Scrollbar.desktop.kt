package com.example.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.VerticalScrollbar
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.rememberScrollbarAdapter
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp

@Composable
actual fun VerticalScrollbar(
    state: LazyListState,
    modifier: Modifier,
    isVisible: Boolean,
    reverseLayout: Boolean
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val isPressed by interactionSource.collectIsPressedAsState()
    val isDragged by interactionSource.collectIsDraggedAsState()
    
    val isActive = isHovered || isPressed || isDragged
    val thickness by animateDpAsState(if (isActive) 8.dp else 4.dp)

    AnimatedVisibility(
        visible = isVisible || isActive,
        enter = fadeIn(),
        exit = fadeOut(),
        modifier = modifier
    ) {
        Box(modifier = Modifier.fillMaxHeight()) {
            VerticalScrollbar(
                adapter = rememberScrollbarAdapter(state),
                modifier = Modifier
                    .align(Alignment.CenterEnd)
                    .width(thickness)
                    .fillMaxHeight(0.8f)
                    .then(
                        if (reverseLayout) {
                            Modifier.graphicsLayer(scaleY = -1f)
                        } else Modifier
                    ),
                interactionSource = interactionSource,
                style = androidx.compose.foundation.defaultScrollbarStyle().copy(
                    unhoverColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.12f),
                    hoverColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f),
                    thickness = thickness,
                    shape = CircleShape
                )
            )
        }
    }
}
