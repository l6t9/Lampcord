@file:OptIn(ExperimentalFoundationApi::class, ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
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
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChange
import androidx.compose.ui.layout.layout
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import androidx.compose.ui.zIndex
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.state.ChannelNavigator
import me.lampu.lampcord.shared.state.EmojiStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.ReadStateStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.TapTapAction
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.EmojiPicker
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.chat.ReactionPickerSheet
import me.lampu.lampcord.shared.ui.components.ForwardedMessage
import me.lampu.lampcord.shared.ui.components.RoleIcon
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.components.messagebody.MessageBody
import me.lampu.lampcord.shared.ui.components.messagebody.ReactionsView
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.UserAvatar
import me.lampu.lampcord.shared.utils.*
import kotlin.time.Instant
import org.koin.compose.koinInject
import kotlin.math.roundToInt
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import me.lampu.lampcord.shared.ui.kit.pointerClickable
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.ui.kit.handCursor

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MessageItem(
    message: Message,
    priorMessage: Message? = null,
    nextMessage: Message? = null,
    isPreview: Boolean = false,
    messageStore: MessageStore = koinInject(),
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    messageApi: MessageApi = koinInject(),
    channelApi: ChannelApi = koinInject(),
    guildApi: GuildApi = koinInject(),
    readStateStore: ReadStateStore = koinInject()
) {
    if (message.type != null && message.type != 0 && message.type != 19 && message.type != 20) {
        SystemMessage(message)
        return
    }

    var isHovered by remember { mutableStateOf(false) }
    var showReactionPicker by remember { mutableStateOf(false) }
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showCreateThreadDialog by remember { mutableStateOf(false) }
    var showMessageContextMenu by remember { mutableStateOf(false) }
    var messageLongPressRequest by remember { mutableStateOf(0) }
    
    val currentUser by userStore.currentUser.collectAsState()
    val members by userStore.members.collectAsState()
    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()

    val guildId = remember(message, navigationStore.selectedGuild) {
        message.guild_id ?: navigationStore.selectedGuild?.id
    }

    val authorId = message.author?.id

    val author = remember(message.author, currentUser) {
        if (authorId == currentUser?.id) currentUser ?: message.author else message.author
    }

    val member = remember(message.member, authorId, members, guildId) {
        val cached = if (guildId != null && authorId != null) members[guildId]?.get(authorId) else null
        message.member ?: cached
    }

    LaunchedEffect(authorId, guildId) {
        if (authorId != null && guildId != null && message.member == null) {
            val existing = userStore.getMember(guildId, authorId)
            if (existing == null) {
                try {
                    val fetched = guildApi.getGuildMember(guildId, authorId)
                    if (fetched != null) {
                        userStore.cacheMember(guildId, authorId, fetched)
                    }
                } catch (e: Exception) { }
            }
        }
    }

    val currentMember = remember(navigationStore.selectedGuild, currentUser, members) {
        val gId = navigationStore.selectedGuild?.id ?: return@remember null
        val uId = currentUser?.id ?: return@remember null
        userStore.getMember(gId, uId)
    }

    val cAuthor = author
    val cMember = member

    val isMe = message.author?.id == currentUser?.id
    val guild = navigationStore.selectedGuild
    val channel = navigationStore.selectedChannel
    val canAddReaction = remember(guild, currentMember, channel, currentUser?.id) {
        if (guild == null || currentMember == null) true
        else PermissionHelper.hasPermission(currentMember, guild, channel, Permission.ADD_REACTIONS, currentUser?.id)
    }
    val canManageMessages = if (guild == null || currentMember == null) false
        else PermissionHelper.hasPermission(currentMember, guild, channel, Permission.MANAGE_MESSAGES, currentUser?.id)


    val contextMenuItems = remember(message, currentUser, userSettings, priorMessage, currentMember, canAddReaction, cAuthor) {
        if (message.isPending) {
            return@remember listOf(
                ContextMenuItem("Delete", Icons.Default.Delete, onClick = { messageStore.deletePendingMessage(message) }, group = "Destructive")
            )
        }

        val canManageThreads = if (guild == null || currentMember == null) false
            else PermissionHelper.hasPermission(currentMember, guild, channel, Permission.MANAGE_THREADS, currentUser?.id)

        val items = mutableListOf<ContextMenuItem>()
        
        if (isMe) {
            items.add(ContextMenuItem("Edit Message", Icons.Filled.Edit, onClick = { messageStore.editingMessage = message }, group = "Primary"))
        }

        items.add(ContextMenuItem("Reply", Icons.Rounded.Reply, onClick = { messageStore.replyingTo = message }, group = "Primary"))
        items.add(ContextMenuItem("Forward", Icons.Filled.Forward, onClick = { navigationStore.forwardingMessage = message }, group = "Primary"))
        
        cAuthor?.let { a ->
            items.add(ContextMenuItem("Profile", Icons.Filled.AccountCircle, onClick = { 
                profileStore.showProfile(a.id, guildId) 
            }, group = "Primary"))
        }

        if (canManageThreads) {
            items.add(ContextMenuItem("Create Thread", Icons.Filled.Tag, onClick = { showCreateThreadDialog = true }, group = "Primary"))
        }

        items.add(ContextMenuItem("Mark Unread", Icons.Filled.VisibilityOff, onClick = {
            val targetId = priorMessage?.id ?: message.id
            scope.launch {
                channelApi.ackMessage(message.channel_id, targetId)
                readStateStore.ackMessage(message.channel_id, targetId)
            }
        }, group = "Secondary"))

        if (canManageMessages) {
            items.add(ContextMenuItem(if (message.pinned) "Unpin Message" else "Pin Message", Icons.Filled.PushPin, onClick = {
                if (message.pinned) messageStore.unpinMessage(message) else messageStore.pinMessage(message)
            }, group = "Secondary"))
        }

        items.add(ContextMenuItem("Copy Text", Icons.Filled.ContentCopy, onClick = { setClipboardText(message.content) }, group = "Content"))
        items.add(ContextMenuItem("Copy Link", Icons.Filled.Link, onClick = {
            val guildId = message.guild_id ?: navigationStore.selectedGuild?.id ?: "@me"
            val channelId = message.channel_id
            val messageId = message.id
            setClipboardText("https://discord.com/channels/$guildId/$channelId/$messageId")
        }, group = "Content"))

        if (message.attachments.isNotEmpty()) {
            items.add(ContextMenuItem("Save Files", Icons.Filled.Download, onClick = {
                scope.launch {
                    message.attachments.forEach { attachment ->
                        val url = attachment.url.ifBlank { attachment.proxy_url }
                        downloadToDownloads(url, attachment.filename)
                    }
                    showToast("Downloads started")
                }
            }, group = "Content"))
        }

        if (isMe || canManageMessages) {
            items.add(ContextMenuItem("Delete Message", Icons.Filled.Delete, color = Color.Red, onClick = { showDeleteDialog = true }, group = "Destructive"))
        }

        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy Message ID", Icons.Filled.Dns, onClick = { setClipboardText(message.id) }, group = "Developer"))
            items.add(ContextMenuItem("Copy Author ID", Icons.Filled.Dns, onClick = { setClipboardText(message.author?.id ?: "") }, group = "Developer"))
        }

        items
    }

    val isMentioned = remember(message, currentUser, currentMember) { if (message.isPending) false else messageStore.isMessageMentioningMe(message, currentUser, currentMember) }

    val reduceMotion = Settings.shared.reduceMotion
    val messageAlpha by animateFloatAsState(
        if (message.isPending) 0.5f else 1f,
        animationSpec = if (reduceMotion) snap() else spring()
    )

    val isInline = priorMessage != null
    val useBubbles = settingsStore.chatBubbles
    val spacingMode = settingsStore.messageSpacingMode
    val isCompact = settingsStore.compactMode

    val hasNextSameUser = remember(message, nextMessage) {
        if (nextMessage == null) return@remember false
        if (nextMessage.author?.id != message.author?.id) return@remember false
        if (nextMessage.referenced_message != null) return@remember false
        val currentType = message.type ?: 0
        val nextType = nextMessage.type ?: 0
        if ((currentType != 0 && currentType != 19) || (nextType != 0 && nextType != 19)) return@remember false
        
        try {
            val currentTs = Instant.parse(message.timestamp)
            val nextTs = Instant.parse(nextMessage.timestamp)
            (nextTs - currentTs) < 7.minutes
        } catch (e: Exception) {
            false
        }
    }

    val bubbleShape = remember(isInline, hasNextSameUser) {
        val radiusOuter = 18.dp
        val radiusInner = 4.dp
        when {
            !isInline && !hasNextSameUser -> RoundedCornerShape(radiusOuter)
            !isInline && hasNextSameUser -> RoundedCornerShape(topStart = radiusOuter, topEnd = radiusOuter, bottomStart = radiusInner, bottomEnd = radiusInner)
            isInline && hasNextSameUser -> RoundedCornerShape(radiusInner)
            else -> RoundedCornerShape(topStart = radiusInner, topEnd = radiusInner, bottomStart = radiusOuter, bottomEnd = radiusOuter)
        }
    }
    val isHighlighted = messageStore.highlightedMessageId == message.id
    
    val tapTapMode = remember { me.lampu.lampcord.shared.settings.Settings.shared.tapTap }
    val gestureMode = remember { me.lampu.lampcord.shared.settings.Settings.shared.swipeToReplyEnabled }
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
            .zIndex(if (isHovered || showReactionPicker) 10f else 1f)
            .offset { IntOffset(offsetX.roundToInt(), 0) }
            .pointerInput(message.id, gestureMode) {
                if (!isPreview && gestureMode) {
                    // Observe on the Final pass so the parent panel's draggable gets first claim at the
                    // Main pass. Without this the row consumes the drag and swipe-to-channel-list dies
                    // whenever swipe-to-reply is enabled.
                    awaitEachGesture {
                        val down = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Final)
                        if (down.isConsumed) return@awaitEachGesture

                        var pointer = down.id
                        var travelled = 0f
                        var locked = false

                        while (true) {
                            val event = awaitPointerEvent(PointerEventPass.Final)
                            val change = event.changes.firstOrNull { it.id == pointer } ?: break
                            if (!change.pressed || change.isConsumed) {
                                if (locked && offsetX < -80f) {
                                    messageStore.replyingTo = message
                                }
                                offsetX = 0f
                                break
                            }

                            val delta = change.positionChange().x
                            travelled += kotlin.math.abs(delta)

                            if (!locked && travelled > viewConfiguration.touchSlop) {
                                // If another handler already claimed this gesture, stand down.
                                if (change.isConsumed || event.changes.any { it.isConsumed }) {
                                    return@awaitEachGesture
                                }
                                if (kotlin.math.abs(delta) > kotlin.math.abs(change.positionChange().y)) {
                                    locked = true
                                } else {
                                    break
                                }
                            }

                            if (locked) {
                                offsetX = (offsetX + delta).coerceIn(-150f, 0f)
                                change.consume()
                            }
                        }
                    }
                }
            }
            .pointerInput(message.id, tapTapMode) {
                if (!isPreview && tapTapMode != me.lampu.lampcord.shared.settings.TapTapAction.DISABLED) {
                    detectTapGestures(
                        onLongPress = if (getPlatformName() == "android") {
                            { _ -> messageLongPressRequest++ }
                        } else null,
                        onDoubleTap = {
                            val isMe = message.author?.id == currentUser?.id

                            when (tapTapMode) {
                                TapTapAction.REPLY_OR_EDIT -> {
                                    if (isMe) {
                                        messageStore.editingMessage = message
                                    } else {
                                        messageStore.replyingTo = message
                                    }
                                }
                                TapTapAction.EMOJI_PICKER -> if (canAddReaction) { showReactionPicker = true }
                                TapTapAction.DELETE_MESSAGE -> if (isMe || canManageMessages) { messageStore.deleteMessage(message) }

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
                .animateContentSize(animationSpec = if (reduceMotion) snap() else spring())
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

            val topPadding = if (useBubbles) {
                if (isInline) 1.5.dp else 6.dp
            } else {
                when (spacingMode) {
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> 0.dp
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> if (isInline) 0.dp else 6.dp
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> if (isInline) 2.dp else 10.dp
                }
            }
            val bottomPadding = if (useBubbles) {
                if (hasNextSameUser) 1.5.dp else 6.dp
            } else {
                when (spacingMode) {
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> 0.dp
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> if (hasNextSameUser) 0.dp else 6.dp
                    me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> if (hasNextSameUser) 2.dp else 10.dp
                }
            }

            ContextMenu(
                items = contextMenuItems,
                enabled = !isPreview,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 8.dp, end = 8.dp, top = topPadding, bottom = bottomPadding),
                header = if (settingsStore.showContextMenuMessage) {
                    {
                        Column(modifier = Modifier.padding(horizontal = 4.dp)) {
                            Text(
                                text = cAuthor?.global_name ?: cAuthor?.username ?: "Unknown",
                                style = MaterialTheme.typography.titleLarge,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                            Text(
                                text = message.content,
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                } else null,
                reactions = if (canAddReaction) {
                    { onDismiss ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            val commonReactions = emojiStore.frequentEmojis.take(5)
                            commonReactions.forEach { emojiKey ->
                                IconButton(
                                    onClick = {
                                        messageStore.toggleReaction(message, emojiFromKey(emojiKey))
                                        onDismiss()
                                    },
                                    modifier = Modifier
                                        .size(48.dp)
                                        .background(MaterialTheme.colorScheme.surfaceContainerHighest, CircleShape)
                                ) {
                                    val url = remember(emojiKey) { emojiFromKey(emojiKey).getDisplayUrl() }
                                    AsyncImage(
                                        model = url,
                                        contentDescription = emojiKey,
                                        modifier = Modifier.size(24.dp)
                                    )
                                }
                            }
                            IconButton(
                                onClick = { 
                                    showReactionPicker = true
                                    onDismiss()
                                },
                                modifier = Modifier
                                    .size(48.dp)
                                    .background(MaterialTheme.colorScheme.surfaceContainerHighest, CircleShape)
                            ) {
                                Icon(Icons.Filled.AddReaction, null, modifier = Modifier.size(24.dp))
                            }
                        }
                    }
                } else null,
                respectChildGestures = true,
                openRequest = messageLongPressRequest
            ) {
                val guilds by guildStore.guilds.collectAsState()
                val roleData = remember(message, guilds, navigationStore.selectedGuild, members, cAuthor, cMember) {
                    val guild = (if (message.guild_id != null) guilds.find { it.id == message.guild_id } else null)
                        ?: navigationStore.selectedGuild
                        ?: return@remember null

                    val authorId = cAuthor?.id ?: return@remember null
                    val m = cMember ?: userStore.getMember(guild.id, authorId) ?: return@remember null
                    val colorRole = m.getRoleColorRole(guild)

                    if (colorRole != null) {
                        val primaryInt = colorRole.colors?.primary_color ?: colorRole.color
                        val gradient = if (colorRole.colors?.secondary_color != null) {
                            listOfNotNull(
                                Color(primaryInt or 0xFF000000.toInt()),
                                Color(colorRole.colors.secondary_color or 0xFF000000.toInt()),
                                colorRole.colors.tertiary_color?.let { Color(it or 0xFF000000.toInt()) }
                            )
                        } else null
                        val color = if (primaryInt != 0) Color(primaryInt or 0xFF000000.toInt()) else Color.Unspecified
                        color to gradient
                    } else null
                }

                val roleColor = roleData?.first ?: Color.Unspecified
                val roleGradient = roleData?.second
                
                val displayColor = if (roleColor == Color.Unspecified) MaterialTheme.colorScheme.onSurface else roleColor

                @Composable
                fun MessageMainContent() {
                    if (!isInline && cAuthor != null) {
                        val isDm = message.guild_id == null && navigationStore.selectedGuild == null
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(bottom = 2.dp)
                        ) {
                            var namePosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
                            UsernameView(
                                name = cMember?.nick ?: cAuthor.global_name ?: cAuthor.username ?: "Unknown User",
                                style = cMember?.display_name_styles ?: cAuthor.display_name_styles,
                                baseStyle = if (useBubbles) MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold)
                                else MaterialTheme.typography.titleSmall.copy(
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 15.sp
                                ),
                                color = if (isDm) Color.White else displayColor,
                                roleGradient = roleGradient,
                                overflow = TextOverflow.Clip,
                                modifier = Modifier
                                    .onGloballyPositioned { namePosition = it.positionInRoot() }
                                    .clickableCursor(enabled = !isPreview) { profileStore.showProfile(cAuthor.id, guildId, namePosition) },
                                ignoreEffects = !isHovered,
                                ignoreColors = if (isDm) !isHovered else false
                            )
                            cAuthor.primary_guild?.let {
                                Spacer(Modifier.width(4.dp))
                                ClanTagView(it, modifier = Modifier.weight(1f, fill = false))
                            }
                            val guild = navigationStore.selectedGuild
                            val roleIcon = cMember?.getRoleIcon(guild) ?: userStore.getMember(guild?.id ?: "", cAuthor.id)?.getRoleIcon(guild)
                            if (roleIcon != null) {
                                RoleIcon(roleIcon, modifier = Modifier.padding(start = 4.dp))
                            }
                            UserTagView(cAuthor, modifier = Modifier.padding(start = 4.dp))
                            Spacer(Modifier.width(8.dp))
                            MessageTimestamp(
                                timestamp = message.timestamp,
                                style = if (useBubbles) MaterialTheme.typography.labelSmall
                                else MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = if (useBubbles) 0.7f else 0.5f)
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
                    val isDuplicateForward = hasSnapshots && message.content.trim() == snapshotContent?.trim()
                    val shouldShowContent = !hasSnapshots || (message.content.isNotEmpty() && !isDuplicateForward)

                    if (shouldShowContent) {
                        Box(modifier = Modifier.fillMaxWidth()) {
                            Column {
                                if (message.oldContent != null) {
                                    Text(
                                        text = message.oldContent,
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = if (useBubbles) 0.5f else 0.4f),
                                        modifier = Modifier.padding(bottom = 1.dp),
                                        textDecoration = androidx.compose.ui.text.style.TextDecoration.LineThrough
                                    )
                                }
                                DiscordMarkdownText(
                                    content = message.content,
                                    style = if (useBubbles) MaterialTheme.typography.bodyMedium
                                    else MaterialTheme.typography.bodyLarge.copy(
                                        fontSize = 15.sp * settingsStore.chatboxFontSize,
                                        lineHeight = 20.sp * settingsStore.chatboxFontSize
                                    ),
                                    color = if (message.isDeleted) MaterialTheme.colorScheme.error else if (useBubbles) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onBackground
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
                                modifier = Modifier.clickableCursor { messageStore.retryMessage(message) }
                            )
                            Text(
                                text = "Delete",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.error,
                                modifier = Modifier.clickableCursor { messageStore.deletePendingMessage(message) }
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

                Column(modifier = Modifier.fillMaxWidth()) {
                    if (message.referenced_message != null) {
                        ReplyBar(message.referenced_message)
                    }

                    if (message.interaction != null) {
                        InteractionHeader(message.interaction, guildId = guildId)
                    }

                    if (isCompact) {
                        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.Top) {
                            if (cAuthor != null) {
                                var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                                Box(
                                    modifier = Modifier
                                        .size(20.dp)
                                        .padding(top = 2.dp)
                                        .onGloballyPositioned { avatarPosition = it.positionInRoot() }
                                ) {
                                    UserAvatar(
                                        user = cAuthor,
                                        size = 20.dp,
                                        guildId = guildId,
                                        memberAvatar = cMember?.avatar,
                                        decorationData = cMember?.avatar_decoration_data,
                                        modifier = Modifier.pointerClickable(enabled = !isPreview) { profileStore.showProfile(cAuthor.id, guildId, avatarPosition) },
                                        isHovered = isHovered
                                    )
                                }
                            } else {
                                Spacer(modifier = Modifier.width(20.dp))
                            }

                            Spacer(modifier = Modifier.width(6.dp))

                            Column(modifier = Modifier.weight(1f)) {
                                androidx.compose.foundation.layout.FlowRow(
                                    modifier = Modifier.fillMaxWidth(),
                                    verticalArrangement = Arrangement.Center
                                ) {
                                    if (cAuthor != null) {
                                        val isDm = message.guild_id == null && navigationStore.selectedGuild == null
                                        var namePosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                                        Row(
                                            verticalAlignment = Alignment.CenterVertically,
                                            modifier = Modifier.padding(end = 4.dp)
                                        ) {
                                            UsernameView(
                                                name = cMember?.nick ?: cAuthor.global_name ?: cAuthor.username ?: "Unknown User",
                                                style = cMember?.display_name_styles ?: cAuthor.display_name_styles,
                                                baseStyle = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
                                                color = if (isDm) Color.White else displayColor,
                                                roleGradient = roleGradient,
                                                overflow = TextOverflow.Clip,
                                                modifier = Modifier
                                                    .onGloballyPositioned { namePosition = it.positionInRoot() }
                                                    .clickableCursor { profileStore.showProfile(cAuthor.id, guildId, namePosition) },
                                                ignoreEffects = !isHovered,
                                                ignoreColors = if (isDm) !isHovered else false
                                            )
                                            cAuthor.primary_guild?.let {
                                                Spacer(Modifier.width(4.dp))
                                                ClanTagView(it, modifier = Modifier.weight(1f, fill = false))
                                            }
                                            val guild = navigationStore.selectedGuild
                                            val roleIcon = cMember?.getRoleIcon(guild) ?: userStore.getMember(guild?.id ?: "", cAuthor.id)?.getRoleIcon(guild)
                                            if (roleIcon != null) {
                                                RoleIcon(roleIcon, modifier = Modifier.padding(start = 4.dp))
                                            }
                                            UserTagView(cAuthor, modifier = Modifier.padding(start = 4.dp))
                                            Text(
                                                ": ",
                                                style = MaterialTheme.typography.bodyMedium,
                                                fontWeight = FontWeight.Bold,
                                                color = if (isDm) Color.White else displayColor
                                            )
                                        }
                                    }

                                    val hasSnapshots = !message.message_snapshots.isNullOrEmpty()
                                    val snapshotContent = message.message_snapshots?.firstOrNull()?.message?.content
                                    val isDuplicateForward = hasSnapshots && message.content.trim() == snapshotContent?.trim()
                                    val shouldShowContent = !hasSnapshots || (message.content.isNotEmpty() && !isDuplicateForward)

                                    if (shouldShowContent) {
                                        if (message.oldContent != null) {
                                            Text(
                                                text = message.oldContent,
                                                style = MaterialTheme.typography.bodyMedium,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                                                modifier = Modifier.padding(end = 4.dp),
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

                                if (message.sendError != null) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                                    ) {
                                        Icon(Icons.Rounded.Error, null, tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(14.dp))
                                        Text(text = message.sendError, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.error)
                                        Text(text = "Retry", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.primary, modifier = Modifier.clickableCursor { messageStore.retryMessage(message) })
                                        Text(text = "Delete", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.error, modifier = Modifier.clickableCursor { messageStore.deletePendingMessage(message) })
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
                    } else if (useBubbles) {
                        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.Top) {
                            if (!isInline && cAuthor != null) {
                                var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                                Box(
                                    modifier = Modifier
                                        .size(40.dp)
                                        .onGloballyPositioned { avatarPosition = it.positionInRoot() }
                                ) {
                                    UserAvatar(
                                        user = cAuthor,
                                        size = 40.dp,
                                        guildId = guildId,
                                        memberAvatar = cMember?.avatar,
                                        decorationData = cMember?.avatar_decoration_data,
                                        modifier = Modifier.pointerClickable { profileStore.showProfile(cAuthor.id, guildId, avatarPosition) },
                                        isHovered = isHovered
                                    )
                                }
                            } else {
                                Spacer(modifier = Modifier.width(40.dp))
                            }

                            Spacer(modifier = Modifier.width(8.dp))

                            val bubbleColor = when {
                                isHighlighted -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
                                isMentioned -> MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
                                message.isDeleted -> MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.35f)
                                message.sendError != null -> MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.25f)
                                isHovered || showReactionPicker -> MaterialTheme.colorScheme.surfaceContainerHighest
                                else -> MaterialTheme.colorScheme.surfaceContainerHigh
                            }

                            Surface(
                                shape = bubbleShape,
                                color = bubbleColor,
                                modifier = Modifier.weight(1f)
                            ) {
                                Column(modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp)) {
                                    MessageMainContent()
                                }
                            }
                        }
                    } else {
                        Row(modifier = Modifier.fillMaxWidth()) {
                            if (!isInline && cAuthor != null) {
                                var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                                Box(
                                    modifier = Modifier
                                        .size(40.dp)
                                        .onGloballyPositioned { avatarPosition = it.positionInRoot() }
                                ) {
                                    UserAvatar(
                                        user = cAuthor,
                                        size = 40.dp,
                                        guildId = guildId,
                                        memberAvatar = cMember?.avatar,
                                        decorationData = cMember?.avatar_decoration_data,
                                        modifier = Modifier.pointerClickable { profileStore.showProfile(cAuthor.id, guildId, avatarPosition) },
                                        isHovered = isHovered
                                    )
                                }
                            } else {
                                Spacer(modifier = Modifier.width(40.dp))
                            }

                            Spacer(modifier = Modifier.width(8.dp))

                            Column {
                                MessageMainContent()
                            }
                        }
                    }
                }
            }
        }

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
            list.add(Triple(Icons.Filled.MoreHoriz, "More") { showMessageContextMenu = true })
            list
        }

        val showActions = isHovered || showReactionPicker || showMessageContextMenu
        val actionAlpha by animateFloatAsState(
            targetValue = if (showActions) 1f else 0f,
            animationSpec = if (reduceMotion) snap() else tween(durationMillis = 150),
            label = "actionAlpha"
        )
        val actionScale by animateFloatAsState(
            targetValue = if (showActions) 1f else 0.92f,
            animationSpec = if (reduceMotion) snap() else spring(dampingRatio = 0.7f, stiffness = 400f),
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
                        layout(placeable.width, 0) {
                            placeable.placeRelative(0, 0)
                        }
                    }
            ) {
                ButtonGroup(
                    modifier = Modifier
                        .height(32.dp)
                        .handCursor(),
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
                                Box {
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

                                    if (label == "More") {
                                        DropdownMenu(
                                            expanded = showMessageContextMenu,
                                            onDismissRequest = { showMessageContextMenu = false },
                                            shape = RoundedCornerShape(16.dp)
                                        ) {
                                            val groups = contextMenuItems.groupBy { it.group }.values
                                            groups.forEachIndexed { groupIndex, items ->
                                                items.forEach { item ->
                                                    DropdownMenuItem(
                                                        modifier = Modifier.handCursor(),
                                                        text = { Text(item.label) },
                                                        onClick = {
                                                            item.onClick()
                                                            showMessageContextMenu = false
                                                        },
                                                        leadingIcon = item.icon?.let { itemIcon ->
                                                            { Icon(itemIcon, null, tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant) }
                                                        }
                                                    )
                                                }
                                                if (groupIndex < groups.size - 1) HorizontalDivider()
                                            }
                                        }
                                    }
                                }
                            },
                            menuContent = {
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
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
            val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
            if (isMobile) {
                ReactionPickerSheet(
                    onDismiss = { showReactionPicker = false },
                    onEmojiSelected = { emoji ->
                        messageStore.toggleReaction(message, emoji)
                    }
                )
            } else {
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
                        messageStore.toggleReaction(message, emoji)
                        showReactionPicker = false
                    }
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
                        val thread = channelApi.createThreadFromMessage(message.channel_id, message.id, name)
                        if (thread != null) {
                            guildStore.handleChannelCreateOrUpdate(thread)
                            navigationStore.selectThread(thread, explicitlySelected = true)
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

private fun emojiFromKey(key: String): Emoji {
    val normalized = key.trim().removePrefix("<a:").removePrefix("<:").removeSuffix(">")
    val parts = normalized.split(":")
    val id = parts.lastOrNull()?.takeIf { it.isNotEmpty() && it.all(Char::isDigit) }
    return if (id != null) {
        Emoji(name = parts.dropLast(1).joinToString(":").takeIf { it.isNotBlank() }, id = id, animated = key.startsWith("<a:"))
    } else {
        val emojiName = normalized.removeSurrounding(":")
        val unicode = EmojiIndex.getCharForName(emojiName) ?: emojiName
        Emoji(name = unicode, id = null)
    }
}
