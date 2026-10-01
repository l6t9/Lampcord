package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.MessageReaction
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.rememberDiscordSheetState
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.ui.components.ImageLoadState

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun ReactionUsersDialog(
    channelId: String,
    messageId: String,
    reactions: List<MessageReaction>,
    initialEmoji: MessageReaction,
    onDismiss: () -> Unit,
    messageApi: MessageApi = koinInject()
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
    val pagerState = rememberPagerState(initialPage = reactions.indexOf(initialEmoji).coerceAtLeast(0)) { reactions.size }
    
    if (isMobile) {
        me.lampu.lampcord.shared.ui.components.DiscordBottomSheet(
            onDismissRequest = onDismiss,
            sheetState = rememberDiscordSheetState(),
            containerColor = MaterialTheme.colorScheme.surface,
            dragHandle = { BottomSheetDefaults.DragHandle() }
        ) {
            ReactionUsersContent(
                channelId = channelId,
                messageId = messageId,
                reactions = reactions,
                pagerState = pagerState,
                messageApi = messageApi
            )
        }
    } else {
        Dialog(onDismissRequest = onDismiss) {
            Surface(
                modifier = Modifier
                    .width(440.dp)
                    .heightIn(max = 600.dp),
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface
            ) {
                ReactionUsersContent(
                    channelId = channelId,
                    messageId = messageId,
                    reactions = reactions,
                    pagerState = pagerState,
                    messageApi = messageApi
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
private fun ReactionUsersContent(
    channelId: String,
    messageId: String,
    reactions: List<MessageReaction>,
    pagerState: PagerState,
    messageApi: MessageApi
) {
    val scope = rememberCoroutineScope()

    Column(modifier = Modifier.fillMaxWidth()) {
        SecondaryScrollableTabRow(
            selectedTabIndex = pagerState.currentPage,
            containerColor = Color.Transparent,
            edgePadding = 16.dp,
            divider = {}
        ) {
            reactions.forEachIndexed { index, reaction ->
                Tab(
                    selected = pagerState.currentPage == index,
                    onClick = { 
                        scope.launch {
                    if (!Settings.shared.reduceMotion) pagerState.animateScrollToPage(index) else pagerState.scrollToPage(index)
                        }
                    }
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        val url = reaction.emoji.getDisplayUrl()
                        var loadFailed by remember(url) { mutableStateOf(false) }
                        if (url != null && !loadFailed) {
                            AsyncImage(
                                model = url, 
                                contentDescription = null, 
                                modifier = Modifier.size(16.dp),
                                showPlaceholder = false,
                                onState = { state -> if (state is ImageLoadState.Error) loadFailed = true }
                            )
                        } else {
                            Text(reaction.emoji.name ?: "", fontSize = 14.sp)
                        }
                        Text(reaction.count.toString(), style = MaterialTheme.typography.labelMedium)
                    }
                }
            }
        }

        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

        HorizontalPager(
            state = pagerState,
            modifier = Modifier.weight(1f).fillMaxWidth(),
            verticalAlignment = Alignment.Top
        ) { page ->
            ReactionUserList(
                channelId = channelId,
                messageId = messageId,
                reaction = reactions[page],
                messageApi = messageApi
            )
        }
    }
}

@Composable
private fun ReactionUserList(
    channelId: String,
    messageId: String,
    reaction: MessageReaction,
    messageApi: MessageApi
) {
    var users by remember { mutableStateOf<List<User>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }

    val emojiStr = remember(reaction) {
        if (reaction.emoji.id != null) "${reaction.emoji.name}:${reaction.emoji.id}" else reaction.emoji.name ?: ""
    }

    LaunchedEffect(emojiStr) {
        isLoading = true
        users = messageApi.getReactionUsers(channelId, messageId, emojiStr)
        isLoading = false
    }

    Box(modifier = Modifier.fillMaxSize()) {
        if (isLoading) {
            ContainedLoadingIndicator(modifier = Modifier.align(Alignment.Center))
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                items(users) { user ->
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" }
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = null,
                            modifier = Modifier.size(32.dp).clip(CircleShape)
                        )
                        Spacer(modifier = Modifier.width(12.dp))
                        Text(
                            text = user.global_name ?: user.username ?: "Unknown",
                            style = MaterialTheme.typography.bodyLarge,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
            }
        }
    }
}
