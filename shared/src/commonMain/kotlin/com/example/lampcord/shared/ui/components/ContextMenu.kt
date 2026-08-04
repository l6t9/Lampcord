package com.example.lampcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.isSecondaryPressed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier = Modifier,
    shape: androidx.compose.ui.graphics.Shape = RoundedCornerShape(0.dp),
    content: @Composable () -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    var offset by remember { mutableStateOf(DpOffset.Zero) }
    val density = LocalDensity.current

    Box(
        modifier = modifier
            .pointerInput(items) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        // Capture position on Press, but wait for Release or just trigger on Press like Metrolist
                        if (event.type == PointerEventType.Press && event.buttons.isSecondaryPressed) {
                            val position = event.changes.first().position
                            offset = with(density) { DpOffset(position.x.toDp(), position.y.toDp()) }
                            expanded = true
                            // Consume to prevent other handlers
                            event.changes.forEach { it.consume() }
                        }
                    }
                }
            }
            .combinedClickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null, // Disable default highlight
                onClick = {},
                onLongClick = {
                    offset = DpOffset.Zero
                    expanded = true
                }
            )
            .clip(shape)
    ) {
        content()
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            offset = offset
        ) {
            items.forEach { item ->
                DropdownMenuItem(
                    text = { 
                        Text(
                            text = item.label,
                            color = item.color ?: MaterialTheme.colorScheme.onSurface
                        ) 
                    },
                    onClick = {
                        item.onClick()
                        expanded = false
                    },
                    leadingIcon = item.icon?.let { { 
                        Icon(
                            imageVector = it,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                            tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                        ) 
                    } }
                )
            }
        }
    }
}

data class ContextMenuItem(
    val label: String,
    val icon: ImageVector? = null,
    val color: Color? = null,
    val onClick: () -> Unit
)
