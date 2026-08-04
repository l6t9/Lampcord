package com.example.lampcord.shared.ui.components

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
import com.example.lampcord.shared.model.Member
import com.example.lampcord.shared.model.MemberListGroup
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.icons.Icons
import com.example.lampcord.shared.utils.setClipboardText

@Composable
fun MemberList(chatState: ChatState) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

    // Scroll to top when channel changes
    LaunchedEffect(chatState.selectedChannel?.id) {
        scrollState.scrollToItem(0)
    }

    // Replicating Paicord scrolling range logic: always keep the first 100 rows subscribed,
    // plus up to two 100-row blocks around the current scroll position, debounced via LaunchedEffect.
    val firstVisible = scrollState.firstVisibleItemIndex
    val rowCount = chatState.memberListRowCount
    val ranges = remember(firstVisible, rowCount) {
        val pairs = mutableListOf(listOf(0, 99))
        val maxIndex = rowCount - 1
        if (maxIndex >= 100) {
            val currentBlock = firstVisible / 100
            val maxBlock = maxIndex / 100
            val clampedBlock = minOf(currentBlock, maxBlock)
            var block = clampedBlock
            var added = 0
            while (added < 2 && block >= 1) {
                val start = block * 100
                if (pairs.none { it[0] == start }) {
                    pairs.add(1, listOf(start, start + 99))
                    added++
                }
                block--
            }
        }
        pairs
    }
    LaunchedEffect(ranges) {
        if (ranges.isNotEmpty()) {
            // Debounce the request to avoid spamming the gateway
            kotlinx.coroutines.delay(300)
            chatState.requestMemberListRange(ranges)
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
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(0.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                LazyColumn(
                    state = scrollState,
                    modifier = Modifier.fillMaxSize().padding(top = 8.dp),
                    contentPadding = PaddingValues(bottom = 52.dp)
                ) {
                    if (chatState.memberListRowCount == 0) {
                        items(20) {
                            MemberSkeleton()
                        }
                    } else {
                        items(chatState.memberListRowCount) { index ->
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

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberItem(member: Member, chatState: ChatState) {
    val user = member.user ?: return
    val avatarUrl = member.avatar?.let {
        "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
    }

    val roleColor = remember(member.roles, chatState.selectedGuild) {
        val guild = chatState.selectedGuild ?: return@remember Color.Unspecified
        val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
        val highestRole = memberRoles.maxByOrNull { it.position }
        if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
    }

    val contextMenuItems = remember(user, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Profile", Icons.Filled.AccountCircle) { chatState.showProfile(user.id) },
            ContextMenuItem("Mention", Icons.Outlined.AlternateEmail) {
                val channelId = chatState.selectedChannel?.id ?: return@ContextMenuItem
                val current = chatState.draftMessages[channelId] ?: ""
                chatState.draftMessages[channelId] = "$current <@${user.id}> "
            },
            ContextMenuItem("Message", Icons.Filled.Share) { /* TODO */ }
        )
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy User ID", Icons.Filled.Dns) { setClipboardText(user.id) })
        }
        items
    }

    var itemPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var isHovered by remember { mutableStateOf(false) }

    val isListening = member.presence?.activities?.any { it.type == 2 } == true
    val isOffline = (member.presence?.status == "offline" || member.presence == null) && !isListening

    ContextMenu(
        items = contextMenuItems,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp)
            .onGloballyPositioned { itemPosition = it.positionInRoot() }
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
            .graphicsLayer {
                alpha = if (isOffline && !isHovered) 0.4f else 1f
            },
        shape = RoundedCornerShape(8.dp)
    ) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(44.dp),
            onClick = { chatState.showProfile(user.id, itemPosition) },
            color = Color.Transparent,
            shape = RoundedCornerShape(8.dp)
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(modifier = Modifier.size(32.dp)) {
                    Surface(
                        modifier = Modifier.fillMaxSize(),
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer
                    ) {
                        if (avatarUrl != null) {
                            AsyncImage(model = avatarUrl, contentDescription = user.username, modifier = Modifier.fillMaxSize())
                        } else {
                            Box(contentAlignment = Alignment.Center) {
                                Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                            }
                        }
                    }
                    
                    val status = chatState.getUserStatus(user.id)
                    
                    StatusIndicator(
                        status = status,
                        size = 14.dp,
                        modifier = Modifier.align(Alignment.BottomEnd).offset(x = 2.dp, y = 2.dp),
                        borderColor = MaterialTheme.colorScheme.surfaceContainerLow,
                        backgroundColor = MaterialTheme.colorScheme.surfaceContainerLow
                    )
                }
                Spacer(modifier = Modifier.width(12.dp))
                Column {
                    Text(
                        text = member.nick ?: user.global_name ?: user.username,
                        style = MaterialTheme.typography.bodyMedium,
                        color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    
                    val activities = member.presence?.activities ?: emptyList()
                    val customStatus = activities.find { it.type == 4 }
                    val otherActivity = activities.find { it.type != 4 }
                    
                    if (customStatus != null) {
                        UserActivity(customStatus, compact = true)
                    } else if (otherActivity != null) {
                        UserActivity(otherActivity, compact = true)
                    }
                }
            }
        }
    }
}

@Composable
fun MemberGroupItem(group: MemberListGroup, chatState: ChatState) {
    val role = remember(group.id, chatState.selectedGuild) {
        chatState.selectedGuild?.roles?.find { it.id == group.id }
    }
    val roleName = remember(group.id, role) {
        if (group.id == "online") "Online"
        else if (group.id == "offline") "Offline"
        else role?.name ?: group.id
    }
    
    // Find up-to-date count from chatState.memberListGroups if the item's count is stale
    val displayCount = remember(group, chatState.memberListGroups.size) {
        chatState.memberListGroups.find { it.id == group.id }?.let { it.count ?: it.member_count } ?: group.count ?: group.member_count ?: 0
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .padding(top = 8.dp),
        contentAlignment = Alignment.CenterStart
    ) {
        Text(
            text = "$roleName — $displayCount",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.5.sp
        )
    }
}

@Composable
fun MemberSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(44.dp)
            .padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(32.dp),
            shape = CircleShape
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            ShimmerBox(
                modifier = Modifier
                    .width(100.dp)
                    .height(14.dp),
                shape = RoundedCornerShape(7.dp)
            )
            ShimmerBox(
                modifier = Modifier
                    .width(60.dp)
                    .height(10.dp),
                shape = RoundedCornerShape(5.dp)
            )
        }
    }
}
