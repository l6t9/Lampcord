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
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.members.MemberGroupItem
import me.lampu.lampcord.shared.ui.components.members.MemberHeader
import me.lampu.lampcord.shared.ui.components.members.MemberItem
import me.lampu.lampcord.shared.ui.components.members.MemberRowEnv
import me.lampu.lampcord.shared.ui.components.members.MemberSkeleton
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlin.math.abs
import kotlin.time.Duration.Companion.milliseconds

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberList(
    navigationStore: NavigationStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    relationshipStore: RelationshipStore = koinInject(),
    header: @Composable (() -> Unit)? = null
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    val activeChannel = navigationStore.selectedThread ?: navigationStore.selectedChannel
    val isDm = activeChannel?.type == 1 || activeChannel?.type == 3

    // Skip rendering images decodes while scrolling faster than they can be loaded.
    var loadImages by remember { mutableStateOf(true) }
    LaunchedEffect(scrollState) {
        var lastIndex = scrollState.firstVisibleItemIndex
        var lastTime = getCurrentTimeMillis()
        snapshotFlow { scrollState.firstVisibleItemIndex to scrollState.isScrollInProgress }
            .collect { (index, moving) ->
                val now = getCurrentTimeMillis()
                if (!moving) {
                    loadImages = true
                    lastIndex = index
                    lastTime = now
                    return@collect
                }
                val elapsed = now - lastTime
                if (elapsed < 100) return@collect
                loadImages = abs(index - lastIndex) * 1000L / elapsed < 12L
                lastIndex = index
                lastTime = now
            }
    }

    val presences = presenceStore.presences.collectAsState()
    val relationshipTypes = relationshipStore.relationshipTypes.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()
    val currentUserId = currentUser?.id
    val userSettings = settingsStore.userSettings
    val currentUserStatus = userSettings?.status
    val developerMode = userSettings?.developer_mode == true
    val selectedGuild = navigationStore.selectedGuild
    val isTouch = remember { getPlatformName().let { it == "android" || it == "ios" } }
    val reduceMotion = Settings.shared.reduceMotion
    val env = remember(selectedGuild, currentUserId, currentUserStatus, developerMode, reduceMotion, loadImages) {
        MemberRowEnv(
            guild = selectedGuild,
            currentUserId = currentUserId,
            currentUserStatus = currentUserStatus,
            developerMode = developerMode,
            presences = presences,
            relationshipTypes = relationshipTypes,
            animate = !isTouch && !reduceMotion,
            isTouch = isTouch,
            loadImages = loadImages
        )
    }

    if (isDm) {
        val allUsers by userStore.users.collectAsState()
        val privateChannels by koinInject<GuildStore>().privateChannels.collectAsState()
        val recipients = remember(activeChannel, privateChannels, allUsers, currentUser) {
            val channel = privateChannels.find { it.id == activeChannel.id } ?: activeChannel
            val list = mutableListOf<User>()
            currentUser?.let { list.add(it) }

            channel.recipients?.let { list.addAll(it) }

            if (channel.recipients.isNullOrEmpty() && !channel.recipient_ids.isNullOrEmpty()) {
                channel.recipient_ids.forEach { id ->
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
                    MemberHeader(activeChannel)
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
                    MemberItem(Member(user = user), env)
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

    val block by remember { derivedStateOf { scrollState.firstVisibleItemIndex / 100 } }
    val settled by remember { derivedStateOf { !scrollState.isScrollInProgress } }
    val rowCount = memberListStore.memberListRowCount
    val ranges = remember(block, rowCount) {
        val start = block * 100
        val blocks = linkedSetOf(0, start, start + 100).filter { it == 0 || it < rowCount }
        blocks.map { listOf(it, it + 99) }
    }

    val channelId = navigationStore.selectedChannel?.id
    var lastRequestedChannelId by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(ranges, channelId, settled) {
        if (channelId == null || !settled) return@LaunchedEffect
        if (channelId != lastRequestedChannelId) {
            memberListStore.requestMemberListRange(ranges)
            lastRequestedChannelId = channelId
        } else {
            delay(300.milliseconds)
            memberListStore.requestMemberListRange(ranges)
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

                    val rows = memberListStore.memberListItems
                    val liveCount = rows.size
                    if (liveCount == 0) {
                        items(20) {
                            MemberSkeleton()
                        }
                    } else {
                        items(
                            count = liveCount,
                            key = { index ->
                                val item = rows.getOrNull(index)
                                val baseId = item?.member?.userId() ?: item?.group?.id ?: "null"
                                // 126.21 Parity: Discord member lists are index-based.
                                // We include the index in the key to prevent crashes if the state is temporarily inconsistent
                                // (e.g. during a channel switch or rapid gateway updates).
                                "$index-$baseId"
                            },
                            contentType = { index ->
                                val item = rows.getOrNull(index)
                                when {
                                    item?.member != null -> 0
                                    item?.group != null -> 1
                                    else -> 2
                                }
                            }
                        ) { index ->
                            val item = rows.getOrNull(index)
                            when {
                                item?.member != null -> MemberItem(item.member, env)
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
