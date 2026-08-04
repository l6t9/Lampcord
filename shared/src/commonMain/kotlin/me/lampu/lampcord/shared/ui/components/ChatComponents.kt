package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.toTwemojiUrl
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.getClipboardFiles
import me.lampu.lampcord.shared.utils.Permission

@Composable
fun TypingIndicator(chatState: ChatState, channelId: String) {
    val typingMap = chatState.typingUsers[channelId] ?: return
    val userIds = typingMap.keys.toList()
    if (userIds.isEmpty()) return
    
    val names = userIds.map { id ->
         chatState.getMember(chatState.selectedGuild?.id ?: "", id)?.nick 
         ?: chatState.userStore.getUser(id)?.global_name 
         ?: chatState.userStore.getUser(id)?.username 
         ?: "Someone"
    }
    
    val text = when {
        names.size == 1 -> "${names[0]} is typing..."
        names.size == 2 -> "${names[0]} and ${names[1]} are typing..."
        names.size == 3 -> "${names[0]}, ${names[1]} and ${names[2]} are typing..."
        else -> "Several people are typing..."
    }

    Surface(
        modifier = Modifier.fillMaxWidth().height(24.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            TypingDots()
            Spacer(Modifier.width(8.dp))
            Text(
                text = text,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
fun TypingDots(modifier: Modifier = Modifier) {
    val infiniteTransition = rememberInfiniteTransition(label = "typingDots")
    val alpha1 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 0; 1f at 300; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha1"
    )
    val alpha2 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 150; 1f at 450; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha2"
    )
    val alpha3 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 300; 1f at 600; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha3"
    )

    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha1), CircleShape))
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha2), CircleShape))
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha3), CircleShape))
    }
}

