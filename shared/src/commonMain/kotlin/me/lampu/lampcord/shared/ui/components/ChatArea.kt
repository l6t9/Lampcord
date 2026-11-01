package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.join_voice
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.Surface
import androidx.compose.runtime.derivedStateOf
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.zIndex
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.ThemeStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.chat.MessageItem
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import kotlinx.serialization.json.JsonPrimitive
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant
import me.lampu.lampcord.shared.ui.kit.clickableCursor

private const val MIN_SCROLLBACK = 15

@Composable
fun ChatArea(
    modifier: Modifier = Modifier,
    messageStore: MessageStore = koinInject(),
    relationshipStore: RelationshipStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    themeStore: ThemeStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    readStateStore: me.lampu.lampcord.shared.state.ReadStateStore = koinInject()
) {
    val joinVoice = me.lampu.lampcord.shared.voice.rememberVoiceJoin(voiceStore)
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName() == "android" ||
        me.lampu.lampcord.shared.utils.getPlatformName() == "ios"
    val channelId = navigationStore.selectedThread?.id ?: navigationStore.selectedChannel?.id
    val scrollState = remember(channelId) { LazyListState() }
    var isHovered by remember { mutableStateOf(false) }
    val messages by messageStore.messages.collectAsState()
    val relationships by relationshipStore.relationships.collectAsState()
    val activeReadState by remember(channelId) {
        readStateStore.readStates.map { it[channelId] }.distinctUntilChanged()
    }.collectAsState(initial = readStateStore.readStates.value[channelId])
    val reduceMotion = Settings.shared.reduceMotion

    val themeBackgroundUrl = themeStore.themeBackgroundUrl ?: ""
    val themeBackgroundAlpha = themeStore.themeBackgroundAlpha

    val backgroundUrl = if (themeBackgroundUrl.isNotEmpty()) themeBackgroundUrl else settingsStore.chatBackground

    val ackedMessageId = if (channelId == null) {
        "0"
    } else {
        activeReadState?.last_message_id?.toString()?.removeSurrounding("\"") ?: "0"
    }

    val unreadInfo by remember(ackedMessageId) {
        derivedStateOf {
            val ackedLong = ackedMessageId.toLongOrNull() ?: 0L
            var count = 0
            var oldestUnreadId: String? = null
            for (message in messages) {
                if (message.isPending) continue
                if ((message.id.toLongOrNull() ?: 0L) > ackedLong) {
                    count++
                    oldestUnreadId = message.id
                } else {
                    break
                }
            }
            count to oldestUnreadId
        }
    }
    val unreadMessagesCount = unreadInfo.first
    val firstUnreadMessageId = unreadInfo.second

    val isScrolledFromBottom by remember(scrollState) {
        derivedStateOf { scrollState.firstVisibleItemIndex > 0 }
    }

    DisposableEffect(channelId) {
        if (channelId != null) {
            navigationStore.setChatInteraction(channelId, true)
        }
        onDispose {
            if (channelId != null) {
                navigationStore.clearChatInteraction(channelId)
            }
        }
    }

    LaunchedEffect(scrollState) {
        var ackJob: Job? = null
        snapshotFlow {
            val visibleItems = scrollState.layoutInfo.visibleItemsInfo
            val atBottom = scrollState.firstVisibleItemIndex == 0
            val lastVisibleItemIndex = visibleItems.lastOrNull()?.index

            atBottom to lastVisibleItemIndex
        }
            .distinctUntilChanged()
            .collect { (atBottom, lastVisibleItemIndex) ->
                if (lastVisibleItemIndex != null) {

                    if (channelId != null) {
                        navigationStore.setChatInteraction(channelId, atBottom)
                    }
                    if (atBottom && messages.isNotEmpty()) {
                        val latestId = messages.first().id
                        if ((latestId.toLongOrNull() ?: 0L) > (ackedMessageId.toLongOrNull() ?: 0L)) {
                            ackJob?.cancel()
                            ackJob = launch {
                                delay(1500)
                                readStateStore.ackMessage(channelId ?: "", latestId)
                            }
                        }
                    }
                    if (channelId != null && lastVisibleItemIndex >= messages.size - 5) {
                        messageStore.loadMoreMessages(
                            channelId,
                            navigationStore.selectedGuild?.id,
                            navigationStore.selectedThread?.id
                        )
                    }
                }
            }
    }

    LaunchedEffect(channelId) {
        if (channelId != null && messages.isEmpty()) {
            messageStore.loadMoreMessages(
                channelId,
                navigationStore.selectedGuild?.id,
                navigationStore.selectedThread?.id
            )
        }
    }

    val latestMessageId = messages.firstOrNull()?.id
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(latestMessageId) {
        if (latestMessageId != null) {
            if (scrollState.firstVisibleItemIndex <= 1) {
                scrollState.scrollToItem(0)
            }
        }
    }

    LaunchedEffect(messageStore.scrollToMessageId) {
        val messageId = messageStore.scrollToMessageId ?: return@LaunchedEffect
        val index = withTimeoutOrNull(5_000) {
            snapshotFlow { messages.indexOfFirst { it.id == messageId } }
                .first { it != -1 }
        } ?: -1

        if (index != -1) {
            messageStore.highlightedMessageId = messageId

            val visibleItems = scrollState.layoutInfo.visibleItemsInfo
            val viewportHeight = scrollState.layoutInfo.viewportSize.height

            if (visibleItems.isNotEmpty()) {
                val averageItemHeight = visibleItems.map { it.size }.average().toInt()
                val centerOffset = (viewportHeight / 2) - (averageItemHeight / 2)

                if (reduceMotion) {
                    scrollState.scrollToItem(index, scrollOffset = -centerOffset)
                } else {
                    scrollState.animateScrollToItem(index, scrollOffset = -centerOffset)
                }
            } else {
                if (reduceMotion) scrollState.scrollToItem(index) else scrollState.animateScrollToItem(index)
            }
        }
        messageStore.scrollToMessageId = null
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
        if (backgroundUrl.isNotEmpty()) {
            AsyncImage(
                model = backgroundUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop
            )
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.surface.copy(alpha = if (themeBackgroundUrl.isNotEmpty()) themeBackgroundAlpha else 0.6f))
            )
        }

        val blockedAuthorIds = remember(relationships) {
            relationships.asSequence()
                .filter { it.type == 2 }
                .mapNotNull { it.id ?: it.user?.id ?: it.user_id }
                .toSet()
        }

        val hideBlockedMessages = me.lampu.lampcord.shared.settings.Settings.shared.hideBlockedMessages
        val filteredMessages = remember(messages, blockedAuthorIds, hideBlockedMessages) {
            if (hideBlockedMessages) {
                messages.filter { msg -> msg.author?.id?.let { it !in blockedAuthorIds } ?: true }
            } else messages
        }

        LazyColumn(
            state = scrollState,
            modifier = Modifier
                .fillMaxSize(),
            reverseLayout = true
        ) {
            itemsIndexed(
                items = filteredMessages,
                key = { _, message -> message.id },
                contentType = { _, message -> message.type ?: 0 }
            ) { index, message ->
                val priorMessage = filteredMessages.getOrNull(index + 1)
                val nextMessage = filteredMessages.getOrNull(index - 1)

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

                val isInline = remember(message, priorMessage, showDateSeparator) {
                    if (priorMessage == null) return@remember false
                    if (showDateSeparator) return@remember false
                    if (priorMessage.author?.id != message.author?.id) return@remember false
                    if (message.referenced_message != null) return@remember false
                    val currentType = message.type ?: 0
                    val priorType = priorMessage.type ?: 0
                    if (currentType != 0 && currentType != 19) return@remember false
                    if (priorType != 0 && priorType != 19) return@remember false

                    try {
                        val currentTs = Instant.parse(message.timestamp)
                        val priorTs = Instant.parse(priorMessage.timestamp)
                        (currentTs - priorTs) < 7.minutes
                    } catch (e: Exception) {
                        false
                    }
                }

                val isFirstUnread = message.id == firstUnreadMessageId

                Column(modifier = Modifier.fillMaxWidth()) {
                    if (showDateSeparator) {
                        DateSeparator(message.timestamp)
                    }
                    if (isFirstUnread) {
                        UnreadSeparator()
                    }
                    MessageItem(
                        message = message,
                        priorMessage = if (isInline) priorMessage else null,
                        nextMessage = nextMessage
                    )
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
                            imageVector = when(navigationStore.selectedChannel?.type ?: 0) {
                                1, 3 -> Icons.Rounded.AlternateEmail
                                else -> Icons.Filled.Tag
                            },
                            contentDescription = "",
                            modifier = Modifier
                                        .width(46.dp)
                                        .height(54.dp),
                            tint = MaterialTheme.colorScheme.onSurface
                        )
                    }
                    Spacer(modifier = Modifier.height(14.dp))
                    Text(
                        text = when(navigationStore.selectedChannel?.type ?: 0) {
                            1, 3 -> "This is the start of your conversation."
                            else -> "Welcome to #${navigationStore.selectedChannel?.name ?: "null"}"
                        },
                        fontSize = 32.sp,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = when(navigationStore.selectedChannel?.type ?: 0) {
                            1, 3 -> "This is the very beginning of your direct message history."
                            else -> "This is the start of the #${navigationStore.selectedChannel?.name ?: "null"} channel."
                        },
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                    )

                    if (navigationStore.selectedChannel?.type == 2 && voiceStore.activeChannel?.id != navigationStore.selectedChannel?.id) {
                        Spacer(modifier = Modifier.height(16.dp))
                        Button(
                            onClick = { navigationStore.selectedChannel?.let { joinVoice(it, false) } },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary)
                        ) {
                            Icon(Icons.AutoMirrored.Filled.VolumeUp, null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(Res.string.join_voice))
                        }
                    }
                }
            }
        }
        VerticalScrollbar(
            state = scrollState,
            modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
            isVisible = isHovered || isMobile,
            reverseLayout = true
        )
        if (unreadMessagesCount > 0 && isScrolledFromBottom) {
            Surface(
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .zIndex(2f)
                    .padding(top = 12.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .clickableCursor {
                        coroutineScope.launch {
                            if (reduceMotion) scrollState.scrollToItem(0) else scrollState.animateScrollToItem(0)
                        }
                    },
                shape = RoundedCornerShape(12.dp),
                color = MaterialTheme.colorScheme.primary,
                tonalElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Filled.ArrowDownward,
                        contentDescription = null,
                        modifier = Modifier.size(14.dp),
                        tint = MaterialTheme.colorScheme.onPrimary
                    )
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = "$unreadMessagesCount NEW MESSAGE${if (unreadMessagesCount > 1) "S" else ""}",
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontWeight = FontWeight.Black,
                            letterSpacing = 0.5.sp,
                            fontSize = 10.sp
                        ),
                        color = MaterialTheme.colorScheme.onPrimary,
                        maxLines = 1
                    )
                }
            }
        }

        val scrolledAway = remember(scrollState) { derivedStateOf {
            scrollState.firstVisibleItemIndex > MIN_SCROLLBACK
        } }

        if (scrolledAway.value) {
            androidx.compose.material3.FloatingActionButton(
                onClick = {
                    coroutineScope.launch {
                        if (reduceMotion) scrollState.scrollToItem(0) else scrollState.animateScrollToItem(0)
                    }
                },
                modifier = Modifier.align(Alignment.BottomEnd).padding(16.dp)
            ) {
                Icon(Icons.Filled.ArrowDownward, null)
            }
        }
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
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
        Text(
            text = dateText,
            modifier = Modifier.padding(horizontal = 8.dp),
            style = MaterialTheme.typography.labelSmall.copy(
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 0.5.sp
            ),
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
        )
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
    }
}

@Composable
fun UnreadSeparator() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp, horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.error)
        Text(
            text = "NEW MESSAGES",
            modifier = Modifier.padding(horizontal = 8.dp),
            style = MaterialTheme.typography.labelSmall.copy(
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold
            ),
            color = MaterialTheme.colorScheme.error
        )
        HorizontalDivider(modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.error)
    }
}
