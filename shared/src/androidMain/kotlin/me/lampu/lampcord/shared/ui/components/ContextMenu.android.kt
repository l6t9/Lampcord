package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
actual fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier,
    shape: androidx.compose.ui.graphics.Shape,
    content: @Composable () -> Unit
) {
    var showSheet by remember { mutableStateOf(false) }

    Box(
        modifier = modifier
            .clip(shape)
            .pointerInput(items) {
                awaitPointerEventScope {
                    while (true) {
                        val down = awaitFirstDown(pass = PointerEventPass.Initial)
                        val timedOut = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                            waitForUpOrCancellation(pass = PointerEventPass.Initial)
                            false
                        } ?: true
                        
                        if (timedOut) {
                            showSheet = true
                            down.consume()
                            // Consume all subsequent events until all pointers are up
                            while (true) {
                                val event = awaitPointerEvent(PointerEventPass.Initial)
                                event.changes.forEach { it.consume() }
                                if (event.changes.all { !it.pressed }) break
                            }
                        }
                    }
                }
            }
    ) {
        content()
    }

    if (showSheet) {
        ModalBottomSheet(
            onDismissRequest = { showSheet = false },
            containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
            dragHandle = {
                Box(
                    modifier = Modifier
                        .padding(vertical = 12.dp)
                        .size(width = 40.dp, height = 4.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
                )
            }
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .navigationBarsPadding()
                    .padding(bottom = 8.dp)
            ) {
                items.forEach { item ->
                    ListItem(
                        headlineContent = {
                            Text(
                                text = item.label,
                                color = item.color ?: MaterialTheme.colorScheme.onSurface
                            )
                        },
                        leadingContent = item.icon?.let {
                            {
                                Icon(
                                    imageVector = it,
                                    contentDescription = null,
                                    tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        },
                        modifier = Modifier.clickable {
                            item.onClick()
                            showSheet = false
                        }
                    )
                }
            }
        }
    }
}
