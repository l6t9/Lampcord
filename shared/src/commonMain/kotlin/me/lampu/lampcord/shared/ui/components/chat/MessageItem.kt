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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.changedToDown
import androidx.compose.ui.input.pointer.isSecondaryPressed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
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

@Composable
fun GoogleMessagesMenuItem(
    icon: ImageVector,
    label: String,
    color: Color = MaterialTheme.colorScheme.onSurface,
    onClick: () -> Unit
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        Icon(
            imageVector = icon,
            contentDescription = label,
            tint = color,
            modifier = Modifier.size(20.dp)
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
            color = color
        )
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

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MessageItem(
    message: Message,
    priorMessage: Message? = null,
    isFollowedBySameAuthor: Boolean = false,
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
    var showGoogleMessagesMenu by remember { mutableStateOf(false) }
    
    val currentUser by userStore.currentUser.collectAsState()
    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()

    val isMe = remember(message.author, currentUser) {
        val myId = currentUser?.id ?: userStore.currentUser.value?.id
        val authorId = message.author?.id
        authorId != null && myId != null && authorId == myId
    }

    val contextMenuItems = remember(message, currentUser, userSettings) {
        if (message.isPending) {
            return@remember listOf(
                ContextMenuItem("Delete", Icons.Default.Delete) { 
                    messageStore.deletePendingMessage(message)
                }
            )
        }
        
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
    
    val messageAlpha by animateFloatAsState(if (showGoogleMessagesMenu) 0f else (if (message.isPending) 0.5f else 1f))

    val isInline = priorMessage != null
    val isHighlighted = messageStore.highlightedMessageId == message.id
    
    val gestureMode = remember { me.lampu.lampcord.shared.settings.Settings.shared.chatGestures }
    var offsetX by remember { mutableFloatStateOf(0f) }

    LaunchedEffect(isHighlighted) {
        if (isHighlighted) {
            kotlinx.coroutines.delay(2000.milliseconds)
            messageStore.highlightedMessageId = null
        }
    }

    val messageStyle = settingsStore.messageStyle
    val isExpressive = messageStyle == me.lampu.lampcord.shared.settings.MessageStyle.EXPRESSIVE_BUBBLES
    val alignRight = isExpressive && isMe

    var itemPositionInRoot by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var itemSizeInRoot by remember { mutableStateOf(androidx.compose.ui.unit.IntSize.Zero) }

    val bubbleShape = remember(isInline, isFollowedBySameAuthor, isExpressive, alignRight) {
        if (!isExpressive) RoundedCornerShape(0.dp)
        else if (alignRight) {
            RoundedCornerShape(
                topStart = 18.dp,
                topEnd = if (isInline) 6.dp else 18.dp,
                bottomStart = 18.dp,
                bottomEnd = if (isFollowedBySameAuthor) 6.dp else 18.dp
            )
        } else {
            RoundedCornerShape(
                topStart = if (isInline) 6.dp else 18.dp,
                topEnd = 18.dp,
                bottomEnd = 18.dp,
                bottomStart = if (isFollowedBySameAuthor) 6.dp else 18.dp
            )
        }
    }

    val bubbleContainerColor = when {
        !isExpressive -> Color.Transparent
        isHighlighted -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
        isMentioned -> MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
        message.isDeleted -> MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.25f)
        alignRight && (isHovered || showReactionPicker) -> MaterialTheme.colorScheme.primary.copy(alpha = 0.85f)
        alignRight -> MaterialTheme.colorScheme.primary
        isHovered || showReactionPicker -> MaterialTheme.colorScheme.surfaceContainerHighest
        message.sendError != null -> MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.2f)
        else -> MaterialTheme.colorScheme.surfaceContainerHigh
    }

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

    val showActions = (isHovered || showReactionPicker) && !isExpressive
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

    val density = LocalDensity.current
    var rowWidthPx by remember { mutableStateOf(0f) }
    var bubbleLeftPx by remember { mutableStateOf(0f) }
    var bubbleRightPx by remember { mutableStateOf(0f) }
    var barWidthPx by remember { mutableStateOf(0f) }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .onGloballyPositioned { layoutCoordinates ->
                rowWidthPx = layoutCoordinates.size.width.toFloat()
            }
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
                            change.consume()
                            if (dragAmount < 0 || offsetX < 0) {
                                offsetX = (offsetX + dragAmount).coerceIn(-100f, 0f)
                            }
                        }
                    )
                }
            }
            .graphicsLayer(clip = false)
    ) {
        val guildId = message.guild_id ?: navigationStore.selectedGuild?.id

        // Background and Content Layer
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .background(
                    color = when {
                        isExpressive -> Color.Transparent
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
            if (!isExpressive && (isMentioned || message.isDeleted)) {
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

            @Composable
            fun MessageContent() {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .graphicsLayer(clip = false)
                        .onGloballyPositioned { coords ->
                            itemPositionInRoot = coords.positionInRoot()
                            itemSizeInRoot = coords.size
                        }
                ) {
                    if (message.referenced_message != null) {
                        ReplyBar(message.referenced_message)
                    }
                    
                    if (message.interaction != null) {
                        InteractionHeader(message.interaction)
                    }

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .graphicsLayer(clip = false),
                        horizontalArrangement = if (alignRight) Arrangement.End else Arrangement.Start
                    ) {
                        if (!alignRight) {
                            if (!isInline && message.author != null) {
                                var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
                                val avatarTopPadding = if (!isInline && message.author != null) 22.dp else 0.dp

                                Box(
                                    modifier = Modifier
                                        .padding(top = avatarTopPadding)
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
                            
                            Spacer(modifier = Modifier.width(8.dp))
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
                        
                        val displayColor = if (alignRight) MaterialTheme.colorScheme.onPrimary else (if (roleColor == Color.Unspecified) MaterialTheme.colorScheme.onSurface else roleColor)
                        val bubbleTextColor = if (alignRight) MaterialTheme.colorScheme.onPrimary else (if (message.isDeleted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface)

                        Box(
                            modifier = (if (isExpressive) {
                                Modifier.weight(1f, fill = false)
                            } else {
                                Modifier.weight(1f)
                            })
                            .graphicsLayer(clip = false)
                            .onGloballyPositioned { coords ->
                                val pos = coords.positionInParent()
                                bubbleLeftPx = pos.x
                                bubbleRightPx = pos.x + coords.size.width
                            }
                        ) {
                            Column(
                                horizontalAlignment = if (alignRight) Alignment.End else Alignment.Start
                            ) {
                                if (!isInline && message.author != null && !isMe) {
                                    val isDm = message.guild_id == null && navigationStore.selectedGuild == null
                                    Row(
                                        horizontalArrangement = Arrangement.Start,
                                        verticalAlignment = Alignment.CenterVertically,
                                        modifier = Modifier.padding(bottom = 2.dp, start = if (alignRight) 0.dp else 2.dp, end = if (alignRight) 2.dp else 0.dp)
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
                                        message.author.primary_guild?.let {
                                            Spacer(Modifier.width(4.dp))
                                            ClanTagView(it)
                                        }
                                        UserTagView(message.author, modifier = Modifier.padding(start = 4.dp))
                                    }
                                }

                                Box(
                                    modifier = if (isExpressive) {
                                        Modifier
                                            .background(color = bubbleContainerColor, shape = bubbleShape)
                                            .padding(horizontal = 14.dp, vertical = 10.dp)
                                    } else {
                                        Modifier.fillMaxWidth()
                                    }
                                ) {
                                    Column {
                                        val hasSnapshots = !message.message_snapshots.isNullOrEmpty()
                                        val snapshotContent = message.message_snapshots?.firstOrNull()?.message?.content
                                        val isDuplicateForward = hasSnapshots && message.content.trim() == snapshotContent?.trim()
                                        val shouldShowContent = !hasSnapshots || (message.content.isNotEmpty() && !isDuplicateForward)

                                        if (shouldShowContent) {
                                            Box {
                                                Column {
                                                    if (message.oldContent != null) {
                                                        Text(
                                                            text = message.oldContent,
                                                            style = MaterialTheme.typography.bodyMedium,
                                                            color = bubbleTextColor.copy(alpha = 0.5f),
                                                            modifier = Modifier.padding(bottom = 2.dp),
                                                            textDecoration = androidx.compose.ui.text.style.TextDecoration.LineThrough
                                                        )
                                                    }
                                                    DiscordMarkdownText(
                                                        content = message.content,
                                                        style = MaterialTheme.typography.bodyMedium,
                                                        color = bubbleTextColor
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
                                            ForwardedMessage(message, contentColor = bubbleTextColor)
                                        }

                                        if (message.thread != null) {
                                            ThreadStarterBar(message.thread)
                                        }

                                        ReactionsView(message)
                                    }
                                }

                                val showTimestampUnderneath = !isFollowedBySameAuthor || isHovered || message.isDeleted
                                if (showTimestampUnderneath) {
                                    Row(
                                        horizontalArrangement = if (alignRight) Arrangement.End else Arrangement.Start,
                                        verticalAlignment = Alignment.CenterVertically,
                                        modifier = Modifier
                                            .padding(top = 2.dp, start = if (alignRight) 0.dp else 4.dp, end = if (alignRight) 4.dp else 0.dp)
                                    ) {
                                        MessageTimestamp(
                                            timestamp = message.timestamp,
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.65f)
                                        )
                                        
                                        if (message.isDeleted) {
                                            Spacer(Modifier.width(6.dp))
                                            Text(
                                                "(deleted)",
                                                style = MaterialTheme.typography.labelSmall,
                                                color = MaterialTheme.colorScheme.error,
                                                fontWeight = FontWeight.Bold
                                            )
                                        }
                                    }
                                }

                                if (actionAlpha > 0f) {
                                    val leftBoundaryPx = bubbleLeftPx
                                    val rightBoundaryPx = if (rowWidthPx > 0f) rowWidthPx - (density.density * 12f) else Float.MAX_VALUE
                                    val barWidthOrDefaultPx = if (barWidthPx > 0f) barWidthPx else (density.density * 260f)

                                    val idealRightPx = bubbleRightPx
                                    val idealLeftPx = idealRightPx - barWidthOrDefaultPx

                                    val clampRightShift = (rightBoundaryPx - idealRightPx).coerceAtMost(0f)
                                    val rawLeftPx = idealLeftPx + clampRightShift
                                    val clampLeftShift = (leftBoundaryPx - rawLeftPx).coerceAtLeast(0f)

                                    val totalShiftPx = clampRightShift + clampLeftShift
                                    val xShiftPx = totalShiftPx.roundToInt()

                                    Popup(
                                        alignment = Alignment.TopEnd,
                                        offset = IntOffset(
                                            x = xShiftPx,
                                            y = (-14).dp.value.toInt()
                                        ),
                                        properties = PopupProperties(focusable = false, dismissOnClickOutside = false)
                                    ) {
                                        Box(
                                            modifier = Modifier
                                                .onGloballyPositioned { barWidthPx = it.size.width.toFloat() }
                                                .graphicsLayer {
                                                    alpha = actionAlpha
                                                    scaleX = actionScale
                                                    scaleY = actionScale
                                                }
                                        ) {
                                            ButtonGroup(
                                                modifier = Modifier.height(32.dp)
                                            ) {
                                                actions.forEach { (icon, label, onClick) ->
                                                    IconButton(
                                                        onClick = onClick,
                                                        modifier = Modifier.size(32.dp)
                                                    ) {
                                                        Icon(
                                                            imageVector = icon,
                                                            contentDescription = label,
                                                            modifier = Modifier.size(16.dp),
                                                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                        )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if (isExpressive) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .graphicsLayer(clip = false)
                        .padding(
                            start = 8.dp,
                            end = 8.dp,
                            top = if (isInline) 1.5.dp else 8.dp,
                            bottom = if (isFollowedBySameAuthor) 1.5.dp else 8.dp
                        )
                        .pointerInput(message.id) {
                            detectTapGestures(
                                onLongPress = { showGoogleMessagesMenu = true }
                            )
                        }
                        .pointerInput(message.id) {
                            awaitPointerEventScope {
                                while (true) {
                                    val event = awaitPointerEvent(PointerEventPass.Initial)
                                    val down = event.changes.find { it.changedToDown() }
                                    if (down != null && event.buttons.isSecondaryPressed) {
                                        showGoogleMessagesMenu = true
                                        down.consume()
                                    }
                                }
                            }
                        }
                ) {
                    MessageContent()
                }
            } else {
                ContextMenu(
                    items = contextMenuItems,
                    modifier = Modifier
                        .fillMaxWidth()
                        .graphicsLayer(clip = false)
                        .padding(
                            start = 8.dp,
                            end = 8.dp,
                            top = if (isInline) 1.5.dp else 6.dp,
                            bottom = if (isFollowedBySameAuthor) 1.5.dp else 6.dp
                        )
                ) {
                    MessageContent()
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

            if (showGoogleMessagesMenu) {
                Popup(
                    onDismissRequest = { showGoogleMessagesMenu = false },
                    properties = PopupProperties(focusable = true)
                ) {
                    var windowWidth by remember { mutableStateOf(0.dp) }
                    var windowHeight by remember { mutableStateOf(0.dp) }

                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(Color.Black.copy(alpha = 0.6f))
                            .onGloballyPositioned { coords ->
                                windowWidth = with(density) { coords.size.width.toDp() }
                                windowHeight = with(density) { coords.size.height.toDp() }
                            }
                            .clickable { showGoogleMessagesMenu = false }
                    ) {
                        if (windowHeight > 0.dp) {
                            val itemX = with(density) { itemPositionInRoot.x.toDp() }
                            val itemY = with(density) { itemPositionInRoot.y.toDp() }
                            val itemW = with(density) { itemSizeInRoot.width.toDp() }
                            val itemH = with(density) { itemSizeInRoot.height.toDp() }

                            val emojiBarH = 48.dp
                            val menuCardH = if (isMe) 340.dp else 295.dp
                            val gap = 10.dp

                            val idealBottomY = itemY + itemH + gap + menuCardH
                            val maxAllowedBottomY = windowHeight - 72.dp
                            val overflowY = (idealBottomY - maxAllowedBottomY).coerceAtLeast(0.dp)

                            val adjustedItemY = (itemY - overflowY).coerceAtLeast(16.dp + emojiBarH + gap)
                            val startY = adjustedItemY - emojiBarH - gap

                            val startX = itemX.coerceIn(16.dp, (windowWidth - itemW).coerceAtLeast(16.dp))

                            Column(
                                horizontalAlignment = if (alignRight) Alignment.End else Alignment.Start,
                                verticalArrangement = Arrangement.spacedBy(gap),
                                modifier = Modifier
                                    .offset(x = startX, y = startY)
                                    .width(if (itemW > 0.dp) itemW else 360.dp)
                                    .clickable(enabled = false) {}
                            ) {
                                Surface(
                                    shape = RoundedCornerShape(28.dp),
                                    color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                    shadowElevation = 8.dp
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)
                                    ) {
                                        val quickEmojis = listOf("👍", "❤️", "😂", "😮", "😢", "😡")
                                        quickEmojis.forEach { emojiStr ->
                                            IconButton(
                                                onClick = {
                                                    scope.launch {
                                                        discordClient.addReaction(message.channel_id, message.id, emojiStr)
                                                    }
                                                    showGoogleMessagesMenu = false
                                                },
                                                modifier = Modifier.size(36.dp)
                                            ) {
                                                Text(emojiStr, fontSize = 20.sp)
                                            }
                                        }
                                        
                                        Box(
                                            modifier = Modifier
                                                .size(36.dp)
                                                .clip(CircleShape)
                                                .background(MaterialTheme.colorScheme.primaryContainer)
                                                .clickable {
                                                    showGoogleMessagesMenu = false
                                                    showReactionPicker = true
                                                },
                                            contentAlignment = Alignment.Center
                                        ) {
                                            Icon(
                                                Icons.Filled.AddReaction,
                                                contentDescription = "Add reaction",
                                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                                modifier = Modifier.size(20.dp)
                                            )
                                        }
                                    }
                                }

                                MessageContent()

                                Surface(
                                    shape = RoundedCornerShape(20.dp),
                                    color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                    shadowElevation = 8.dp,
                                    modifier = Modifier.width(220.dp)
                                ) {
                                    Column(modifier = Modifier.padding(vertical = 8.dp)) {
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Rounded.Reply,
                                            label = "Reply"
                                        ) {
                                            messageStore.replyingTo = message
                                            showGoogleMessagesMenu = false
                                        }
                                        
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Filled.Forward,
                                            label = "Forward"
                                        ) {
                                            navigationStore.forwardingMessage = message
                                            showGoogleMessagesMenu = false
                                        }
                                        
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Filled.ContentCopy,
                                            label = "Copy"
                                        ) {
                                            setClipboardText(message.content)
                                            showGoogleMessagesMenu = false
                                        }
                                        
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Filled.PushPin,
                                            label = if (message.pinned) "Unpin" else "Pin"
                                        ) {
                                            if (message.pinned) messageStore.unpinMessage(message) else messageStore.pinMessage(message)
                                            showGoogleMessagesMenu = false
                                        }
                                        
                                        if (isMe) {
                                            GoogleMessagesMenuItem(
                                                icon = Icons.Filled.Edit,
                                                label = "Edit"
                                            ) {
                                                messageStore.editingMessage = message
                                                showGoogleMessagesMenu = false
                                            }
                                        }
                                        
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Filled.Delete,
                                            label = "Delete",
                                            color = MaterialTheme.colorScheme.error
                                        ) {
                                            showDeleteDialog = true
                                            showGoogleMessagesMenu = false
                                        }
                                        
                                        GoogleMessagesMenuItem(
                                            icon = Icons.Filled.Info,
                                            label = "Info"
                                        ) {
                                            setClipboardText(message.id)
                                            showGoogleMessagesMenu = false
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
