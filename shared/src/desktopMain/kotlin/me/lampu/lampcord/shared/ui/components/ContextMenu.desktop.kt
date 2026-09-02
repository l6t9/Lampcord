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
import kotlinx.coroutines.withTimeoutOrNull

@Composable
actual fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier,
    shape: androidx.compose.ui.graphics.Shape,
    header: (@Composable () -> Unit)?,
    reactions: (@Composable (onDismiss: () -> Unit) -> Unit)?,
    enabled: Boolean,
    respectChildGestures: Boolean,
    openRequest: Int,
    content: @Composable () -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    var offset by remember { mutableStateOf(DpOffset.Zero) }
    val density = LocalDensity.current

    LaunchedEffect(openRequest) {
        if (openRequest > 0) expanded = true
    }

    Box(
        modifier = modifier
            .pointerInput(items, enabled) {
                if (!enabled) return@pointerInput
                awaitPointerEventScope {
                    while (true) {
                        // Let nested content (notably Markdown links) inspect
                        // secondary clicks before the message menu claims them.
                        val event = awaitPointerEvent(PointerEventPass.Main)
                        val down = event.changes.find { it.changedToDown() } ?: continue

                        if (event.buttons.isSecondaryPressed) {
                            if (down.isConsumed) continue
                            offset = with(density) { DpOffset(down.position.x.toDp(), down.position.y.toDp()) }
                            expanded = true
                            down.consume()
                            continue
                        }
                        if (down.type == PointerType.Mouse) continue

                        val longPressed = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                            while (true) {
                                val nextEvent = awaitPointerEvent(PointerEventPass.Main)
                                val change = nextEvent.changes.firstOrNull { it.id == down.id }
                                if (change == null || !change.pressed) return@withTimeoutOrNull false
                                if ((change.position - down.position).getDistance() > viewConfiguration.touchSlop) {
                                    return@withTimeoutOrNull false
                                }
                            }
                            false
                        } ?: true

                        if (!longPressed) continue

                        offset = with(density) { DpOffset(down.position.x.toDp(), down.position.y.toDp()) }
                        expanded = true
                        down.consume()
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
