package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.*
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp

@Composable
actual fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier,
    shape: androidx.compose.ui.graphics.Shape,
    header: (@Composable () -> Unit)?,
    reactions: (@Composable (onDismiss: () -> Unit) -> Unit)?,
    enabled: Boolean,
    content: @Composable () -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    var offset by remember { mutableStateOf(DpOffset.Zero) }
    val density = LocalDensity.current

    Box(
        modifier = modifier
            .pointerInput(items, enabled) {
                if (!enabled) return@pointerInput
                awaitPointerEventScope {
                    while (true) {
                        // Let nested content handle the Main pass first. A
                        // link consumes its own click; non-link message text
                        // remains available for this menu on the Final pass.
                        val event = awaitPointerEvent(PointerEventPass.Final)
                        val down = event.changes.find { it.changedToDown() && !it.isConsumed }
                        
                        if (down != null) {
                            if (event.buttons.isSecondaryPressed) {
                                offset = with(density) { DpOffset(down.position.x.toDp(), down.position.y.toDp()) }
                                expanded = true
                                event.changes.forEach { it.consume() }
                            } else if (down.type != PointerType.Mouse) {
                                val timedOut = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                                    var upOrCanceled = false
                                    while (!upOrCanceled) {
                                        val nextEvent = awaitPointerEvent()
                                        if (nextEvent.changes.any { it.changedToUp() || it.isConsumed }) {
                                            upOrCanceled = true
                                        }
                                    }
                                    false
                                } ?: true
                                
                                if (timedOut) {
                                    offset = with(density) { DpOffset(down.position.x.toDp(), down.position.y.toDp()) }
                                    expanded = true
                                }
                            }
                        }
                    }
                }
            }
            .clip(shape)
    ) {
        content()
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            offset = offset,
            shape = RoundedCornerShape(16.dp)
        ) {
            val groups = linkedMapOf<String?, MutableList<ContextMenuItem>>()
            items.forEach { it ->
                val key = it.group
                if (!groups.containsKey(key)) groups[key] = mutableListOf()
                groups[key]!!.add(it)
            }

            groups.entries.forEachIndexed { gIdx, entry ->
                val groupItems = entry.value

                groupItems.forEach { item ->
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
                        leadingIcon = item.icon?.let {
                            {
                                Icon(
                                    imageVector = it,
                                    contentDescription = null,
                                    modifier = Modifier.size(18.dp),
                                    tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    )
                }

                if (gIdx < groups.size - 1) HorizontalDivider()
            }
        }
    }
}
