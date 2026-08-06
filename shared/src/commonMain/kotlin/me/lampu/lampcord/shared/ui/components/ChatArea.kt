package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background 
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape 
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color 
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.chat.MessageItem
import me.lampu.lampcord.shared.utils.DateTimeUtils
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock
import kotlin.time.Instant
import kotlin.time.Duration.Companion.minutes
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun ChatArea(
    modifier: Modifier = Modifier,
    chatState: ChatState,
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    LaunchedEffect(scrollState) {
        snapshotFlow { scrollState.layoutInfo.visibleItemsInfo }
            .collect { visibleItems ->
                if (visibleItems.isNotEmpty()) {
                    val lastVisibleItem = visibleItems.last()
                    if (lastVisibleItem.index >= chatState.messages.size - 5) {
                        chatState.loadMoreMessages()
                    }
                }
            }
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        when (event.type) {
                            PointerEventType.Enter -> isHovered = true
                            PointerEventType.Exit -> isHovered = false
                        }
                    }
                }
            }
    ) {
        LazyColumn(
            state = scrollState,
            modifier = Modifier.fillMaxSize(),
            reverseLayout = true
        ) {
            items(
                count = chatState.messages.size,
                key = { index -> chatState.messages[index].id }
            ) { index ->
                val message = chatState.messages[index]
                val priorMessage = chatState.messages.getOrNull(index + 1)
                
                // Date separator logic
                val showDateSeparator = remember(message, priorMessage) {
                    if (priorMessage == null) return@remember true
                    try {
                        val currentTs = Instant.parse(message.timestamp).toLocalDateTime(TimeZone.currentSystemDefault()).date
                        val priorTs = Instant.parse(priorMessage.timestamp).toLocalDateTime(TimeZone.currentSystemDefault()).date
                        currentTs != priorTs
                    } catch (e: Exception) {
                        false
                    }
                }

                // Grouping logic: 7 minutes window, same author, current is regular message
                val isInline = remember(message, priorMessage) {
                    if (priorMessage == null) return@remember false
                    if (priorMessage.author.id != message.author.id) return@remember false
                    if (message.referenced_message != null) return@remember false
                    if (showDateSeparator) return@remember false
                    // Only regular messages (type 0) can be grouped inline.
                    // They can group under other regular messages (0) or replies (19).
                    val currentType = message.type ?: 0
                    val priorType = priorMessage.type ?: 0
                    if (currentType != 0) return@remember false
                    if (priorType != 0 && priorType != 19) return@remember false
                    
                    try {
                        val currentTs = Instant.parse(message.timestamp)
                        val priorTs = Instant.parse(priorMessage.timestamp)
                        (currentTs - priorTs) < 7.minutes
                    } catch (e: Exception) {
                        false
                    }
                }

                Column {
                    Box(Modifier.animateItem()) {
                        MessageItem(message, chatState, if (isInline) priorMessage else null)
                    }
                    
                    if (showDateSeparator) {
                        DateSeparator(message.timestamp)
                    }
                }
            }
            item { 
                Spacer(modifier = Modifier.height(16.dp)) 
            }
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    horizontalAlignment = Alignment.Start
                ) {
                    Box(
                        modifier = Modifier
                            .size(72.dp)
                            .background(color = MaterialTheme.colorScheme.surfaceContainerHigh, shape = CircleShape),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            imageVector = Icons.Filled.Tag,
                            contentDescription = "",
                            modifier = Modifier.size(46.dp),
                            tint = Color.White
                        )
                    }
                    Spacer(modifier = Modifier.height(12.dp))
                    Text(
                        text = "Welcome to #channel",
                        fontSize = 24.sp,
                        color = Color.White
                    )
                    println(chatState)
                }
            }
        }
        VerticalScrollbar(
            state = scrollState,
            modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
            isVisible = isHovered,
            reverseLayout = true
        )
    }
}

@Composable
fun DateSeparator(timestamp: String) {
    val dateText = remember(timestamp) {
        try {
            val instant = Instant.parse(timestamp)
            val localDate = instant.toLocalDateTime(TimeZone.currentSystemDefault()).date
            val month = localDate.month.name.lowercase().replaceFirstChar { it.uppercase() }
            "$month ${localDate.day}, ${localDate.year}"
        } catch (e: Exception) {
            ""
        }
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 16.dp, horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))
        Text(
            text = dateText,
            modifier = Modifier.padding(horizontal = 8.dp),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.Bold
        )
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))
    }
}

