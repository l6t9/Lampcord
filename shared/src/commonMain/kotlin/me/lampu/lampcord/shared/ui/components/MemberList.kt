package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.members.MemberGroupItem
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberSkeleton
import org.koin.compose.koinInject

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberList(
    navigationStore: NavigationStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    header: @Composable (() -> Unit)? = null
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    val activeChannel = navigationStore.selectedThread ?: navigationStore.selectedChannel
    val isDm = activeChannel?.type == 1 || activeChannel?.type == 3

    if (isDm) {
        val currentUser by userStore.currentUser.collectAsState()
        val allUsers by userStore.users.collectAsState()
        val privateChannels by koinInject<me.lampu.lampcord.shared.state.GuildStore>().privateChannels.collectAsState()
        val recipients = remember(activeChannel, privateChannels, allUsers) {
            val channel = privateChannels.find { it.id == activeChannel?.id } ?: activeChannel
            val list = mutableListOf<me.lampu.lampcord.shared.model.User>()
            currentUser?.let { list.add(it) }
            
            channel?.recipients?.let { list.addAll(it) }
            
            if (channel?.recipients.isNullOrEmpty() && !channel?.recipient_ids.isNullOrEmpty()) {
                channel?.recipient_ids?.forEach { id ->
                    allUsers[id]?.let { list.add(it) }
                }
            }

            list.distinctBy { it.id }
        }

        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surface,
            shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
        ) {
            LazyColumn(
                state = scrollState,
                modifier = Modifier
                    .fillMaxSize(),
                contentPadding = PaddingValues(top = 8.dp, bottom = 52.dp)
            ) {
                stickyHeader {
                    activeChannel?.let { MemberHeader(it) }
                }
                item {
                    Text(
                        text = "Members — ${recipients.size}",
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 0.5.sp
                        ),
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp)
                    )
                }
                items(recipients, key = { it.id }) { user ->
                    MemberItem(me.lampu.lampcord.shared.model.Member(user = user))
                }
            }
        }
        return
    }

    // Original Guild Member List Logic
    // Scroll to top when channel changes
    LaunchedEffect(activeChannel?.id) {
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
            shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp),
            tonalElevation = 0.dp
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                LazyColumn(
                    state = scrollState,
                    modifier = Modifier
                        .fillMaxSize(),
                    contentPadding = PaddingValues(bottom = 52.dp)
                ) {
                        stickyHeader {
                            activeChannel?.let { MemberHeader(it) }
                        }
                    if (header != null) {
                        stickyHeader {
                            Surface(
                                color = MaterialTheme.colorScheme.surface,
                                tonalElevation = 0.dp
                            ) {
                                header()
                            }
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
