package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.previewText
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.FinderStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.UserAvatar
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ForwardDialog(
    message: Message,
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    userStore: UserStore = koinInject(),
    entityStore: EntityStore = koinInject(),
    presenceStore: PresenceStore = koinInject()
) {
    var comment by remember { mutableStateOf("") }
    val selectedIds = remember { mutableStateListOf<String>() }

    LaunchedEffect(Unit) {
        finderStore.searchQuery = ""
    }

    val resultsState by finderStore.results.collectAsState()
    val query by finderStore.searchQueryFlow.collectAsState()
    val results = remember(query, resultsState) {
        resultsState.filter { it !is FinderResult.Guild && it.targetChannelId() != null }
    }

    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"

    val content: @Composable () -> Unit = {
        ForwardContent(
            message = message,
            comment = comment,
            onCommentChange = { comment = it },
            query = query,
            onQueryChange = { finderStore.searchQuery = it },
            results = results,
            selectedIds = selectedIds,
            onToggle = { id ->
                if (id in selectedIds) selectedIds.remove(id) else selectedIds.add(id)
            },
            onSend = {
                val currentUser = userStore.currentUser.value
                val targets = results.mapNotNull { r -> r.targetChannelId()?.takeIf { it in selectedIds }?.let { r to it } }
                    .distinctBy { it.second }
                targets.forEach { (_, channelId) ->
                    val targetChannel = entityStore.channels.value[channelId] ?: return@forEach
                    messageStore.forwardMessage(targetChannel, message)
                    if (comment.isNotBlank() && currentUser != null) {
                        messageStore.sendMessage(
                            channelId = targetChannel.id,
                            content = comment,
                            currentUser = currentUser,
                            guildId = targetChannel.guild_id
                        )
                    }
                }
                if (targets.isNotEmpty()) onDismiss()
            },
            onDismiss = onDismiss,
            userStore = userStore,
            presenceStore = presenceStore,
            isMobile = isMobile
        )
    }

    if (isMobile) {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.surface) {
            content()
        }
    } else {
        Dialog(
            onDismissRequest = onDismiss,
            properties = DialogProperties(usePlatformDefaultWidth = false)
        ) {
            Surface(
                modifier = Modifier
                    .widthIn(max = 560.dp)
                    .fillMaxWidth(0.95f)
                    .heightIn(max = 850.dp)
                    .fillMaxHeight(0.9f)
                    .clip(MaterialTheme.shapes.large),
                shape = MaterialTheme.shapes.large,
                color = MaterialTheme.colorScheme.surfaceContainerLowest,
            ) {
                content()
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ForwardContent(
    message: Message,
    comment: String,
    onCommentChange: (String) -> Unit,
    query: String,
    onQueryChange: (String) -> Unit,
    results: List<FinderResult>,
    selectedIds: List<String>,
    onToggle: (String) -> Unit,
    onSend: () -> Unit,
    onDismiss: () -> Unit,
    userStore: UserStore,
    presenceStore: PresenceStore,
    isMobile: Boolean
) {
    Scaffold(
        containerColor = Color.Transparent,
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        topBar = {
            TopAppBar(
                title = { Text("Forward To", fontWeight = FontWeight.SemiBold) },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.Default.Close, contentDescription = "Close")
                    }
                }
            )
        },
        bottomBar = {
            Surface(tonalElevation = 2.dp) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .imePadding()
                        .navigationBarsPadding()
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    MessagePreviewCompact(message)
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        TextField(
                            value = comment,
                            onValueChange = onCommentChange,
                            modifier = Modifier.weight(1f),
                            placeholder = { Text("Add an optional message...") },
                            colors = TextFieldDefaults.colors(
                                focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                focusedIndicatorColor = Color.Transparent,
                                unfocusedIndicatorColor = Color.Transparent
                            ),
                            shape = RoundedCornerShape(24.dp),
                            singleLine = true
                        )
                        IconButton(
                            onClick = onSend,
                            enabled = selectedIds.isNotEmpty()
                        ) {
                            Icon(
                                Icons.AutoMirrored.Filled.Send,
                                contentDescription = "Send",
                                tint = if (selectedIds.isNotEmpty()) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            TextField(
                value = query,
                onValueChange = onQueryChange,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                placeholder = { Text("Search") },
                leadingIcon = { Icon(Icons.Filled.Search, null) },
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent
                ),
                shape = RoundedCornerShape(28.dp),
                singleLine = true
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f))
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                items(results, key = { it.targetChannelId() ?: it.hashCode().toString() }) { result ->
                    val channelId = result.targetChannelId() ?: return@items
                    val selected = channelId in selectedIds
                    ForwardSelectableRow(
                        result = result,
                        selected = selected,
                        onClick = { onToggle(channelId) },
                        userStore = userStore,
                        presenceStore = presenceStore
                    )
                }
                if (results.isEmpty()) {
                    item {
                        Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                            Text("No results found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ForwardSelectableRow(
    result: FinderResult,
    selected: Boolean,
    onClick: () -> Unit,
    userStore: UserStore,
    presenceStore: PresenceStore
) {
    val user = when (result) {
        is FinderResult.UserResult -> result.user
        is FinderResult.DirectMessage -> result.recipient
        else -> null
    }
    val currentUser by userStore.currentUser.collectAsState()
    val status = user?.let { presenceStore.getUserStatus(it.id, currentUser?.id, "online") }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 8.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (user != null) {
            UserAvatar(user = user, size = 40.dp)
        } else {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = when (result) {
                        is FinderResult.Channel -> when (result.channel.type) {
                            2, 13 -> Icons.Filled.VolumeUp
                            5 -> Icons.Filled.Campaign
                            else -> Icons.Filled.Tag
                        }
                        else -> Icons.Filled.Tag
                    },
                    contentDescription = null,
                    modifier = Modifier.size(22.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = result.forwardTitle(),
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            result.forwardSubtitle()?.let { subtitle ->
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
        Checkbox(checked = selected, onCheckedChange = { onClick() })
    }
}

@Composable
private fun MessagePreviewCompact(message: Message) {
    Surface(
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(modifier = Modifier.padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
            AsyncImage(
                model = CdnUrls.getUserAvatarUrl(message.author?.id ?: "", message.author?.avatar),
                contentDescription = null,
                modifier = Modifier.size(28.dp).clip(CircleShape)
            )
            Spacer(Modifier.width(10.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "Forwarded",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    text = message.previewText.ifEmpty { "Attachment" },
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

private fun FinderResult.forwardTitle(): String = when (this) {
    is FinderResult.Guild -> guild.name ?: "Unnamed Server"
    is FinderResult.Channel -> channel.name ?: "unnamed-channel"
    is FinderResult.UserResult -> nickname ?: user.global_name ?: user.username ?: "Unknown user"
    is FinderResult.DirectMessage -> recipient?.let { it.global_name ?: it.username }
        ?: channel.name?.takeIf { it.isNotBlank() }
        ?: "Direct Message"
}

private fun FinderResult.forwardSubtitle(): String? = when (this) {
    is FinderResult.Guild -> "Server"
    is FinderResult.Channel -> guild?.name
    is FinderResult.UserResult -> user.username
    is FinderResult.DirectMessage -> if (channel.recipients?.size?.let { it > 1 } == true || channel.recipient_ids?.size?.let { it > 1 } == true) "Group DM" else "Direct Message"
}
