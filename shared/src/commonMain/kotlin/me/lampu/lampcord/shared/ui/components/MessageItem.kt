@file:OptIn(ExperimentalFoundationApi::class, ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.ui.components.messagebody.MessageBody
import me.lampu.lampcord.shared.ui.components.messagebody.ReactionsView

@Composable
fun InteractionHeader(interaction: me.lampu.lampcord.shared.model.MessageInteraction) {
    Row(
        modifier = Modifier
            .padding(start = 8.dp, bottom = 4.dp)
            .height(24.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        androidx.compose.foundation.Canvas(
            modifier = Modifier
                .width(36.dp) 
                .fillMaxHeight()
        ) {
            val cornerRadius = 8.dp.toPx()
            val gutterX = 20.dp.toPx()
            val targetY = size.height / 2f
            
            val path = androidx.compose.ui.graphics.Path().apply {
                moveTo(gutterX, size.height + 8.dp.toPx()) 
                lineTo(gutterX, targetY + cornerRadius)
                quadraticTo(
                    gutterX, targetY,
                    gutterX + cornerRadius, targetY
                )
                lineTo(size.width, targetY)
            }

            drawPath(
                path = path,
                color = Color(0xFF4E5058),
                style = androidx.compose.ui.graphics.drawscope.Stroke(
                    width = 1.5.dp.toPx(),
                    cap = androidx.compose.ui.graphics.StrokeCap.Round
                )
            )
        }

        Spacer(modifier = Modifier.width(4.dp))

        val avatarUrl = interaction.user.avatar?.let {
            "https://cdn.discordapp.com/avatars/${interaction.user.id}/$it.png?size=48"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = interaction.user.global_name ?: interaction.user.username,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
        )

        Spacer(modifier = Modifier.width(4.dp))
        
        Text(
            text = "used",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f)
        )

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = "/${interaction.name}",
            style = MaterialTheme.typography.labelSmall,
            color = Color(0xFF00A8FC),
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
fun ReplyBar(referencedMessage: Message, chatState: ChatState) {
    Row(
        modifier = Modifier
            .padding(start = 8.dp, bottom = 4.dp)
            .height(24.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        androidx.compose.foundation.Canvas(
            modifier = Modifier
                .width(36.dp) 
                .fillMaxHeight()
        ) {
            val cornerRadius = 8.dp.toPx()
            val gutterX = 20.dp.toPx()
            val targetY = size.height / 2f
            
            val path = androidx.compose.ui.graphics.Path().apply {
                moveTo(gutterX, size.height + 8.dp.toPx()) 
                lineTo(gutterX, targetY + cornerRadius)
                quadraticTo(
                    gutterX, targetY,
                    gutterX + cornerRadius, targetY
                )
                lineTo(size.width, targetY)
            }

            drawPath(
                path = path,
                color = Color(0xFF4E5058),
                style = androidx.compose.ui.graphics.drawscope.Stroke(
                    width = 1.5.dp.toPx(),
                    cap = androidx.compose.ui.graphics.StrokeCap.Round
                )
            )
        }

        Spacer(modifier = Modifier.width(4.dp))

        val avatarUrl = referencedMessage.member?.avatar?.let {
            "https://cdn.discordapp.com/guilds/${referencedMessage.guild_id ?: chatState.selectedGuild?.id}/users/${referencedMessage.author.id}/avatars/$it.png?size=48"
        } ?: referencedMessage.author.avatar?.let {
            "https://cdn.discordapp.com/avatars/${referencedMessage.author.id}/$it.png?size=48"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        val roleColor by remember(referencedMessage, chatState.selectedGuild) {
            derivedStateOf {
                val guild = chatState.selectedGuild ?: return@derivedStateOf Color.White
                val member = referencedMessage.member ?: chatState.getMember(guild.id, referencedMessage.author.id) ?: return@derivedStateOf Color.White
                val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                val highestRole = memberRoles.maxByOrNull { it.position }
                if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.White
            }
        }

        Text(
            text = referencedMessage.author.global_name ?: referencedMessage.author.username,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = roleColor.copy(alpha = 0.8f)
        )

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = referencedMessage.content,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f),
            maxLines = 1,
            overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MessageTimestamp(timestamp: String, style: androidx.compose.ui.text.TextStyle, color: Color) {
    val fullDate = remember(timestamp) { DateTimeUtils.formatFullDate(timestamp) }
    val displayDate = remember(timestamp) { DateTimeUtils.formatTimestamp(timestamp) }
    
    TooltipBox(
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(
            positioning = TooltipAnchorPosition.Above
        ),
        tooltip = {
            RichTooltip(
                caretShape = TooltipDefaults.caretShape()
            ) {
                val parts = fullDate.split(" at ")
                if (parts.size == 2) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(parts[0])
                        Text(parts[1])
                    }
                } else {
                    Text(fullDate)
            }
        }
        },
        state = rememberTooltipState()
    ) {
        Text(
            text = displayDate,
            style = style,
            color = color
        )
    }
}

@Composable
fun SystemMessage(message: Message, chatState: ChatState) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp, horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        val (icon, iconTint, text) = when (message.type) {
            1 -> Triple(Icons.Filled.PersonAdd, Color(0xFF23A559), "${message.author.username} added a recipient.")
            2 -> Triple(Icons.Filled.PersonRemove, Color(0xFFF23F43), "${message.author.username} removed a recipient.")
            6 -> Triple(Icons.Filled.PushPin, MaterialTheme.colorScheme.primary, "${message.author.username} pinned a message to this channel.")
            7 -> Triple(Icons.AutoMirrored.Filled.ArrowForward, Color(0xFF23A559), "${message.author.global_name ?: message.author.username} joined the server.")
            8, 9, 10, 11 -> Triple(Icons.Filled.RocketLaunch, Color(0xFFFF73FA), "${message.author.global_name ?: message.author.username} just boosted the server!")
            18 -> Triple(Icons.Filled.Tag, MaterialTheme.colorScheme.primary, "${message.author.global_name ?: message.author.username} started a thread.")
            else -> Triple(Icons.Filled.Info, MaterialTheme.colorScheme.onSurfaceVariant, "System message (Type ${message.type})")
        }

        if (message.type == 7) {
            Box(modifier = Modifier.width(40.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            var profilePosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
            Text(
                text = message.author.global_name ?: message.author.username,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                modifier = Modifier
                    .onGloballyPositioned { profilePosition = it.positionInRoot() }
                    .clickable { chatState.showProfile(message.author.id, profilePosition) }
            )
            
            Spacer(modifier = Modifier.width(4.dp))
            
            Text(
                text = "just slid into the server.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface
            )
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        } else {
            Box(modifier = Modifier.width(40.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MessageItem(message: Message, chatState: ChatState, priorMessage: Message? = null) {
    if (message.type != null && message.type != 0 && message.type != 19 && message.type != 20) {
        SystemMessage(message, chatState)
        return
    }

    var isHovered by remember { mutableStateOf(false) }
    var showReactionPicker by remember { mutableStateOf(false) }
    
    val contextMenuItems = remember(message, chatState.currentUser) {
        if (message.isPending) {
            return@remember listOf(
                ContextMenuItem("Delete", Icons.Default.Delete) { 
                    chatState.deletePendingMessage(message)
                }
            )
        }
        
        val isMe = message.author.id == chatState.currentUser?.id
        val items = mutableListOf(
            ContextMenuItem("Add Reaction", Icons.Filled.AddReaction) { showReactionPicker = true },
            ContextMenuItem("Reply", Icons.Rounded.Reply) { chatState.replyingTo = message },
            ContextMenuItem("Forward", Icons.Filled.Forward) { chatState.forwardingMessage = message },
            ContextMenuItem("Copy Text", Icons.Filled.ContentCopy) { setClipboardText(message.content) },
            ContextMenuItem("Copy Link", Icons.Filled.Link) {
                val guildId = message.guild_id ?: chatState.selectedGuild?.id ?: "@me"
                val channelId = message.channel_id
                val messageId = message.id
                setClipboardText("https://discord.com/channels/$guildId/$channelId/$messageId")
            },
            ContextMenuItem("Mention", Icons.Outlined.AlternateEmail) {
                val current = chatState.draftMessages[message.channel_id] ?: ""
                chatState.draftMessages[message.channel_id] = "$current <@${message.author.id}> "
            }
        )
        
        if (isMe) {
            items.add(ContextMenuItem("Edit Message", Icons.Filled.Edit) { 
                chatState.editingMessage = message 
            })
        }
        
        items.add(ContextMenuItem("Pin Message", Icons.Filled.PushPin) { /* TODO */ })

        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy Message ID", Icons.Filled.Dns) { setClipboardText(message.id) })
            items.add(ContextMenuItem("Copy Author ID", Icons.Filled.Dns) { setClipboardText(message.author.id) })
        }

        items.add(ContextMenuItem("Delete Message", Icons.Filled.Delete) { /* TODO */ })
        
        items
    }

    val isMentioned by remember(message, chatState.currentUser, chatState.selectedGuild) {
        derivedStateOf { if (message.isPending) false else chatState.isMessageMentioningMe(message) }
    }
    
    val alpha by animateFloatAsState(if (message.isPending) 0.5f else 1f)

    val isInline = priorMessage != null

    Box(
        modifier = Modifier
            .fillMaxWidth()
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
            .background(
                color = when {
                    isMentioned -> MaterialTheme.colorScheme.primary.copy(alpha = 0.05f)
                    isHovered || showReactionPicker -> MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.12f)
                    message.sendError != null -> MaterialTheme.colorScheme.error.copy(alpha = 0.1f)
                    else -> Color.Transparent
                }
            )
            .alpha(alpha)
            .padding(top = if (isInline) 0.dp else 12.dp)
    ) {
        if (showReactionPicker) {
            Popup(
                alignment = Alignment.TopEnd,
                offset = IntOffset(0, (-500).dp.value.toInt()), // Simple offset for now
                onDismissRequest = { showReactionPicker = false },
                properties = PopupProperties(focusable = true)
            ) {
                EmojiPicker(chatState) { emoji ->
                    val emojiStr = if (emoji.id != null) "${emoji.name}:${emoji.id}" else emoji.name ?: ""
                    chatState.addReaction(message.channel_id, message.id, emojiStr)
                    showReactionPicker = false
                }
            }
        }

        if (isMentioned) {
            val mentionBarColor = MaterialTheme.colorScheme.primary
            Spacer(
                modifier = Modifier
                    .matchParentSize()
                    .drawBehind {
                        drawRect(
                            color = mentionBarColor,
                            size = androidx.compose.ui.geometry.Size(2.dp.toPx(), size.height)
                        )
                    }
            )
        }
        
        ContextMenu(
            items = contextMenuItems,
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 2.dp, horizontal = 12.dp)
        ) {
            Column(modifier = Modifier.fillMaxWidth()) {
                if (message.referenced_message != null) {
                    ReplyBar(message.referenced_message, chatState)
                }
                
                if (message.interaction != null) {
                    InteractionHeader(message.interaction)
                }

                Row(modifier = Modifier.fillMaxWidth()) {
                    if (!isInline) {
                        val avatarUrl = message.member?.avatar?.let {
                            "https://cdn.discordapp.com/guilds/${message.guild_id ?: chatState.selectedGuild?.id}/users/${message.author.id}/avatars/$it.png?size=160"
                        } ?: message.author.avatar?.let {
                            "https://cdn.discordapp.com/avatars/${message.author.id}/$it.png?size=160"
                        }

                        var avatarPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }

                        Surface(
                            modifier = Modifier
                                .size(40.dp)
                                .onGloballyPositioned { avatarPosition = it.positionInRoot() },
                            onClick = { chatState.showProfile(message.author.id, avatarPosition) },
                            shape = MaterialTheme.shapes.extraLarge,
                            color = Color.Transparent
                        ) {
                            if (avatarUrl != null) {
                                AsyncImage(
                                    model = avatarUrl,
                                    contentDescription = "Avatar",
                                    modifier = Modifier.fillMaxSize().clip(MaterialTheme.shapes.extraLarge),
                                    filterQuality = FilterQuality.Medium
                                )
                            } else {
                                Surface(
                                    modifier = Modifier.fillMaxSize(),
                                    shape = MaterialTheme.shapes.extraLarge,
                                    color = MaterialTheme.colorScheme.primaryContainer
                                ) {
                                    Box(contentAlignment = Alignment.Center) {
                                        Text(
                                            message.author.username.take(1).uppercase(),
                                            style = MaterialTheme.typography.titleMedium,
                                            color = MaterialTheme.colorScheme.onPrimaryContainer
                                        )
                                    }
                                }
                            }
                        }
                    } else {
                        Spacer(modifier = Modifier.width(40.dp))
                    }
                    
                    val roleColor by remember(message, chatState.selectedGuild) {
                        derivedStateOf {
                            val guild = chatState.selectedGuild ?: return@derivedStateOf Color.Unspecified
                            val member = message.member ?: chatState.getMember(guild.id, message.author.id) ?: return@derivedStateOf Color.Unspecified
                            val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                            val highestRole = memberRoles.maxByOrNull { it.position }
                            if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
                        }
                    }
                    
                    val displayColor = if (roleColor == Color.Unspecified) MaterialTheme.colorScheme.onSurface else roleColor

                    Spacer(modifier = Modifier.width(12.dp))
                    
                    Column {
                        if (!isInline) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                var namePosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
                                Text(
                                    text = message.member?.nick ?: message.author.global_name ?: message.author.username,
                                    style = MaterialTheme.typography.titleSmall,
                                    color = displayColor,
                                    modifier = Modifier
                                        .onGloballyPositioned { namePosition = it.positionInRoot() }
                                        .clickable { chatState.showProfile(message.author.id, namePosition) }
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                MessageTimestamp(
                                    timestamp = message.timestamp,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                        
                        DiscordMarkdownText(
                            content = message.content,
                            style = MaterialTheme.typography.bodyMedium.copy(fontSize = 15.sp, lineHeight = 20.sp),
                            color = MaterialTheme.colorScheme.onSurface,
                            chatState = chatState
                        )

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
                                    modifier = Modifier.clickable { chatState.retryMessage(message) }
                                )
                                Text(
                                    text = "Delete",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.error,
                                    modifier = Modifier.clickable { chatState.deletePendingMessage(message) }
                                )
                            }
                        }

                        MessageBody(message, chatState)

                        message.message_snapshots?.firstOrNull()?.let { snapshot ->
                            ForwardedMessage(message, chatState)
                        }

                        ReactionsView(message, chatState)
                    }
                }
            }
        }

        if (isHovered || showReactionPicker) {
            val isMe = message.author.id == chatState.currentUser?.id
            
            val actions = remember(message, isMe) {
                val list = mutableListOf(
                    Triple(Icons.Filled.AddReaction, "Add Reaction", { showReactionPicker = true }),
                    Triple(Icons.Rounded.Reply, "Reply", { chatState.replyingTo = message }),
                    Triple(Icons.Filled.Forward, "Forward", { chatState.forwardingMessage = message })
                )
                if (isMe) {
                    list.add(Triple(Icons.Filled.Edit, "Edit", { chatState.editingMessage = message }))
                }
                list.add(Triple(Icons.Filled.MoreHoriz, "More", { /* TODO */ }))
                list
            }

            Box(
                modifier = Modifier
                    .matchParentSize()
                    .padding(end = 16.dp)
            ) {
                ButtonGroup(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .offset(y = (-12).dp)
                        .height(32.dp)
                        .widthIn(min = 120.dp) // Ensure it doesn't squish
                        .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)),
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
    }
}