@Composable
fun ChannelHeader(
    channel: me.lampu.lampcord.shared.model.Channel,
    chatState: ChatState
) {
    Surface(
        modifier = Modifier.fillMaxWidth().height(48.dp),
        color = MaterialTheme.colorScheme.surfaceContainer,
        shadowElevation = 0.dp,
        tonalElevation = 0.dp
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
            val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12
            
            if (isDm) {
                // DM style
                val recipient = channel.recipients?.firstOrNull()
                val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                Icon(
                    imageVector = Icons.Outlined.AlternateEmail,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = name, style = MaterialTheme.typography.titleSmall)
            } else {
                if (channel.type == 15) {
                    Icon(
                        imageVector = Icons.Outlined.Forum,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                } else if (isThread) {
                    Text(">", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) // Thread
                } else {
                    Icon(
                        imageVector = Icons.Filled.Tag,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = channel.name ?: "unnamed", style = MaterialTheme.typography.titleSmall)
            }
            
            if (channel.topic?.isNotBlank() == true) {
                Spacer(modifier = Modifier.width(12.dp))
                Text("•", color = MaterialTheme.colorScheme.onSurfaceVariant)
                Spacer(modifier = Modifier.width(12.dp))
                Text(
                    text = channel.topic,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f)
                )
            } else {
                Spacer(modifier = Modifier.weight(1f))
            }
            
            // Search bar (Far Right)
            Surface(
                modifier = Modifier.width(160.dp).height(24.dp),
                shape = RoundedCornerShape(4.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHigh
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Search", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.weight(1f))
                    Icon(Icons.Filled.Search, null, modifier = Modifier.size(14.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
fun ChatInputBar(
    channel: me.lampu.lampcord.shared.model.Channel,
    chatState: ChatState
) {
    Surface(
        color = MaterialTheme.colorScheme.surface,
        modifier = Modifier.fillMaxWidth().heightIn(min = 68.dp)
    ) {
        val canSend by remember(channel, chatState.currentUser) {
            derivedStateOf { chatState.hasPermission(Permission.SEND_MESSAGES) }
        }
        
        var messageText by remember(channel.id) { 
            mutableStateOf(chatState.draftMessages[channel.id] ?: "") 
        }
        var showFilePicker by remember { mutableStateOf(false) }

        // Update draft whenever text changes
        LaunchedEffect(messageText) {
            chatState.draftMessages[channel.id] = messageText
        }

        // Keep local messageText in sync with draft changes from outside
        LaunchedEffect(chatState.draftMessages[channel.id]) {
            val draft = chatState.draftMessages[channel.id] ?: ""
            if (draft != messageText) {
                messageText = draft
            }
        }

        // Sync messageText when editing starts
        LaunchedEffect(chatState.editingMessage) {
            chatState.editingMessage?.let {
                messageText = it.content
                chatState.pendingFiles.clear()
                chatState.replyingTo = null
            }
        }

        LaunchedEffect(chatState.replyingTo) {
            chatState.replyingTo?.let {
                chatState.editingMessage = null
            }
        }

        Column {
            TypingIndicator(chatState, channel.id)

            if (chatState.replyingTo != null || chatState.editingMessage != null) {
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = MaterialTheme.colorScheme.surfaceContainerLow
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val isEditing = chatState.editingMessage != null
                        val activeMsg = chatState.editingMessage ?: chatState.replyingTo
                        
                        Icon(
                            imageVector = if (isEditing) Icons.Filled.Edit else Icons.Rounded.Reply,
                            contentDescription = null,
                            modifier = Modifier.size(16.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = if (isEditing) "Editing message" else "Replying to ${activeMsg?.author?.global_name ?: activeMsg?.author?.username}",
                            style = MaterialTheme.typography.labelSmall,
                            modifier = Modifier.weight(1f)
                        )
                        IconButton(
                            onClick = { 
                                if (isEditing) {
                                    chatState.editingMessage = null
                                    messageText = ""
                                } else {
                                    chatState.replyingTo = null 
                                }
                            },
                            modifier = Modifier.size(24.dp)
                        ) {
                            Icon(Icons.Filled.Close, null, modifier = Modifier.size(16.dp))
                        }
                    }
                }
            }

            if (chatState.pendingFiles.isNotEmpty()) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp)
                        .height(120.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    chatState.pendingFiles.forEachIndexed { index, (name, data) ->
                        Surface(
                            modifier = Modifier.size(100.dp),
                            shape = RoundedCornerShape(8.dp),
                            color = MaterialTheme.colorScheme.surfaceContainerHigh
                        ) {
                            Box {
                                if (name.lowercase().let { it.endsWith(".png") || it.endsWith(".jpg") || it.endsWith(".jpeg") || it.endsWith(".webp") || it.endsWith(".gif") }) {
                                    AsyncImage(
                                        model = data,
                                        contentDescription = name,
                                        modifier = Modifier.fillMaxSize(),
                                        contentScale = ContentScale.Crop
                                    )
                                } else {
                                    Text(name, modifier = Modifier.align(Alignment.Center).padding(4.dp), style = MaterialTheme.typography.labelSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
                                }
                                
                                IconButton(
                                    onClick = { chatState.pendingFiles.removeAt(index) },
                                    modifier = Modifier.align(Alignment.TopEnd).size(24.dp).background(Color.Black.copy(alpha = 0.4f), CircleShape)
                                ) {
                                    Icon(Icons.Filled.Close, null, modifier = Modifier.size(16.dp), tint = Color.White)
                                }
                            }
                        }
                    }
                }
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(68.dp)
                    .padding(horizontal = 8.dp)
                    .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)),
                verticalAlignment = Alignment.CenterVertically
            ) {
                FilePicker(
                    show = showFilePicker,
                    onFileSelected = { chatState.pendingFiles.addAll(it) },
                    onDismiss = { showFilePicker = false }
                )

                IconButton(
                    onClick = { showFilePicker = true },
                    enabled = chatState.editingMessage == null && canSend,
                    colors = IconButtonDefaults.filledTonalIconButtonColors()
                ) {
                    Icon(
                        imageVector = Icons.Filled.Add,
                        contentDescription = "Add",
                        modifier = Modifier.size(20.dp)
                    )
                }
                
                val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12
                
                val placeholderText = when {
                    !canSend -> "You do not have permission to send messages in this channel."
                    isThread -> "Reply to thread..."
                    isDm -> {
                        val recipient = channel.recipients?.firstOrNull()
                        val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                        "Message @$name"
                    }
                    else -> "Message #${channel.name ?: "unnamed"}"
                }

                var showEmojiPicker by remember { mutableStateOf(false) }
                var emojiSearchQuery by remember { mutableStateOf<String?>(null) }

                BasicTextField(
                    value = messageText,
                    onValueChange = { 
                        if (canSend) {
                            messageText = it
                            val lastWord = it.substringAfterLast(' ', it)
                            if (lastWord.startsWith(':') && !lastWord.contains(' ')) {
                                emojiSearchQuery = lastWord.substring(1)
                            } else {
                                emojiSearchQuery = null
                            }
                        }
                    },
                    modifier = Modifier
                        .weight(1f)
                        .padding(horizontal = 4.dp)
                        .height(40.dp)
                        .onPreviewKeyEvent { event ->
                            if (!canSend) return@onPreviewKeyEvent false
                            if (event.type == KeyEventType.KeyDown) {
                                if (event.key == Key.Escape) {
                                    if (chatState.editingMessage != null) {
                                        chatState.editingMessage = null
                                        messageText = ""
                                        return@onPreviewKeyEvent true
                                    }
                                    if (chatState.replyingTo != null) {
                                        chatState.replyingTo = null
                                        return@onPreviewKeyEvent true
                                    }
                                }
                                if (event.isCtrlPressed && event.key == Key.V) {
                                        val files = getClipboardFiles()
                                        if (files.isNotEmpty()) {
                                            chatState.pendingFiles.addAll(files)
                                            return@onPreviewKeyEvent true
                                        }
                                }
                                if (event.key == Key.Enter && !event.isShiftPressed) {
                                    if (messageText.isNotBlank() || chatState.pendingFiles.isNotEmpty()) {
                                        chatState.sendMessage(messageText)
                                        messageText = ""
                                        return@onPreviewKeyEvent true
                                    }
                                }
                            }
                            false
                        },
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                    cursorBrush = androidx.compose.ui.graphics.SolidColor(MaterialTheme.colorScheme.primary),
                    decorationBox = { innerTextField: @Composable () -> Unit ->
                        Surface(
                            shape = RoundedCornerShape(20.dp),
                            color = MaterialTheme.colorScheme.surfaceContainerHigh,
                            modifier = Modifier.fillMaxSize()
                        ) {
                            Row(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .padding(horizontal = 16.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Box(modifier = Modifier.weight(1f)) {
                                    if (messageText.isEmpty()) {
                                        Text(
                                            text = placeholderText,
                                            style = MaterialTheme.typography.bodyMedium,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                    innerTextField()
                                }
                                
                                Box {
                                    IconButton(
                                        onClick = { showEmojiPicker = !showEmojiPicker },
                                        modifier = Modifier.size(32.dp),
                                        colors = IconButtonDefaults.iconButtonColors(
                                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    ) {
                                        Icon(
                                            imageVector = Icons.Filled.SentimentSatisfied,
                                            contentDescription = "Emojis",
                                            modifier = Modifier.size(20.dp)
                                        )
                                    }
                                    if (showEmojiPicker) {
                                        androidx.compose.ui.window.Popup(
                                            alignment = Alignment.BottomEnd,
                                            offset = IntOffset(0, -48),
                                            onDismissRequest = { showEmojiPicker = false }
                                        ) {
                                            EmojiPicker(chatState) { emoji ->
                                                messageText += if (emoji.id != null) "<:${emoji.name}:${emoji.id}>" else emoji.name ?: ""
                                                showEmojiPicker = false
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                )

                if (emojiSearchQuery != null) {
                    val filteredEmojis = remember(emojiSearchQuery, chatState.selectedGuild) {
                        val all = chatState.selectedGuild?.emojis ?: emptyList()
                        all.filter { it.name?.contains(emojiSearchQuery!!, ignoreCase = true) == true }.take(10)
                    }
                    
                    if (filteredEmojis.isNotEmpty()) {
                        androidx.compose.ui.window.Popup(
                            alignment = Alignment.BottomStart,
                            offset = IntOffset(60, (-220).dp.value.toInt())
                        ) {
                            Surface(
                                shape = RoundedCornerShape(8.dp),
                                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                shadowElevation = 4.dp,
                                modifier = Modifier.width(200.dp)
                            ) {
                                Column(modifier = Modifier.padding(4.dp)) {
                                    filteredEmojis.forEach { emoji ->
                                        DropdownMenuItem(
                                            text = { Text(emoji.name ?: "", style = MaterialTheme.typography.bodySmall) },
                                            leadingIcon = {
                                                val url = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=32" else null
                                                if (url != null) {
                                                    AsyncImage(url, null, modifier = Modifier.size(18.dp))
                                                }
                                            },
                                            onClick = {
                                                val before = messageText.substringBeforeLast(':')
                                                messageText = if (emoji.id != null) "$before<:${emoji.name}:${emoji.id}> " else "$before:${emoji.name}: "
                                                emojiSearchQuery = null
                                            }
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
                
                AnimatedVisibility(
                    visible = messageText.isNotBlank() || chatState.pendingFiles.isNotEmpty(),
                    enter = fadeIn(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) + 
                            expandHorizontally(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) +
                            scaleIn(initialScale = 0.8f, animationSpec = spring(dampingRatio = 0.6f, stiffness = Spring.StiffnessMediumLow)),
                    exit = fadeOut(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) + 
                           shrinkHorizontally(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) +
                           scaleOut(targetScale = 0.8f, animationSpec = spring(dampingRatio = 0.6f, stiffness = Spring.StiffnessMediumLow))
                ) {
                    Row {
                        Spacer(modifier = Modifier.width(4.dp))
                        IconButton(
                            onClick = {
                                chatState.sendMessage(messageText)
                                messageText = ""
                            },
                            colors = IconButtonDefaults.filledIconButtonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = MaterialTheme.colorScheme.onPrimary
                            )
                        ) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.Send,
                                contentDescription = "Send",
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}
