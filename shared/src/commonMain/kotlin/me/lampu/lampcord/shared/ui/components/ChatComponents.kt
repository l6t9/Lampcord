package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.*
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.chat.MediaPicker
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.getClipboardFiles
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.getPlatformName

@Composable
fun TypingIndicator(chatState: ChatState, channelId: String) {
    val typingMap = chatState.typingUsers[channelId] ?: return
    val userIds = typingMap.keys.toList()
    if (userIds.isEmpty()) return
    
    val names = userIds.map { id ->
         val member = chatState.getMember(chatState.selectedGuild?.id ?: "", id)
         val userFromStore = chatState.userStore.getUser(id)
         val userFromChannel = chatState.selectedChannel?.recipients?.find { it.id == id }
         
         member?.nick 
         ?: userFromStore?.global_name 
         ?: userFromStore?.username 
         ?: userFromChannel?.global_name
         ?: userFromChannel?.username
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
    channel: me.lampu.lampcord.shared.model.Channel?,
    chatState: ChatState
) {
    Surface(
        modifier = Modifier.fillMaxWidth().height(48.dp),
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = 0.dp,
        tonalElevation = 0.dp
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (chatState.isChannelsAndRolesVisible) {
                Icon(
                    imageVector = Icons.Filled.Flag,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = "Channels & Roles", style = MaterialTheme.typography.titleSmall)
                Spacer(modifier = Modifier.weight(1f))
            } else if (channel != null) {
                val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12
                
                if (isDm) {
                    // DM style
                    val recipient = channel.recipients?.firstOrNull()
                    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                    Icon(
                        imageVector = Icons.Rounded.AlternateEmail,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(text = name, style = MaterialTheme.typography.titleSmall)
                } else {
                    val icon = when (channel.type) {
                        15 -> Icons.Rounded.Forum
                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                        5 -> Icons.Filled.Campaign
                        else -> Icons.Filled.Tag
                    }
                    if (isThread) {
                        Text(">", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) // Thread
                    } else {
                        Icon(
                            imageVector = icon,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(text = me.lampu.lampcord.shared.utils.CleanUtils.cleanChannelName(channel.name ?: "unnamed"), style = MaterialTheme.typography.titleSmall)
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
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    if (channel.type != 2 && channel.type != 13) {
                        IconButton(onClick = { chatState.isPinsVisible = true }) {
                            Icon(Icons.Filled.PushPin, "Pins", modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            }
        }
    }
}

class DiscordInputVisualTransformation(val primaryColor: Color) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val build = AnnotatedString.Builder()
        val rawText = text.text
        
        var i = 0
        while (i < rawText.length) {
            val char = rawText[i]
            if (char == '@' || char == '#' || char == '/' || char == ':') {
                 // Check if it's the start of a word or start of line
                 if (i == 0 || rawText[i-1] == ' ' || rawText[i-1] == '\n') {
                     var end = i + 1
                     while (end < rawText.length && rawText[end] != ' ' && rawText[end] != '\n') {
                         end++
                     }
                     build.withStyle(SpanStyle(color = primaryColor, fontWeight = FontWeight.Bold)) {
                         append(rawText.substring(i, end))
                     }
                     i = end
                     continue
                 }
            }
            build.append(char)
            i++
        }
        
        return TransformedText(build.toAnnotatedString(), OffsetMapping.Identity)
    }
}

@Composable
fun ChatInputBar(
    channel: me.lampu.lampcord.shared.model.Channel,
    chatState: ChatState
) {
    var textFieldValue by remember(channel.id) { 
        val draft = chatState.draftMessages[channel.id] ?: ""
        mutableStateOf(TextFieldValue(draft, TextRange(draft.length))) 
    }
    
    var showFilePicker by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }

    val canSend by remember(channel, chatState.currentUser) {
        derivedStateOf { chatState.hasPermission(Permission.SEND_MESSAGES) }
    }

    // Update draft whenever text changes
    LaunchedEffect(textFieldValue.text) {
        chatState.draftMessages[channel.id] = textFieldValue.text
    }

    // Keep local messageText in sync with draft changes from outside
    LaunchedEffect(chatState.draftMessages[channel.id]) {
        val draft = chatState.draftMessages[channel.id] ?: ""
        if (draft != textFieldValue.text) {
            textFieldValue = TextFieldValue(draft, TextRange(draft.length))
        }
    }

    // Sync messageText when editing starts
    LaunchedEffect(chatState.editingMessage) {
        chatState.editingMessage?.let {
            textFieldValue = TextFieldValue(it.content, TextRange(it.content.length))
            chatState.pendingFiles.clear()
            chatState.replyingTo = null
            focusRequester.requestFocus()
        }
    }

    LaunchedEffect(chatState.replyingTo) {
        chatState.replyingTo?.let {
            chatState.editingMessage = null
            focusRequester.requestFocus()
        }
    }

    fun applyAutocomplete(replacement: String) {
        val text = textFieldValue.text
        val selection = textFieldValue.selection
        if (selection.collapsed) {
            val cursor = selection.start
            val textBefore = text.take(cursor)
            val lastWordStart = textBefore.lastIndexOfAny(charArrayOf(' ', '\n')) + 1
            
            // If it's a command, it's usually at the start
            val actualStart = if (chatState.autocompleteType == AutocompleteType.COMMAND) 0 else lastWordStart
            
            val newText = text.replaceRange(actualStart, cursor, replacement)
            val newCursor = actualStart + replacement.length
            textFieldValue = TextFieldValue(newText, TextRange(newCursor))
            chatState.updateAutocomplete(null, "")
        }
    }

    val primaryColor = MaterialTheme.colorScheme.primary

    Column(modifier = Modifier.fillMaxWidth()) {
        Box(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.fillMaxWidth().align(Alignment.BottomCenter)) {
                // Autocomplete Picker
                AnimatedVisibility(
                    visible = chatState.autocompleteType != null,
                    enter = slideInVertically(initialOffsetY = { it }) + fadeIn(),
                    exit = slideOutVertically(targetOffsetY = { it }) + fadeOut()
                ) {
                    chatState.autocompleteType?.let { type ->
                        AutocompletePicker(
                            chatState = chatState,
                            type = type,
                            query = chatState.autocompleteQuery,
                            selectedIndex = chatState.autocompleteSelectedIndex,
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp),
                            onItemSelected = { item ->
                                if (item.isCommand && item.commandObj != null) {
                                    chatState.activeCommand = item.commandObj
                                    textFieldValue = TextFieldValue("")
                                    chatState.updateAutocomplete(null, "")
                                } else {
                                    applyAutocomplete(item.replacement)
                                }
                            }
                        )
                    }
                }

                Surface(
                    color = Color.Transparent,
                    modifier = Modifier
                        .fillMaxWidth()
                        .navigationBarsPadding()
                        .imePadding()
                ) {
                    Column {
                        TypingIndicator(chatState, channel.id)

                        if (chatState.activeCommand != null) {
                            CommandParameterUI(chatState)
                        }

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
                                                textFieldValue = TextFieldValue("")
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
                                chatState.pendingFiles.forEachIndexed { index, pendingFile ->
                                    Surface(
                                        modifier = Modifier.size(100.dp),
                                        shape = RoundedCornerShape(8.dp),
                                        color = MaterialTheme.colorScheme.surfaceContainerHigh
                                    ) {
                                        Box {
                                            val name = pendingFile.name
                                            val data = pendingFile.data
                                            val uri = pendingFile.uri
                                            val isVideo = name.lowercase().let { it.endsWith(".mp4") || it.endsWith(".mov") || it.endsWith(".mkv") || it.endsWith(".webm") }

                                            if (name.lowercase().let { it.endsWith(".png") || it.endsWith(".jpg") || it.endsWith(".jpeg") || it.endsWith(".webp") || it.endsWith(".gif") }) {
                                                AsyncImage(
                                                    model = data,
                                                    contentDescription = name,
                                                    modifier = Modifier.fillMaxSize(),
                                                    contentScale = ContentScale.Crop
                                                )
                                            } else if (isVideo && uri != null) {
                                                VideoThumbnail(
                                                    uri = uri,
                                                    contentDescription = name,
                                                    modifier = Modifier.fillMaxSize()
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

                        val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
                        if (isMobile) {
                            if (chatState.isMediaPickerVisible) {
                                MediaPicker(chatState) {
                                    chatState.isMediaPickerVisible = false
                                }
                            }
                        } else {
                            AnimatedVisibility(
                                visible = chatState.isMediaPickerVisible,
                                enter = expandVertically() + fadeIn(),
                                exit = shrinkVertically() + fadeOut()
                            ) {
                                MediaPicker(chatState) {
                                    chatState.isMediaPickerVisible = false
                                }
                            }
                        }

                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 8.dp, vertical = 8.dp)
                                .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (getPlatformName() != "android") {
                                FilePicker(
                                    show = showFilePicker,
                                    onFileSelected = { chatState.pendingFiles.addAll(it.map { PendingFile(it.first, it.second) }) },
                                    onDismiss = { showFilePicker = false }
                                )
                            }

                            IconButton(
                                onClick = {
                                    if (getPlatformName() == "android") {
                                        chatState.isMediaPickerVisible = !chatState.isMediaPickerVisible
                                    } else {
                                        showFilePicker = true
                                    }
                                },
                                enabled = chatState.editingMessage == null && canSend,
                                colors = IconButtonDefaults.filledTonalIconButtonColors(
                                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                ),
                                modifier = Modifier.size(40.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Add,
                                    contentDescription = "Add",
                                    modifier = Modifier.size(24.dp)
                                )
                            }
                            
                            Spacer(modifier = Modifier.width(8.dp))

                            val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                            val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12
                            
                            val placeholderText = when {
                                !canSend -> "You do not have permission to send messages."
                                isThread -> "Reply to thread..."
                                isDm -> {
                                    val recipient = channel.recipients?.firstOrNull()
                                    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                                    "Message @$name"
                                }
                                else -> "Message #${channel.name ?: "unnamed"}"
                            }

                            var showEmojiPicker by remember { mutableStateOf(false) }

                            Surface(
                                modifier = Modifier
                                    .weight(1f)
                                    .heightIn(min = 40.dp),
                                shape = RoundedCornerShape(16.dp),
                                color = MaterialTheme.colorScheme.surfaceContainerHigh
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                            BasicTextField(
                                value = textFieldValue,
                                onValueChange = { 
                                    if (canSend) {
                                        textFieldValue = it
                                        
                                        // Simplified autocomplete trigger check
                                        if (it.selection.collapsed) {
                                            val cursor = it.selection.start
                                            val textBefore = it.text.take(cursor)
                                            val lastWord = textBefore.substringAfterLast(' ', textBefore)
                                            
                                            val (type, query) = when {
                                                it.text.startsWith('/') && !it.text.contains(' ') -> 
                                                    AutocompleteType.COMMAND to it.text.substring(1)
                                                lastWord.startsWith('@') -> 
                                                    AutocompleteType.MENTION to lastWord.substring(1)
                                                lastWord.startsWith('#') -> 
                                                    AutocompleteType.CHANNEL to lastWord.substring(1)
                                                lastWord.startsWith(':') -> 
                                                    AutocompleteType.EMOJI to lastWord.substring(1)
                                                else -> null to ""
                                            }
                                            chatState.updateAutocomplete(type, query)
                                        } else {
                                            chatState.updateAutocomplete(null, "")
                                        }

                                        if (it.text.isNotEmpty()) {
                                            chatState.sendTyping()
                                        }
                                    }
                                },
                                visualTransformation = DiscordInputVisualTransformation(primaryColor),
                                modifier = Modifier
                                    .weight(1f)
                                    .padding(start = 12.dp, top = 8.dp, bottom = 8.dp)
                                    .focusRequester(focusRequester)
                                    .onPreviewKeyEvent { event ->
                                        if (!canSend) return@onPreviewKeyEvent false
                                        if (event.type == KeyEventType.KeyDown) {
                                            if (chatState.autocompleteType != null) {
                                                val itemCount = chatState.autocompleteItems.size
                                                when (event.key) {
                                                    Key.DirectionUp -> {
                                                        if (itemCount > 0) {
                                                            chatState.autocompleteSelectedIndex = (chatState.autocompleteSelectedIndex - 1 + itemCount) % itemCount
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                    Key.DirectionDown -> {
                                                        if (itemCount > 0) {
                                                            chatState.autocompleteSelectedIndex = (chatState.autocompleteSelectedIndex + 1) % itemCount
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                    Key.Tab, Key.Enter -> {
                                                        if (itemCount > 0 && chatState.autocompleteSelectedIndex in 0 until itemCount) {
                                                            val item = chatState.autocompleteItems[chatState.autocompleteSelectedIndex]
                                                            if (item.isCommand && item.commandObj != null) {
                                                                chatState.activeCommand = item.commandObj
                                                                textFieldValue = TextFieldValue("")
                                                                chatState.updateAutocomplete(null, "")
                                                            } else {
                                                                applyAutocomplete(item.replacement)
                                                            }
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                }
                                            }

                                                    if (event.key == Key.Escape) {
                                                        if (chatState.autocompleteType != null) {
                                                            chatState.updateAutocomplete(null, "")
                                                            return@onPreviewKeyEvent true
                                                        }
                                                        if (chatState.editingMessage != null) {
                                                            chatState.editingMessage = null
                                                            textFieldValue = TextFieldValue("")
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
                                                                chatState.pendingFiles.addAll(files.map { PendingFile(it.first, it.second) })
                                                                return@onPreviewKeyEvent true
                                                            }
                                                    }
                                                    if (event.key == Key.Enter && !event.isShiftPressed) {
                                                        if (textFieldValue.text.startsWith('/') && !textFieldValue.text.contains(' ')) {
                                                            val cmdName = textFieldValue.text.substring(1).trim()
                                                            val command = chatState.availableCommands.find { it.name == cmdName }
                                                            if (command != null) {
                                                                chatState.sendInteraction(command)
                                                                textFieldValue = TextFieldValue("")
                                                                return@onPreviewKeyEvent true
                                                            }
                                                        }
                                                        if (textFieldValue.text.isNotBlank() || chatState.pendingFiles.isNotEmpty()) {
                                                            chatState.sendMessage(textFieldValue.text)
                                                            textFieldValue = TextFieldValue("")
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                }
                                                false
                                            },
                                        textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                                        cursorBrush = androidx.compose.ui.graphics.SolidColor(MaterialTheme.colorScheme.primary),
                                        decorationBox = { innerTextField: @Composable () -> Unit ->
                                            Box(modifier = Modifier.fillMaxWidth()) {
                                                if (textFieldValue.text.isEmpty()) {
                                                    Text(
                                                        text = placeholderText,
                                                        style = MaterialTheme.typography.bodyMedium,
                                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                                    )
                                                }
                                                innerTextField()
                                            }
                                        }
                                    )

                                    IconButton(
                                        onClick = { showEmojiPicker = !showEmojiPicker },
                                        modifier = Modifier.size(36.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Filled.SentimentSatisfied,
                                            contentDescription = "Emojis",
                                            modifier = Modifier.size(22.dp),
                                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }

                                    if (showEmojiPicker) {
                                        androidx.compose.ui.window.Popup(
                                            alignment = Alignment.BottomEnd,
                                            offset = IntOffset(0, -48),
                                            onDismissRequest = { showEmojiPicker = false }
                                        ) {
                                            EmojiPicker(chatState) { emoji ->
                                                val isExternal = emoji.guild_id != null && emoji.guild_id != chatState.selectedGuild?.id
                                                val hasNitro = (chatState.currentUser?.premium_type ?: 0) > 0
                                                val freeNitro = me.lampu.lampcord.shared.settings.Settings.shared.freeNitroEmojis
                                                
                                                val emojiText = if (emoji.id != null) {
                                                    if (isExternal && !hasNitro && freeNitro) {
                                                        if (me.lampu.lampcord.shared.settings.Settings.shared.realmojis) {
                                                            "<${if (emoji.animated == true) "a" else ""}:F_${emoji.name}:${emoji.id}>"
                                                        } else {
                                                            val ext = if (emoji.animated == true) "gif" else "png"
                                                            "https://cdn.discordapp.com/emojis/${emoji.id}.$ext?size=48"
                                                        }
                                                    } else {
                                                        "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>"
                                                    }
                                                } else emoji.name ?: ""

                                                val newText = textFieldValue.text.take(textFieldValue.selection.start) + emojiText + textFieldValue.text.drop(textFieldValue.selection.end)
                                                textFieldValue = TextFieldValue(newText, TextRange(textFieldValue.selection.start + emojiText.length))
                                                showEmojiPicker = false
                                            }
                                        }
                                    }
                                }
                            }
                            
                            AnimatedVisibility(
                                visible = textFieldValue.text.isNotBlank() || chatState.pendingFiles.isNotEmpty() || chatState.activeCommand != null,
                                enter = fadeIn() + scaleIn(initialScale = 0.8f) + slideInHorizontally { it },
                                exit = fadeOut() + scaleOut(targetScale = 0.8f) + slideOutHorizontally { it },
                                modifier = Modifier.padding(start = 8.dp)
                            ) {
                                IconButton(
                                    onClick = {
                                        if (chatState.activeCommand != null) {
                                            val interactionOptions = chatState.commandOptions.map { (name, value) ->
                                                val optionType = chatState.activeCommand!!.options!!.find { it.name == name }?.type ?: 3
                                                InteractionOption(type = optionType, name = name, value = value)
                                            }
                                            chatState.sendInteraction(chatState.activeCommand!!, interactionOptions)
                                            chatState.activeCommand = null
                                            chatState.commandOptions.clear()
                                        } else {
                                            chatState.sendMessage(textFieldValue.text)
                                        }
                                        textFieldValue = TextFieldValue("")
                                    },
                                    colors = IconButtonDefaults.filledIconButtonColors(
                                        containerColor = MaterialTheme.colorScheme.primary,
                                        contentColor = MaterialTheme.colorScheme.onPrimary
                                    ),
                                    modifier = Modifier.size(40.dp)
                                ) {
                                    Icon(
                                        imageVector = Icons.Rounded.Send,
                                        contentDescription = "Send",
                                        modifier = Modifier.size(20.dp)
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
