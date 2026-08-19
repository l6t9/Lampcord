package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
actual fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier,
    shape: androidx.compose.ui.graphics.Shape,
    header: (@Composable () -> Unit)?,
    reactions: (@Composable (onDismiss: () -> Unit) -> Unit)?,
    content: @Composable () -> Unit
) {
    var showSheet by remember { mutableStateOf(false) }

    Box(
        modifier = modifier
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
        AdaptiveModalBottomSheet(
            onDismissRequest = { showSheet = false },
            containerColor = MaterialTheme.colorScheme.surface
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .navigationBarsPadding()
                    .padding(bottom = 32.dp),
            ) {
                if (header != null) {
                    Box(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                        header()
                    }
                }
                
                if (reactions != null) {
                    Box(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                        reactions { showSheet = false }
                    }
                    Spacer(Modifier.height(8.dp))
                }

                val groups = linkedMapOf<String?, MutableList<ContextMenuItem>>()
                items.forEach { it ->
                    val key = it.group
                    if (!groups.containsKey(key)) groups[key] = mutableListOf()
                    groups[key]!!.add(it)
                }

                groups.entries.forEachIndexed { idx, entry ->
                    val groupItems = entry.value
                    Material3SettingsGroup(
                        items = groupItems.map { item ->
                            Material3SettingsItem(
                                title = { Text(item.label) },
                                icon = item.icon,
                                iconTint = item.color,
                                onClick = {
                                    item.onClick()
                                    showSheet = false
                                }
                            )
                        }
                    )
                    if (idx < groups.size - 1) Spacer(Modifier.height(8.dp))
                }
            }
        }
    }
}
