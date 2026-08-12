@file:OptIn(ExperimentalFoundationApi::class, ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.layout
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import androidx.compose.ui.zIndex
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.EmojiPicker
import me.lampu.lampcord.shared.ui.components.ForwardedMessage
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.components.messagebody.MessageBody
import me.lampu.lampcord.shared.ui.components.messagebody.ReactionsView
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.UserAvatar
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject
import kotlin.math.roundToInt
import kotlin.time.Duration.Companion.milliseconds

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MessageItem(
    message: Message,
    priorMessage: Message? = null,
    messageStore: MessageStore = koinInject(),
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    discordClient: DiscordClient = koinInject()
) {
    if (message.type != null && message.type != 0 && message.type != 19 && message.type != 20) {
        SystemMessage(message)
        return
    }

    var isHovered by remember { mutableStateOf(false) }
    var showReactionPicker by remember { mutableStateOf(false) }
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showCreateThreadDialog by remember { mutableStateOf(false) }
    
    val currentUser by userStore.currentUser.collectAsState()
    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()

    val contextMenuItems = remember(message, currentUser, userSettings) {
        if (message.isPending) {
            return@remember listOf(
                ContextMenuItem("Delete", Icons.Default.Delete) { 
                    messageStore.deletePendingMessage(message)
                }
            )
        }
        
        val isMe = message.author?.id == currentUser?.id
        val items = mutableListOf(
            ContextMenuItem("Add Reaction", Icons.Filled.AddReaction) { showReactionPicker = true },
            ContextMenuItem("Reply", Icons.Rounded.Reply) { messageStore.replyingTo = message },
            ContextMenuItem("Forward", Icons.Filled.Forward) { navigationStore.forwardingMessage = message },
            ContextMenuItem("Copy Text", Icons.Filled.ContentCopy) { setClipboardText(message.content) },
            ContextMenuItem("Copy Link", Icons.Filled.Link) {
                val guildId = message.guild_id ?: navigationStore.selectedGuild?.id ?: "@me"
                val channelId = message.channel_id
                val messageId = message.id
                setClipboardText("https://discord.com/channels/$guildId/$channelId/$messageId")
            },
            ContextMenuItem("Mention", Icons.Rounded.AlternateEmail) {
                val channelId = message.channel_id
                val current = messageStore.draftMessages[channelId] ?: ""
                messageStore.draftMessages[channelId] = "$current <@${message.author?.id}> "
            },
            ContextMenuItem("Create Thread", Icons.Filled.Tag) {
                showCreateThreadDialog = true
            }
        )
        
        if (isMe) {
            items.add(ContextMenuItem("Edit Message", Icons.Filled.Edit) { 
                messageStore.editingMessage = message 
            })
        }
        
        items.add(ContextMenuItem(if (message.pinned) "Unpin Message" else "Pin Message", Icons.Filled.PushPin) {
            if (message.pinned) messageStore.unpinMessage(message) else messageStore.pinMessage(message)
        })

        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy Message ID", Icons.Filled.Dns) { setClipboardText(message.id) })
            items.add(ContextMenuItem("Copy Author ID", Icons.Filled.Dns) { setClipboardText(message.author?.id ?: "") })
        }

        items.add(ContextMenuItem("Delete Message", Icons.Filled.Delete) { 
            showDeleteDialog = true
        })
        
        items
    }

    val currentMember = remember(navigationStore.selectedGuild, currentUser) {
        val guildId = navigationStore.selectedGuild?.id ?: return@remember null
        val userId = currentUser?.id ?: return@remember null
        userStore.getMember(guildId, userId)
    }

    val isMentioned by remember(message, currentUser, currentMember) {
        derivedStateOf { if (message.isPending) false else messageStore.isMessageMentioningMe(message, currentUser, currentMember) }
    }
    
    val messageAlpha by animateFloatAsState(if (message.isPending) 0.5f else 1f)

    val isInline = priorMessage != null
    val isHighlighted = messageStore.highlightedMessageId == message.id
    
    val tapTapEnabled = remember { me.lampu.lampcord.shared.settings.Settings.shared.tapTap }
    val gestureMode = remember { me.lampu.lampcord.shared.settings.Settings.shared.chatGestures }
    var offsetX by remember { mutableFloatStateOf(0f) }

    LaunchedEffect(isHighlighted) {
        if (isHighlighted) {
            kotlinx.coroutines.delay(2000.milliseconds)
            messageStore.highlightedMessageId = null
        }
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .offset { IntOffset(offsetX.roundToInt(), 0) }
            .pointerInput(message.id, gestureMode) {
                if (gestureMode == me.lampu.lampcord.shared.settings.ChatGestures.SWIPE_TO_REPLY) {
                    detectHorizontalDragGestures(
                        onDragEnd = {
                            if (offsetX < -80f) {
                                messageStore.replyingTo = message
                            }
                            offsetX = 0f
                        },
                        onHorizontalDrag = { change, dragAmount ->
                            val newOffset = (offsetX + dragAmount).coerceIn(-150f, 0f)
                            if (newOffset != offsetX) {
                                offsetX = newOffset
                                change.consume()
                            }
                        }
                    )
                }
            }
            .pointerInput(message.id, tapTapEnabled) {
                if (tapTapEnabled) {
                    detectTapGestures(
                        onDoubleTap = {
                            val isMe = message.author?.id == currentUser?.id
                            if (isMe) {
                                messageStore.editingMessage = message
                            } else {
                                messageStore.replyingTo = message
                            }
                        }
                    )
                }
            }
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
            .graphicsLayer(clip = false) // Allow actions to draw outside if parent row allows
    ) {
        val guildId = message.guild_id ?: navigationStore.selectedGuild?.id

        // Background and Content Layer
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .background(
                    color = when {
                        isHighlighted -> MaterialTheme.colorScheme.primary.copy(alpha = 0.2f)
                        isMentioned -> MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.4f)
                        message.isDeleted -> MaterialTheme.colorScheme.error.copy(alpha = 0.15f)
                        isHovered || showReactionPicker -> MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.12f)
                        message.sendError != null -> MaterialTheme.colorScheme.error.copy(alpha = 0.1f)
                        else -> Color.Transparent
                    }
                )
                .alpha(messageAlpha)
                .animateContentSize()
                .graphicsLayer(clip = false)
        ) {
            if (isMentioned || message.isDeleted) {
                val barColor = if (message.isDeleted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                Spacer(
                    modifier = Modifier
                        .matchParentSize()
                        .drawBehind {
                            drawRect(
                                color = barColor,
                                size = androidx.compose.ui.geometry.Size(2.dp.toPx(), size.height)
                            )
                        }
                )
            }

            ContextMenu(
                items = contextMenuItems,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 8.dp, vertical = if (isInline) 1.5.dp else 8.dp)
            ) {
                Column(modifier = Modifier.fillMaxWidth()) {
                    if (message.referenced_message != null) {
                        ReplyBar(message.referenced_message)
                    }
                    
                    if (message.interaction != null) {
                        InteractionHeader(message.interaction)
                    }

                    Row(modifier = Modifier.fillMaxWidth()) {
                        if (!isInline && message.author != null) {
                            var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                            Box(
                                modifier = Modifier
                                    .size(40.dp)
                                    .onGloballyPositioned { avatarPosition = it.positionInRoot() }
                            ) {
                                UserAvatar(
                                    user = message.author,
                                    size = 40.dp,
                                    modifier = Modifier.clickable { profileStore.showProfile(message.author.id, guildId, avatarPosition) }
                                )
                            }
                        } else {
                            Spacer(modifier = Modifier.width(40.dp))
                        }
                        
                        val roleColor by remember(message, navigationStore.selectedGuild) {
                            derivedStateOf {
                                val guild = navigationStore.selectedGuild ?: return@derivedStateOf Color.Unspecified
                                val authorId = message.author?.id ?: return@derivedStateOf Color.Unspecified
                                val member = message.member ?: userStore.getMember(guild.id, authorId) ?: return@derivedStateOf Color.Unspecified
                                val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                                val highestRole = memberRoles.maxByOrNull { it.position }
                                if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
                            }
                        }
                        
                        val displayColor = if (roleColor == Color.Unspecified) MaterialTheme.colorScheme.onSurface else roleColor

                        Spacer(modifier = Modifier.width(8.dp))
                        
                        Column {
                            if (!isInline && message.author != null) {
                                val isDm = message.guild_id == null && navigationStore.selectedGuild == null
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier.padding(bottom = 2.dp)
                                ) {
                                    var namePosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
                                    UsernameView(
                                        name = message.member?.nick ?: message.author.global_name ?: message.author.username ?: "Unknown User",
                                        style = message.member?.display_name_styles ?: message.author.display_name_styles,
                                        baseStyle = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold),
                                        color = if (isDm) Color.White else displayColor,
                                        modifier = Modifier
                                            .onGloballyPositioned { namePosition = it.positionInRoot() }
                                            .clickable { profileStore.showProfile(message.author.id, guildId, namePosition) },
                                        ignoreEffects = !isHovered,
                                        ignoreColors = if (isDm) !isHovered else true
                                    )
                                    me.lampu.lampcord.shared.ui.components.CustomBadgesView(message.author.id)
                                    message.author.primary_guild?.let {
                                        Spacer(Modifier.width(4.dp))
                                        ClanTagView(it)
                                    }
                                    UserTagView(message.author, modifier = Modifier.padding(start = 4.dp))
                                    Spacer(modifier = Modifier.width(8.dp))
                                    MessageTimestamp(
                                        timestamp = message.timestamp,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                                    )
                                    
                                    if (message.isDeleted) {
                                        Spacer(Modifier.width(8.dp))
                                        Text(
                                            "(deleted)",
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.error,
                                            fontWeight = FontWeight.Bold
                                        )
                                    }
                                }
                            }
                            
                            val hasSnapshots = !message.message_snapshots.isNullOrEmpty()
                            val snapshotContent = message.message_snapshots?.firstOrNull()?.message?.content
                            // Logic: If forwarded, ONLY show outer content if it's not a duplicate of original message content
                            val isDuplicateForward = hasSnapshots && message.content.trim() == snapshotContent?.trim()
                            val shouldShowContent = !hasSnapshots || (message.content.isNotEmpty() && !isDuplicateForward)

                            if (shouldShowContent) {
                                Box(modifier = Modifier.fillMaxWidth()) {
                                    Column {
                                        if (message.oldContent != null) {
                                            Text(
                                                text = message.oldContent,
                                                style = MaterialTheme.typography.bodyMedium,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                                                modifier = Modifier.padding(bottom = 2.dp),
                                                textDecoration = androidx.compose.ui.text.style.TextDecoration.LineThrough
                                            )
                                        }
                                        DiscordMarkdownText(
                                            content = message.content,
                                            style = MaterialTheme.typography.bodyMedium,
                                            color = if (message.isDeleted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface
                                        )
                                    }
                                }
                            }

                            if (message.sendError != null) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                                ) {
                                    Icon(
                                        Icons.Rounded.Error,
                                        null,
                                        tint = MaterialTheme.colorScheme.error,
                                        modifier = Modifier.size(14.dp)
                                    )
                                    Text(
                                        text = message.sendError,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.error
                                    )
                                    Text(
                                        text = "Retry",
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.primary,
                                        modifier = Modifier.clickable { messageStore.retryMessage(message) }
                                    )
                                    Text(
                                        text = "Delete",
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.error,
                                        modifier = Modifier.clickable { messageStore.deletePendingMessage(message) }
                                    )
                                }
                            }

                            MessageBody(message)

                            message.message_snapshots?.firstOrNull()?.let {
                                ForwardedMessage(message)
                            }

                            if (message.thread != null) {
                                ThreadStarterBar(message.thread)
                            }

                            ReactionsView(message)
                        }
                    }
                }
            }
        }

        // Overlay Layer (Action Buttons)
        val isMe = message.author?.id == currentUser?.id
        val actions = remember(message, isMe) {
            val list = mutableListOf(
                Triple(Icons.Filled.AddReaction, "Add Reaction") { showReactionPicker = true },
                Triple(Icons.Rounded.Reply, "Reply") { messageStore.replyingTo = message },
                Triple(Icons.Filled.Forward, "Forward") {
                    navigationStore.forwardingMessage = message
                }
            )
            if (isMe) {
                list.add(Triple(Icons.Filled.Edit, "Edit") {
                    messageStore.editingMessage = message
                })
            }
            list.add(Triple(Icons.Filled.Tag, "Create Thread") { showCreateThreadDialog = true })
            list.add(Triple(Icons.Filled.MoreHoriz, "More") { /* TODO */ })
            list
        }

        val showActions = isHovered || showReactionPicker
        val actionAlpha by animateFloatAsState(
            targetValue = if (showActions) 1f else 0f,
            animationSpec = tween(durationMillis = 150),
            label = "actionAlpha"
        )
        val actionScale by animateFloatAsState(
            targetValue = if (showActions) 1f else 0.92f,
            animationSpec = spring(dampingRatio = 0.7f, stiffness = 400f),
            label = "actionScale"
        )

        if (actionAlpha > 0f) {
            Box(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(end = 16.dp)
                    .zIndex(2f)
                    .graphicsLayer {
                        alpha = actionAlpha
                        scaleX = actionScale
                        scaleY = actionScale
                        clip = false
                        translationY = -16.dp.toPx() // Explicitly move up in graphics layer
                    }
                    .layout { measurable, constraints ->
                        val placeable = measurable.measure(constraints)
                        // Occupy 0 height in layout pass so it doesn't affect message spacing
                        layout(placeable.width, 0) {
                            placeable.placeRelative(0, 0)
                        }
                    }
            ) {
                ButtonGroup(
                    modifier = Modifier.height(32.dp),
                    overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                    horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                ) {
                    actions.forEachIndexed { index, (icon, label, onClick) ->
                        customItem(
                            buttonGroupContent = {
                                val shapes = when (index) {
                                    0 -> ButtonGroupDefaults.connectedLeadingButtonShapes()
                                    actions.lastIndex -> ButtonGroupDefaults.connectedTrailingButtonShapes()
                                    else -> ButtonGroupDefaults.connectedMiddleButtonShapes()
                                }
                                ToggleButton(
                                    checked = false,
                                    onCheckedChange = { onClick() },
                                    shapes = shapes,
                                    colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                    contentPadding = PaddingValues(0.dp),
                                    modifier = Modifier.fillMaxSize()
                                ) {
                                    Icon(icon, label, modifier = Modifier.size(18.dp))
                                }
                            },
                            menuContent = {
                                DropdownMenuItem(
                                    text = { Text(label) },
                                    onClick = { onClick() },
                                    leadingIcon = { Icon(icon, null) }
                                )
                            }
                        )
                    }
                }
            }
        }

        if (showReactionPicker) {
            Popup(
                alignment = Alignment.TopEnd,
                offset = IntOffset(0, (-500).dp.value.toInt()), 
                onDismissRequest = { showReactionPicker = false },
                properties = PopupProperties(focusable = true)
            ) {
                EmojiPicker(
                    userStore = userStore,
                    guildStore = guildStore,
                    navigationStore = navigationStore
                ) { emoji ->
                    val emojiStr = if (emoji.id != null) "${emoji.name}:${emoji.id}" else emoji.name ?: ""
                    scope.launch {
                        discordClient.addReaction(message.channel_id, message.id, emojiStr)
                    }
                    showReactionPicker = false
                }
            }
        }

        if (showDeleteDialog) {
            DeleteMessageDialog(
                onDismiss = { showDeleteDialog = false },
                onConfirm = {
                    messageStore.deleteMessage(message)
                    showDeleteDialog = false
                }
            )
        }

        if (showCreateThreadDialog) {
            CreateThreadDialog(
                onDismiss = { showCreateThreadDialog = false },
                onConfirm = { name ->
                    scope.launch {
                        val thread = discordClient.createThreadFromMessage(message.channel_id, message.id, name)
                        if (thread != null) {
                            guildStore.handleChannelCreateOrUpdate(thread)
                            navigationStore.selectThread(thread)
                            showCreateThreadDialog = false
                        }
                    }
                }
            )
        }
    }
}

@Composable
fun CreateThreadDialog(
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit
) {
    var name by remember { mutableStateOf("") }
    
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.fillMaxWidth().padding(16.dp)
        ) {
            Column(modifier = Modifier.padding(24.dp)) {
                Text(
                    text = "Create Thread",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                
                Spacer(Modifier.height(16.dp))
                
                TextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Thread Name") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                
                Spacer(Modifier.height(24.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("Cancel")
                    }
                    Spacer(Modifier.width(8.dp))
                    Button(
                        onClick = { if (name.isNotBlank()) onConfirm(name) },
                        enabled = name.isNotBlank()
                    ) {
                        Text("Create")
                    }
                }
            }
        }
    }
}
