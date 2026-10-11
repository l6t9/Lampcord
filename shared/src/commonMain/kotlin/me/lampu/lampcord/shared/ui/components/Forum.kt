package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.ForumTag
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.utils.EmojiIndex
import me.lampu.lampcord.shared.model.toTwemojiUrl
import me.lampu.lampcord.shared.model.Emoji as ModelEmoji
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.Permission

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ForumPostList(
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val forumChannel = navigationStore.selectedChannel ?: return
    val allChannels by guildStore.allGuildChannels.collectAsState()
    val forumThreads = remember(forumChannel.id, allChannels.size) { 
        guildStore.getForumThreads(forumChannel.id)
            .sortedByDescending { it.lastMessageId() ?: it.id } 
    }

    var selectedTags by remember { mutableStateOf(setOf<String>()) }
    var showNewPostDialog by remember { mutableStateOf(false) }

    val filteredThreads = remember(forumThreads, selectedTags) {
        if (selectedTags.isEmpty()) forumThreads
        else forumThreads.filter { thread -> 
            thread.applied_tags?.any { it in selectedTags } == true 
        }
    }

    Scaffold(
        containerColor = Color.Transparent,
        topBar = {
            Column(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Surface(
                        onClick = { /* Sort */ },
                        shape = RoundedCornerShape(8.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(Icons.Filled.Sort, null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Sort & View", style = MaterialTheme.typography.labelLarge)
                        }
                    }

                    Surface(
                        onClick = { /* All Tags */ },
                        shape = RoundedCornerShape(8.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(Icons.Filled.Sell, null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Tags", style = MaterialTheme.typography.labelLarge)
                        }
                    }
                }

                if (!forumChannel.available_tags.isNullOrEmpty()) {
                    LazyRow(
                        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        items(forumChannel.available_tags) { tag ->
                            val isSelected = tag.id in selectedTags
                            FilterChip(
                                selected = isSelected,
                                onClick = {
                                    selectedTags = if (isSelected) selectedTags - tag.id else selectedTags + tag.id
                                },
                                label = { Text(tag.name) },
                                leadingIcon = { TagEmoji(tag.emoji_id, tag.emoji_name) }
                            )
                        }
                    }
                }
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
            }
        },
        floatingActionButton = {
            FloatingActionButton(
                onClick = { showNewPostDialog = true },
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary
            ) {
                Icon(Icons.Filled.Add, "New Post")
            }
        }
    ) { padding ->
        Box(modifier = Modifier.fillMaxSize().padding(padding)) {
            if (navigationStore.isForumLoading && forumThreads.isEmpty()) {
                ForumSkeleton()
            } else if (filteredThreads.isEmpty()) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text("No posts found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(filteredThreads, key = { it.id }) { thread ->
                        ForumPostItem(thread, forumChannel, onClick = { navigationStore.selectThread(thread, explicitlySelected = true) })
                    }
                }
            }
        }
    }

    if (showNewPostDialog) {
        NewPostDialog(
            onDismiss = { showNewPostDialog = false },
            forumChannelId = forumChannel.id,
            availableTags = forumChannel.available_tags ?: emptyList()
        )
    }
}

@Composable
fun TagEmoji(emojiId: String?, emojiName: String?, size: androidx.compose.ui.unit.Dp = 18.dp) {
    val emojiUrl = remember(emojiId, emojiName) {
        if (emojiId != null) {
            customEmojiCdnUrl(emojiId, animated = false, size = 64)
        } else if (emojiName != null) {
            val unicode = EmojiIndex.getCharForName(emojiName) ?: emojiName
            unicode.toTwemojiUrl()
        } else null
    }

    if (emojiUrl != null) {
        AsyncImage(
            model = emojiUrl,
            contentDescription = emojiName,
            modifier = Modifier.size(size),
            showPlaceholder = false
        )
    }
}

