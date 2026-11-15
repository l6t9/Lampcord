package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.title
import me.lampu.lampcord.shared.generated.resources.user_profile_message
import me.lampu.lampcord.shared.api.CdnUrls
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.ForumTag
import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.model.toTwemojiUrl
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.utils.EmojiIndex
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem

private enum class ForumSortMode(val label: String) {
    LatestActivity("Latest activity"),
    RecentlyCreated("Recently created"),
    MostMessages("Most messages")
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ForumPostList(
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject(),
    userApi: UserApi = koinInject()
) {
    val forumChannel = navigationStore.selectedChannel ?: return
    val allChannels by guildStore.allGuildChannels.collectAsState()
    val allUsers by userStore.users.collectAsState()
    val baseThreads = remember(forumChannel.id, allChannels) {
        guildStore.getForumThreads(forumChannel.id)
    }

    LaunchedEffect(baseThreads, allUsers.size) {
        val missingOwnerIds = baseThreads.asSequence()
            .mapNotNull { it.owner_id }
            .filter { ownerId ->
                ownerId !in allUsers && baseThreads.none { it.message?.author?.id == ownerId }
            }
            .distinct()
            .take(25)
            .toList()

        missingOwnerIds.forEach { ownerId ->
            try {
                val profile = userApi.getUserProfile(ownerId, forumChannel.guild_id)
                val user = profile?.user
                if (user != null) userStore.handleUserUpdate(user)
            } catch (_: Exception) {
            }
        }
    }

    var selectedTags by remember(forumChannel.id) { mutableStateOf(setOf<String>()) }
    var showNewPostDialog by remember(forumChannel.id) { mutableStateOf(false) }
    var sortMode by remember(forumChannel.id) { mutableStateOf(ForumSortMode.LatestActivity) }
    var showSortMenu by remember { mutableStateOf(false) }

    val forumThreads = remember(baseThreads, sortMode) {
        when (sortMode) {
            ForumSortMode.LatestActivity -> baseThreads.sortedByDescending { it.lastMessageId() ?: it.id }
            ForumSortMode.RecentlyCreated -> baseThreads.sortedByDescending { it.id }
            ForumSortMode.MostMessages -> baseThreads.sortedByDescending { it.message_count ?: it.total_message_sent ?: 0 }
        }
    }

    val filteredThreads = remember(forumThreads, selectedTags) {
        if (selectedTags.isEmpty()) forumThreads
        else forumThreads.filter { thread ->
            thread.applied_tags?.any { it in selectedTags } == true
        }
    }

    val availableTags = forumChannel.available_tags.orEmpty()

    Scaffold(
        containerColor = Color.Transparent,
        topBar = {
            Surface(color = MaterialTheme.colorScheme.surface) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    LazyRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        item {
                            Box {
                                FilterChip(
                                    selected = false,
                                    onClick = { showSortMenu = true },
                                    label = { Text(sortMode.label, maxLines = 1) },
                                    leadingIcon = {
                                        Icon(Icons.Filled.Sort, contentDescription = null, modifier = Modifier.size(18.dp))
                                    }
                                )
                                DropdownMenu(
                                    expanded = showSortMenu,
                                    onDismissRequest = { showSortMenu = false }
                                ) {
                                    ForumSortMode.entries.forEach { mode ->
                                        DropdownMenuItem(
                                            text = { Text(mode.label) },
                                            onClick = {
                                                sortMode = mode
                                                showSortMenu = false
                                            },
                                            leadingIcon = if (sortMode == mode) {
                                                { Icon(Icons.Filled.Check, contentDescription = null) }
                                            } else null
                                        )
                                    }
                                }
                            }
                        }

                        if (availableTags.isNotEmpty()) {
                            items(availableTags, key = { it.id }) { tag ->
                                val isSelected = tag.id in selectedTags
                                FilterChip(
                                    selected = isSelected,
                                    onClick = {
                                        selectedTags =
                                            if (isSelected) selectedTags - tag.id else selectedTags + tag.id
                                    },
                                    label = { Text(tag.name) },
                                    leadingIcon = {
                                        val hasEmoji = tag.emoji_id != null || tag.emoji_name != null
                                        if (hasEmoji) {
                                            TagEmoji(tag.emoji_id, tag.emoji_name)
                                        } else if (isSelected) {
                                            Icon(Icons.Filled.Check, contentDescription = null, modifier = Modifier.size(18.dp))
                                        }
                                    }
                                )
                            }
                        }

                        if (selectedTags.isNotEmpty()) {
                            item {
                                TextButton(onClick = { selectedTags = emptySet() }) {
                                    Text("Clear")
                                }
                            }
                        }
                    }
                }
            }
        },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { showNewPostDialog = true },
                containerColor = MaterialTheme.colorScheme.primaryContainer,
                contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                icon = { Icon(Icons.Filled.Add, contentDescription = null) },
                text = { Text("New Post") }
            )
        }
    ) { padding ->
        Box(modifier = Modifier.fillMaxSize().padding(padding)) {
            if (navigationStore.isForumLoading && forumThreads.isEmpty()) {
                ForumSkeleton()
            } else if (filteredThreads.isEmpty()) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Rounded.Forum,
                            contentDescription = null,
                            modifier = Modifier.size(40.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.height(12.dp))
                        Text(
                            if (baseThreads.isEmpty()) "No posts yet" else "No posts match your filters",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(filteredThreads, key = { it.id }) { thread ->
                        ForumPostItem(
                            thread = thread,
                            forumChannel = forumChannel,
                            onClick = { navigationStore.selectThread(thread, explicitlySelected = true) }
                        )
                    }
                    item { Spacer(Modifier.height(80.dp)) }
                }
            }
        }
    }

    if (showNewPostDialog) {
        NewPostDialog(
            onDismiss = { showNewPostDialog = false },
            forumChannelId = forumChannel.id,
            availableTags = availableTags
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
private fun ForumTagBadge(tag: ForumTag) {
    Surface(
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.secondaryContainer
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            if (tag.emoji_id != null || tag.emoji_name != null) {
                TagEmoji(tag.emoji_id, tag.emoji_name, size = 14.dp)
            }
            Text(
                text = tag.name,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSecondaryContainer,
                maxLines = 1
            )
        }
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
                    label = { Text(stringResource(Res.string.title)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    enabled = !isLoading
                )

                Spacer(Modifier.height(12.dp))

                TextField(
                    value = content,
                    onValueChange = { content = it },
                    label = { Text(stringResource(Res.string.user_profile_message)) },
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
                        Text(stringResource(Res.string.cancel))
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ForumPostItem(
    thread: Channel,
    forumChannel: Channel,
    onClick: () -> Unit,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    channelApi: ChannelApi = koinInject()
) {
    val allUsers by userStore.users.collectAsState()
    val scope = rememberCoroutineScope()

    val author = remember(thread.message, thread.owner_id, allUsers) {
        thread.message?.author ?: thread.owner_id?.let { allUsers[it] }
    }

    val timeAgo = remember(thread.id) {
        val timestamp = (thread.id.toLong() shr 22) + 1420070400000L
        DateTimeUtils.formatDiscordTimestamp(timestamp / 1000, "R")
    }

    val currentUser by userStore.currentUser.collectAsState()
    val members by userStore.members.collectAsState()
    val currentMember = remember(navigationStore.selectedGuild?.id, currentUser?.id, members) {
        val gId = navigationStore.selectedGuild?.id ?: return@remember null
        val uId = currentUser?.id ?: return@remember null
        members[gId]?.get(uId)
    }

    val canManageThreads = remember(navigationStore.selectedGuild, currentMember, forumChannel, currentUser?.id) {
        val guild = navigationStore.selectedGuild ?: return@remember false
        val member = currentMember ?: return@remember false
        PermissionHelper.hasPermission(member, guild, forumChannel, Permission.MANAGE_THREADS, currentUser?.id)
    }

    val isArchived = thread.thread_metadata?.archived == true
    val isLocked = thread.thread_metadata?.locked == true
    var showEditTags by remember(thread.id) { mutableStateOf(false) }
    var isUpdating by remember(thread.id) { mutableStateOf(false) }

    fun applyThreadUpdate(archived: Boolean? = null, locked: Boolean? = null, appliedTags: List<String>? = null) {
        if (isUpdating) return
        isUpdating = true
        scope.launch {
            try {
                val updated = channelApi.updateThread(
                    threadId = thread.id,
                    archived = archived,
                    locked = locked,
                    appliedTags = appliedTags
                )
                if (updated != null) {
                    guildStore.handleChannelCreateOrUpdate(
                        updated.copy(guild_id = updated.guild_id ?: thread.guild_id ?: forumChannel.guild_id)
                    )
                }
            } finally {
                isUpdating = false
            }
        }
    }

    val contextMenuItems = remember(thread.id, canManageThreads, isArchived, isLocked, isUpdating) {
        buildList {
            if (canManageThreads) {
                add(
                    ContextMenuItem(
                        if (isArchived) "Reopen post" else "Close post",
                        Icons.Rounded.Close,
                        onClick = { applyThreadUpdate(archived = !isArchived) }
                    )
                )
                add(
                    ContextMenuItem(
                        if (isLocked) "Unlock post" else "Lock post",
                        Icons.Rounded.Lock,
                        onClick = { applyThreadUpdate(locked = !isLocked) }
                    )
                )
                add(
                    ContextMenuItem(
                        "Edit tags",
                        Icons.Rounded.Sell,
                        onClick = { showEditTags = true }
                    )
                )
            }
        }
    }

    val snippet = remember(thread.message?.content) {
        val content = thread.message?.content?.trim().orEmpty()
        when {
            content.isNotBlank() -> content
            !thread.message?.attachments.isNullOrEmpty() -> "Attachment"
            !thread.message?.embeds.isNullOrEmpty() -> "Embed"
            else -> "No message preview"
        }
    }

    val messageCount = (thread.message_count ?: thread.total_message_sent ?: 0).coerceAtLeast(0)

    ContextMenu(items = contextMenuItems, enabled = contextMenuItems.isNotEmpty()) {
        ElevatedCard(
            onClick = onClick,
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(20.dp),
            colors = CardDefaults.elevatedCardColors(
                containerColor = if (isArchived)
                    MaterialTheme.colorScheme.surfaceContainer
                else
                    MaterialTheme.colorScheme.surfaceContainerLow
            ),
            elevation = CardDefaults.elevatedCardElevation(defaultElevation = if (isArchived) 0.dp else 1.dp)
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    val avatarUrl = author?.avatar?.let { CdnUrls.getUserAvatarUrl(author.id, it, 64) }
                    if (avatarUrl != null) {
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp).clip(CircleShape),
                            showPlaceholder = false
                        )
                    } else {
                        Surface(
                            modifier = Modifier.size(20.dp),
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.secondaryContainer
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(
                                    (author?.global_name ?: author?.username ?: "?").take(1).uppercase(),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSecondaryContainer
                                )
                            }
                        }
                    }
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = author?.global_name ?: author?.username ?: "Original poster",
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = " · $timeAgo",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1
                    )
                    Spacer(Modifier.weight(1f))
                    if (thread.flags?.let { it and (1 shl 1) != 0 } == true) {
                        Icon(
                            Icons.Filled.PushPin,
                            contentDescription = "Pinned",
                            modifier = Modifier.size(16.dp),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }

                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        if (isLocked) {
                            Icon(
                                Icons.Rounded.Lock,
                                contentDescription = "Locked",
                                modifier = Modifier.size(18.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Spacer(Modifier.width(6.dp))
                        }
                        Text(
                            text = thread.name ?: "Untitled post",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                            color = if (isArchived) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface
                        )
                    }

                    Text(
                        text = snippet,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                }

                if (!thread.applied_tags.isNullOrEmpty() && !forumChannel.available_tags.isNullOrEmpty()) {
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        thread.applied_tags.forEach { tagId ->
                            val tag = forumChannel.available_tags.find { it.id == tagId } ?: return@forEach
                            ForumTagBadge(tag)
                        }
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            Icon(
                                Icons.Rounded.ChatBubble,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Text(
                                text = "$messageCount",
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        if (isArchived) {
                            Text(
                                text = "Closed",
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        if (isUpdating) {
                            CircularProgressIndicator(modifier = Modifier.size(14.dp), strokeWidth = 2.dp)
                        }
                    }

                    val reactions = thread.reactions ?: thread.message?.reactions
                    if (!reactions.isNullOrEmpty()) {
                        val firstReaction = reactions.first()
                        val totalCount = reactions.sumOf { it.count }
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
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
                                Text(
                                    firstReaction.emoji.name ?: "",
                                    style = MaterialTheme.typography.labelLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                            Text(
                                text = totalCount.toString(),
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }

    if (showEditTags) {
        EditForumTagsDialog(
            thread = thread,
            availableTags = forumChannel.available_tags ?: emptyList(),
            onDismiss = { showEditTags = false },
            onSave = { tags ->
                showEditTags = false
                applyThreadUpdate(appliedTags = tags)
            }
        )
    }
}

@Composable
fun EditForumTagsDialog(
    thread: Channel,
    availableTags: List<ForumTag>,
    onDismiss: () -> Unit,
    onSave: (List<String>) -> Unit
) {
    var selected by remember(thread.id, thread.applied_tags) {
        mutableStateOf(thread.applied_tags?.toSet() ?: emptySet())
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Edit tags") },
        text = {
            if (availableTags.isEmpty()) {
                Text("This forum has no tags.")
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        "Choose tags for this post",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        availableTags.forEach { tag ->
                            val isSelected = tag.id in selected
                            FilterChip(
                                selected = isSelected,
                                onClick = {
                                    selected = if (isSelected) selected - tag.id else selected + tag.id
                                },
                                label = { Text(tag.name) },
                                leadingIcon = { TagEmoji(tag.emoji_id, tag.emoji_name) }
                            )
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { onSave(selected.toList()) }, enabled = availableTags.isNotEmpty()) {
                Text("Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.cancel)) }
        }
    )
}

@Composable
fun ForumSkeleton() {
    Column(
        modifier = Modifier.fillMaxSize().padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        repeat(4) {
            Surface(
                modifier = Modifier.fillMaxWidth().height(132.dp),
                shape = RoundedCornerShape(20.dp),
                color = MaterialTheme.colorScheme.surfaceContainerLow,
                tonalElevation = 1.dp
            ) {}
        }
    }
}
