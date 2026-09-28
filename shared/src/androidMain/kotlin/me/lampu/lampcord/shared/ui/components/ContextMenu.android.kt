package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
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
    var showSheet by remember { mutableStateOf(false) }
    val haptic = LocalHapticFeedback.current

    LaunchedEffect(openRequest) {
        if (openRequest > 0) showSheet = true
    }

    Box(
        modifier = modifier.pointerInput(items, enabled, respectChildGestures) {
            if (!enabled) return@pointerInput
            awaitPointerEventScope {
                while (true) {
                    val pass = if (respectChildGestures) PointerEventPass.Final else PointerEventPass.Initial
                    val down = awaitFirstDown(pass = pass, requireUnconsumed = false)
                    if (respectChildGestures && down.isConsumed) continue

                    val longPressTriggered = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                        while (true) {
                            val event = awaitPointerEvent(pass)
                            val change = event.changes.firstOrNull { it.id == down.id }
                            if (change == null || !change.pressed || respectChildGestures && change.isConsumed) {
                                return@withTimeoutOrNull false
                            }
                            if ((change.position - down.position).getDistance() > viewConfiguration.touchSlop) {
                                return@withTimeoutOrNull false
                            }
                        }
                    } == null

                    if (longPressTriggered) {
                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                        showSheet = true
                        
                        while (true) {
                            val event = awaitPointerEvent(PointerEventPass.Initial)
                            val change = event.changes.firstOrNull { it.id == down.id }
                            if (change != null) {
                                change.consume()
                                if (!change.pressed) break
                            } else {
                                break
                            }
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
            containerColor = MaterialTheme.colorScheme.surface,
            dragHandle = { BottomSheetDefaults.DragHandle() }
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .navigationBarsPadding()
                    .padding(bottom = 32.dp),
            ) {
                if (header != null) {
                    Surface(
                        modifier = Modifier
                            .padding(horizontal = 16.dp, vertical = 8.dp)
                            .fillMaxWidth(),
                        color = MaterialTheme.colorScheme.surfaceContainer,
                        shape = RoundedCornerShape(20.dp)
                    ) {
                        Box(modifier = Modifier.padding(16.dp)) {
                            header()
                        }
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

                val cornerRadius = 20.dp
                val reducedRadius = 5.dp

                groups.entries.forEachIndexed { idx, entry ->
                    val groupItems = entry.value
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        groupItems.forEachIndexed { itemIdx, item ->
                            val isFirst = itemIdx == 0
                            val isLast = itemIdx == groupItems.lastIndex
                            val itemShape = RoundedCornerShape(
                                topStart = if (isFirst) cornerRadius else reducedRadius,
                                topEnd = if (isFirst) cornerRadius else reducedRadius,
                                bottomStart = if (isLast) cornerRadius else reducedRadius,
                                bottomEnd = if (isLast) cornerRadius else reducedRadius,
                            )

                            Surface(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(itemShape)
                                    .clickableCursor {
                                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                        item.onClick()
                                        showSheet = false
                                    },
                                shape = itemShape,
                                color = MaterialTheme.colorScheme.surfaceContainer
                            ) {
                                ListItem(
                                    headlineContent = { Text(item.label, color = item.color ?: Color.Unspecified, style = MaterialTheme.typography.bodyLarge) },
                                    leadingContent = item.icon?.let { { Icon(it, null, modifier = Modifier.size(22.dp), tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant) } },
                                    colors = ListItemDefaults.colors(containerColor = Color.Transparent)
                                )
                            }
                        }
                    }
                    if (idx < groups.size - 1) Spacer(Modifier.height(16.dp))
                }
            }
        }
    }
}
