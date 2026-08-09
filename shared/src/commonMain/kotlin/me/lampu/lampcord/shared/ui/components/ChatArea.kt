package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background 
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
    
    val latestMessageId = chatState.messages.firstOrNull()?.id
    
    LaunchedEffect(latestMessageId) {
        if (latestMessageId != null) {
            // If we are at or near the bottom (item 0 in reverseLayout), scroll to new item
            if (scrollState.firstVisibleItemIndex <= 1) {
                scrollState.scrollToItem(0)
            }
        }
    }

    LaunchedEffect(chatState.scrollToMessageId) {
        chatState.scrollToMessageId?.let { messageId ->
            val index = chatState.messages.indexOfFirst { it.id == messageId }
            if (index != -1) {
                chatState.highlightedMessageId = messageId
                
                // Get viewport height and estimate item offset to center it
                val visibleItems = scrollState.layoutInfo.visibleItemsInfo
                val viewportHeight = scrollState.layoutInfo.viewportSize.height

                if (visibleItems.isNotEmpty()) {
                    // Try to calculate an offset that centers the item
                    // If we don't know the exact item height, we use an average or a safe estimate
                    val averageItemHeight = visibleItems.map { it.size }.average().toInt()
                    val centerOffset = (viewportHeight / 2) - (averageItemHeight / 2)

                    scrollState.animateScrollToItem(index, scrollOffset = -centerOffset)
                } else {
                    scrollState.animateScrollToItem(index)
                }
            }
            chatState.scrollToMessageId = null
        }
    }
    
    Box(
        modifier = modifier
            .fillMaxSize()
            .padding(bottom = 10.dp)
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
        val filteredMessages = remember(chatState.messages.size, chatState.relationshipStore.relationships.size) {
            val hideBlocked = me.lampu.lampcord.shared.settings.Settings.shared.hideBlockedMessages
            if (hideBlocked) {
                chatState.messages.filter { msg ->
                    val authorId = msg.author?.id
                    if (authorId == null) true
                    else chatState.relationshipStore.relationships.none { (it.id ?: it.user?.id ?: it.user_id) == authorId && it.type == 2 }
                }
            } else chatState.messages
        }

        LazyColumn(
            state = scrollState,
            modifier = Modifier.fillMaxSize(),
            reverseLayout = true
        ) {
            items(
                items = filteredMessages,
                key = { it.id }
            ) { message ->
                val index = filteredMessages.indexOf(message)
                val priorMessage = filteredMessages.getOrNull(index + 1)
                
                // Grouping logic: 7 minutes window, same author, current is regular message
                val isInline = remember(message, priorMessage) {
                    if (priorMessage == null) return@remember false
                    if (priorMessage.author?.id != message.author?.id) return@remember false
                    if (message.referenced_message != null) return@remember false
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
                            .size(74.dp)
                            .background(color = MaterialTheme.colorScheme.surfaceContainerHigh, shape = CircleShape),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            imageVector = when(chatState.selectedChannel?.type ?: 0) {
                                1, 3 -> Icons.Outlined.AlternateEmail
                                else -> Icons.Filled.Tag
                            },
                            contentDescription = "",
                            modifier = Modifier
                                        .width(46.dp)
                                        .height(54.dp),
                            tint = Color.White
                        )
                    }
                    Spacer(modifier = Modifier.height(14.dp))
                    Text(
                        text = when(chatState.selectedChannel?.type ?: 0) {
                            1, 3 -> "This is the start of your conversation."
                            else -> "Welcome to #${chatState.selectedChannel?.name ?: "null"}"
                        },
                        fontSize = 24.sp,
                        color = Color.White
                    )

                    if ((chatState.selectedChannel?.type == 2 || chatState.selectedChannel?.type == 13) && chatState.currentVoiceState?.channel_id != chatState.selectedChannel?.id) {
                        Spacer(modifier = Modifier.height(16.dp))
                        Button(
                            onClick = { chatState.selectedChannel?.let { chatState.connectToVoice(it) } },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary)
                        ) {
                            Icon(Icons.AutoMirrored.Filled.VolumeUp, null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Join Voice")
                        }
                    }
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

