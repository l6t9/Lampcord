package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.members.MemberGroupItem
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberSkeleton
import org.koin.compose.koinInject

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberList(
    navigationStore: NavigationStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    header: @Composable (() -> Unit)? = null
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    // Scroll to top when channel changes
    LaunchedEffect(navigationStore.selectedChannel?.id) {
        scrollState.scrollToItem(0)
    }

    // Scrolling range logic: keep current and surrounding blocks subscribed.
    val firstVisible = scrollState.firstVisibleItemIndex
    val rowCount = memberListStore.memberListRowCount
    val ranges = remember(firstVisible, rowCount) {
        val currentBlock = (firstVisible / 100) * 100
        val blocks = mutableSetOf(0)
        
        blocks.add(currentBlock)
        if (currentBlock >= 100) blocks.add(currentBlock - 100)
        blocks.add(currentBlock + 100)
        blocks.add(currentBlock + 200)
        
        val filtered = blocks.filter { it < rowCount }.sorted()
        if (filtered.isEmpty()) {
            listOf(listOf(0, 99))
        } else {
            filtered.map { listOf(it, it + 99) }
        }
    }

    val channelId = navigationStore.selectedChannel?.id
    var lastRequestedChannelId by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(ranges, channelId) {
        if (channelId != null) {
            if (channelId != lastRequestedChannelId) {
                memberListStore.requestMemberListRange(ranges)
                lastRequestedChannelId = channelId
            } else {
                kotlinx.coroutines.delay(300)
                memberListStore.requestMemberListRange(ranges)
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

                    if (memberListStore.memberListRowCount == 0) {
                        items(20) {
                            MemberSkeleton()
                        }
                    } else {
                        items(
                            count = memberListStore.memberListRowCount,
                            key = { index -> 
                                val item = memberListStore.memberListItems[index]
                                val baseId = item?.member?.userId() ?: item?.group?.id ?: "null"
                                // 126.21 Parity: Discord member lists are index-based.
                                // We include the index in the key to prevent crashes if the state is temporarily inconsistent
                                // (e.g. during a channel switch or rapid gateway updates).
                                "$index-$baseId"
                            }
                        ) { index ->
                            val item = memberListStore.memberListItems[index]
                            when {
                                item?.member != null -> MemberItem(item.member)
                                item?.group != null -> MemberGroupItem(item.group)
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
