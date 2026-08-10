package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.MemberListGroup
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.members.MemberGroupItem
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberSkeleton
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberList(
    chatState: ChatState,
    header: @Composable (() -> Unit)? = null
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    // Scroll to top when channel changes
    LaunchedEffect(chatState.selectedChannel?.id) {
        scrollState.scrollToItem(0)
    }

    // Scrolling range logic: keep current and surrounding blocks subscribed.
    val firstVisible = scrollState.firstVisibleItemIndex
    val rowCount = chatState.memberListRowCount
    val ranges = remember(firstVisible, rowCount) {
        val currentBlock = (firstVisible / 100) * 100
        val blocks = mutableSetOf(0) // Always keep top members
        
        blocks.add(currentBlock)
        if (currentBlock >= 100) blocks.add(currentBlock - 100)
        blocks.add(currentBlock + 100)
        blocks.add(currentBlock + 200)
        
        blocks.filter { it < rowCount }.sorted().map { listOf(it, it + 99) }
    }

    // 126.21 Parity: Initial request is immediate, subsequent scrolls are debounced.
    val channelId = chatState.selectedChannel?.id
    var lastRequestedChannelId by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(ranges, channelId) {
        if (ranges.isNotEmpty() && channelId != null) {
            if (channelId != lastRequestedChannelId) {
                // Immediate request for new channel selection
                chatState.requestMemberListRange(ranges)
                lastRequestedChannelId = channelId
            } else {
                // Debounce for scrolling
                kotlinx.coroutines.delay(300)
                chatState.requestMemberListRange(ranges)
            }
        }
    }

    Column(
        modifier = Modifier
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
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surface,
            shape = RoundedCornerShape(0.dp),
            tonalElevation = 1.dp
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                LazyColumn(
                    state = scrollState,
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(bottom = 52.dp)
                ) {
                    if (header != null) {
                        stickyHeader {
                            Surface(
                                color = MaterialTheme.colorScheme.surface,
                                tonalElevation = 1.dp
                            ) {
                                header()
                            }
                        }
                    } else {
                        // Spacing at top if no header
                        item {
                            Spacer(Modifier.height(8.dp))
                        }
                    }

                    if (chatState.memberListRowCount == 0) {
                        items(20) {
                            MemberSkeleton()
                        }
                    } else {
                        items(
                            count = chatState.memberListRowCount,
                            key = { index -> 
                                val item = chatState.memberListItems[index]
                                val baseId = item?.member?.userId() ?: item?.group?.id ?: "null"
                                // 126.21 Parity: Discord member lists are index-based.
                                // We include the index in the key to prevent crashes if the state is temporarily inconsistent
                                // (e.g. during a channel switch or rapid gateway updates).
                                "$index-$baseId"
                            }
                        ) { index ->
                            val item = chatState.memberListItems[index]
                            when {
                                item?.member != null -> MemberItem(item.member, chatState)
                                item?.group != null -> MemberGroupItem(item.group, chatState)
                                else -> {
                                    MemberSkeleton()
                                }
                            }
                        }
                    }
                }

                VerticalScrollbar(
                    state = scrollState,
                    modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
                    isVisible = isHovered
                )
            }
        }
    }
}