@Composable
fun NewPostDialog(
    onDismiss: () -> Unit,
    forumChannelId: String,
    availableTags: List<ForumTag>,
    channelApi: ChannelApi = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    var title by remember { mutableStateOf("") }
    var content by remember { mutableStateOf("") }
    var appliedTags by remember { mutableStateOf(setOf<String>()) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.fillMaxWidth().padding(16.dp)
        ) {
            Column(modifier = Modifier.padding(24.dp)) {
                Text(
                    text = "New Post",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(Modifier.height(16.dp))
                
                TextField(
                    value = title,
                    onValueChange = { title = it },
                    label = { Text("Title") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    enabled = !isLoading
                )
                
                Spacer(Modifier.height(12.dp))
                
                TextField(
                    value = content,
                    onValueChange = { content = it },
                    label = { Text("Message") },
                    modifier = Modifier.fillMaxWidth().heightIn(min = 100.dp),
                    enabled = !isLoading
                )

                if (availableTags.isNotEmpty()) {
                    Spacer(Modifier.height(16.dp))
                    Text("Tags", style = MaterialTheme.typography.labelLarge)
                    Spacer(Modifier.height(8.dp))
                    androidx.compose.foundation.layout.FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        availableTags.forEach { tag ->
                            val isSelected = tag.id in appliedTags
                            FilterChip(
                                selected = isSelected,
                                onClick = {
                                    appliedTags = if (isSelected) appliedTags - tag.id else appliedTags + tag.id
                                },
                                label = { Text(tag.name) },
                                leadingIcon = { TagEmoji(tag.emoji_id, tag.emoji_name) }
                            )
                        }
                    }
                }
                
                Spacer(Modifier.height(24.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    androidx.compose.material3.TextButton(onClick = onDismiss, enabled = !isLoading) {
                        Text("Cancel")
                    }
                    Spacer(Modifier.width(8.dp))
                    Button(
                        onClick = {
                            if (title.isNotBlank() && content.isNotBlank()) {
                                isLoading = true
                                scope.launch {
                                    val thread = channelApi.createThread(forumChannelId, title, content, appliedTags.toList())
                                    if (thread != null) {
                                        guildStore.handleChannelCreateOrUpdate(thread)
                                        navigationStore.selectThread(thread, explicitlySelected = true)
                                        onDismiss()
                                    } else {
                                        isLoading = false
                                    }
                                }
                            }
                        },
                        enabled = !isLoading && title.isNotBlank() && content.isNotBlank()
                    ) {
                        if (isLoading) {
                            androidx.compose.material3.CircularProgressIndicator(
                                modifier = Modifier.size(24.dp),
                                color = MaterialTheme.colorScheme.onPrimary,
                                strokeWidth = 2.dp
                            )
                        } else {
                            Text("Post")
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ForumPostItem(
    thread: Channel, 
    forumChannel: Channel, 
    onClick: () -> Unit,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject()
) {
    val allUsers by userStore.users.collectAsState()
    val author = remember(thread, allUsers) {
        thread.message?.author ?: thread.owner_id?.let { allUsers[it] }
    }
    
    val timeAgo = remember(thread) {
        val timestamp = (thread.id.toLong() shr 22) + 1420070400000L
        DateTimeUtils.formatDiscordTimestamp(timestamp / 1000, "R")
    }

    val currentUser by userStore.currentUser.collectAsState()
    val currentMember = remember(navigationStore.selectedGuild, currentUser, userStore.members) {
        val gId = navigationStore.selectedGuild?.id ?: return@remember null
        val uId = currentUser?.id ?: return@remember null
        userStore.getMember(gId, uId)
    }

    val canManageThreads = remember(navigationStore.selectedGuild, currentMember, forumChannel, currentUser) {
        val guild = navigationStore.selectedGuild ?: return@remember false
        val member = currentMember ?: return@remember false
        PermissionHelper.hasPermission(member, guild, forumChannel, Permission.MANAGE_THREADS, currentUser?.id)
    }

    val isArchived = thread.thread_metadata?.archived == true
    val isLocked = thread.thread_metadata?.locked == true

    val contextMenuItems = remember(thread, canManageThreads, isArchived, isLocked) {
        val items = mutableListOf<ContextMenuItem>()
        if (canManageThreads) {
            items.add(
                ContextMenuItem(
                    if (isArchived) "Restore Post" else "Close Post",
                    Icons.Rounded.Close,
                    onClick = { /* TODO: Implement in GuildStore/ChannelApi */ }
                )
            )
            items.add(
                ContextMenuItem(
                    if (isLocked) "Unlock Post" else "Lock Post",
                    Icons.Rounded.Lock,
                    onClick = { /* TODO */ }
                )
            )
            items.add(
                ContextMenuItem(
                    "Edit Tags",
                    Icons.Rounded.Sell,
                    onClick = { /* TODO */ }
                )
            )
        }
        items
    }

    ContextMenu(
        items = contextMenuItems,
        enabled = contextMenuItems.isNotEmpty()
    ) {
        Surface(
            onClick = onClick,
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            border = androidx.compose.foundation.BorderStroke(
                1.dp,
                MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f)
            )
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                if (thread.flags?.let { it and (1 shl 1) != 0 } == true) {
                    Icon(
                        Icons.Filled.PushPin,
                        null,
                        modifier = Modifier.size(16.dp).align(Alignment.Start),
                        tint = MaterialTheme.colorScheme.primary
                    )
                    Spacer(Modifier.height(8.dp))
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    val avatarUrl =
                        author?.avatar?.let { "https://cdn.discordapp.com/avatars/${author.id}/$it.png?size=64" }
                    AsyncImage(
                        model = avatarUrl,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp).clip(CircleShape),
                        showPlaceholder = false
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = author?.global_name ?: author?.username ?: "User",
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = timeAgo,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    )
                }

                Spacer(Modifier.height(8.dp))

                Row(verticalAlignment = Alignment.CenterVertically) {
                    if (isLocked) {
                        Icon(
                            Icons.Rounded.Lock,
                            null,
                            modifier = Modifier.size(18.dp).padding(end = 8.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                    }
                    Text(
                        text = thread.name ?: "Untitled Post",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false),
                        color = if (isArchived) MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f) else Color.Unspecified
                    )
                }

                Spacer(Modifier.height(4.dp))

                val snippet = thread.message?.content ?: "..."
                Text(
                    text = snippet,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f),
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )

                if (!thread.applied_tags.isNullOrEmpty() && !forumChannel.available_tags.isNullOrEmpty()) {
                    Spacer(Modifier.height(12.dp))
                    androidx.compose.foundation.layout.FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        thread.applied_tags.forEach { tagId ->
                            val tag = forumChannel.available_tags.find { it.id == tagId }
                            if (tag != null) {
                                Surface(
                                    color = MaterialTheme.colorScheme.surfaceVariant,
                                    shape = RoundedCornerShape(4.dp)
                                ) {
                                    Row(
                                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        TagEmoji(tag.emoji_id, tag.emoji_name, size = 14.dp)
                                        if (tag.emoji_id != null || tag.emoji_name != null) {
                                            Spacer(Modifier.width(4.dp))
                                        }
                                        Text(tag.name, style = MaterialTheme.typography.labelSmall)
                                    }
                                }
                            }
                        }
                    }
                }

                Spacer(Modifier.height(16.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            Icons.Rounded.ChatBubble,
                            null,
                            modifier = Modifier.size(16.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                        Spacer(Modifier.width(6.dp))
                        Text(
                            text = (thread.message_count ?: 0).toString(),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                        )
                        if (isArchived) {
                            Spacer(Modifier.width(12.dp))
                            Surface(
                                color = MaterialTheme.colorScheme.surfaceVariant,
                                shape = RoundedCornerShape(4.dp)
                            ) {
                                Text(
                                    "Closed",
                                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                                    style = MaterialTheme.typography.labelSmall,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                        }
                    }

                    val reactions = thread.reactions ?: thread.message?.reactions
                    if (!reactions.isNullOrEmpty()) {
                        val firstReaction = reactions.first()
                        val totalCount = reactions.sumOf { it.count }
                        Surface(
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                        ) {
                            Row(
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                val emojiUrl = firstReaction.emoji.getDisplayUrl()
                                if (emojiUrl != null) {
                                    AsyncImage(
                                        model = emojiUrl,
                                        contentDescription = null,
                                        modifier = Modifier.size(16.dp),
                                        showPlaceholder = false
                                    )
                                } else {
                                    Text(firstReaction.emoji.name ?: "", fontSize = 12.sp)
                                }
                                Spacer(Modifier.width(6.dp))
                                Text(
                                    text = totalCount.toString(),
                                    style = MaterialTheme.typography.labelMedium,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ForumSkeleton() {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        repeat(3) {
            Surface(
                modifier = Modifier.fillMaxWidth().height(160.dp),
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.1f)
            ) {}
        }
    }
}
