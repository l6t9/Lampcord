package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.model.InteractionOption
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.state.CommandStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.TypingStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.chat.MediaPicker
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.getClipboardFiles
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun TypingIndicator(
    typingStore: TypingStore = koinInject(),
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val typingUsers by typingStore.typingUsers.collectAsState()
    val channelId = navigationStore.selectedChannel?.id ?: return
    val typingMap = typingUsers[channelId] ?: return
    val userIds = typingMap.keys.toList()
    if (userIds.isEmpty()) return
    
    val names = userIds.map { id ->
         val member = navigationStore.selectedGuild?.let { userStore.getMember(it.id, id) }
         val userFromStore = userStore.getUser(id)
         val userFromChannel = navigationStore.selectedChannel?.recipients?.find { it.id == id }
         
         member?.nick 
         ?: userFromStore?.global_name 
         ?: userFromStore?.username 
         ?: userFromChannel?.global_name
         ?: userFromChannel?.username
         ?: "Someone"
    }
    
    val text = when (names.size) {
        1 -> "${names[0]} is typing..."
        2 -> "${names[0]} and ${names[1]} are typing..."
        3 -> "${names[0]}, ${names[1]} and ${names[2]} are typing..."
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
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val allUsers by userStore.users.collectAsState()
    
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
            if (navigationStore.isChannelsAndRolesVisible) {
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
                    val name = if (channel.name?.isNotBlank() == true) {
                        channel.name
                    } else {
                        val recipientId = channel.recipients?.firstOrNull()?.id ?: channel.recipient_ids?.firstOrNull()
                        val recipient = recipientId?.let { allUsers[it] } ?: channel.recipients?.firstOrNull()
                        recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
                    }

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
                    if (channel.type == 0 || channel.type == 5 || channel.type == 15) {
                        IconButton(onClick = { navigationStore.isThreadPanelVisible = !navigationStore.isThreadPanelVisible }) {
                            Icon(Icons.Filled.Tag, "Threads", modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                    if (channel.type != 2 && channel.type != 13) {
                        IconButton(onClick = { navigationStore.isPinsVisible = true }) {
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
    messageStore: MessageStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    autocompleteStore: AutocompleteStore = koinInject(),
    commandStore: CommandStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    var textFieldValue by remember(channel.id) { 
        val draft = messageStore.draftMessages[channel.id] ?: ""
        mutableStateOf(TextFieldValue(draft, TextRange(draft.length))) 
    }
    
    var showFilePicker by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }

    val currentUser by userStore.currentUser.collectAsState()
    val member = remember(navigationStore.selectedGuild, currentUser) {
        val guild = navigationStore.selectedGuild ?: return@remember null
        val user = currentUser ?: return@remember null
        userStore.getMember(guild.id, user.id)
    }

    val canSend by remember(channel, currentUser, member, navigationStore.selectedGuild) {
        derivedStateOf {
            val guild = navigationStore.selectedGuild
            val user = currentUser
            if (user == null) true
            else if (guild == null) true // DMs
            else me.lampu.lampcord.shared.utils.PermissionHelper.hasPermission(
                member ?: Member(user = user),
                guild,
                channel,
                Permission.SEND_MESSAGES,
                user.id
            )
        }
    }

    // Update draft whenever text changes
    LaunchedEffect(textFieldValue.text) {
        if (textFieldValue.text.isEmpty()) {
            messageStore.draftMessages.remove(channel.id)
        } else {
            messageStore.draftMessages[channel.id] = textFieldValue.text
        }
    }

    // Keep local messageText in sync with draft changes from outside
    LaunchedEffect(messageStore.draftMessages[channel.id]) {
        val draft = messageStore.draftMessages[channel.id] ?: ""
        if (draft != textFieldValue.text) {
            textFieldValue = TextFieldValue(draft, TextRange(draft.length))
        }
    }

    // Sync messageText when editing starts
    LaunchedEffect(messageStore.editingMessage) {
        messageStore.editingMessage?.let {
            textFieldValue = TextFieldValue(it.content, TextRange(it.content.length))
            messageStore.pendingFiles.clear()
            messageStore.replyingTo = null
            focusRequester.requestFocus()
        }
    }

    LaunchedEffect(messageStore.replyingTo) {
        messageStore.replyingTo?.let {
            messageStore.editingMessage = null
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
            val actualStart = if (autocompleteStore.autocompleteType == AutocompleteType.COMMAND) 0 else lastWordStart
            
            val newText = text.replaceRange(actualStart, cursor, replacement)
            val newCursor = actualStart + replacement.length
            textFieldValue = TextFieldValue(newText, TextRange(newCursor))
            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
        }
    }

    val primaryColor = MaterialTheme.colorScheme.primary

    Column(modifier = Modifier.fillMaxWidth()) {
        Box(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.fillMaxWidth().align(Alignment.BottomCenter)) {
                // Autocomplete Picker
                AnimatedVisibility(
                    visible = autocompleteStore.autocompleteType != null,
                    enter = slideInVertically(initialOffsetY = { it }) + fadeIn(),
                    exit = slideOutVertically(targetOffsetY = { it }) + fadeOut()
                ) {
                    autocompleteStore.autocompleteType?.let { type ->
                        AutocompletePicker(
                            type = type,
                            query = autocompleteStore.autocompleteQuery,
                            selectedIndex = autocompleteStore.autocompleteSelectedIndex,
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp),
                            onItemSelected = { item ->
                                if (item.isCommand && item.commandObj != null) {
                                    commandStore.activeCommand = item.commandObj
                                    textFieldValue = TextFieldValue("")
                                    autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
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
                        TypingIndicator()

                        if (commandStore.activeCommand != null) {
                            CommandParameterUI()
                        }

                        if (messageStore.replyingTo != null || messageStore.editingMessage != null) {
                            Surface(
                                modifier = Modifier.fillMaxWidth(),
                                color = MaterialTheme.colorScheme.surfaceContainerLow
                            ) {
                                Row(
                                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    val isEditing = messageStore.editingMessage != null
                                    val activeMsg = messageStore.editingMessage ?: messageStore.replyingTo
                                    
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
                                                messageStore.editingMessage = null
                                                textFieldValue = TextFieldValue("")
                                            } else {
                                                messageStore.replyingTo = null 
                                            }
                                        },
                                        modifier = Modifier.size(24.dp)
                                    ) {
                                        Icon(Icons.Filled.Close, null, modifier = Modifier.size(16.dp))
                                    }
                                }
                            }
                        }

                        if (messageStore.pendingFiles.isNotEmpty()) {
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(8.dp)
                                    .height(120.dp),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                messageStore.pendingFiles.forEachIndexed { index, pendingFile ->
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
                                                onClick = { messageStore.pendingFiles.removeAt(index) },
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
                            if (navigationStore.isMediaPickerVisible) {
                                MediaPicker(onDismiss = {
                                    navigationStore.isMediaPickerVisible = false
                                })
                            }
                        } else {
                            AnimatedVisibility(
                                visible = navigationStore.isMediaPickerVisible,
                                enter = expandVertically() + fadeIn(),
                                exit = shrinkVertically() + fadeOut()
                            ) {
                                MediaPicker(onDismiss = {
                                    navigationStore.isMediaPickerVisible = false
                                })
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
                                    onFileSelected = { it -> messageStore.pendingFiles.addAll(it.map { PendingFile(it.first, it.second) }) },
                                    onDismiss = { showFilePicker = false }
                                )
                            }

                            IconButton(
                                onClick = {
                                    if (getPlatformName() == "android") {
                                        navigationStore.isMediaPickerVisible = !navigationStore.isMediaPickerVisible
                                    } else {
                                        showFilePicker = true
                                    }
                                },
                                enabled = messageStore.editingMessage == null && canSend,
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
                                            autocompleteStore.updateAutocomplete(type, query, navigationStore.selectedGuild)
                                        } else {
                                            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                                        }

                                        if (it.text.isNotEmpty()) {
                                            messageStore.sendTyping(channel.id)
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
                                            if (autocompleteStore.autocompleteType != null) {
                                                val itemCount = autocompleteStore.autocompleteItems.size
                                                when (event.key) {
                                                    Key.DirectionUp -> {
                                                        if (itemCount > 0) {
                                                            autocompleteStore.autocompleteSelectedIndex = (autocompleteStore.autocompleteSelectedIndex - 1 + itemCount) % itemCount
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                    Key.DirectionDown -> {
                                                        if (itemCount > 0) {
                                                            autocompleteStore.autocompleteSelectedIndex = (autocompleteStore.autocompleteSelectedIndex + 1) % itemCount
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                    Key.Tab, Key.Enter -> {
                                                        if (itemCount > 0 && autocompleteStore.autocompleteSelectedIndex in 0 until itemCount) {
                                                            val item = autocompleteStore.autocompleteItems[autocompleteStore.autocompleteSelectedIndex]
                                                            if (item.isCommand && item.commandObj != null) {
                                                                commandStore.activeCommand = item.commandObj
                                                                textFieldValue = TextFieldValue("")
                                                                autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                                                            } else {
                                                                applyAutocomplete(item.replacement)
                                                            }
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                }
                                            }

                                                    if (event.key == Key.Escape) {
                                                        if (autocompleteStore.autocompleteType != null) {
                                                            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild)
                                                            return@onPreviewKeyEvent true
                                                        }
                                                        if (messageStore.editingMessage != null) {
                                                            messageStore.editingMessage = null
                                                            textFieldValue = TextFieldValue("")
                                                            return@onPreviewKeyEvent true
                                                        }
                                                        if (messageStore.replyingTo != null) {
                                                            messageStore.replyingTo = null
                                                            return@onPreviewKeyEvent true
                                                        }
                                                    }
                                                    if (event.isCtrlPressed && event.key == Key.V) {
                                                            val files = getClipboardFiles()
                                                            if (files.isNotEmpty()) {
                                                                messageStore.pendingFiles.addAll(files.map { PendingFile(it.first, it.second) })
                                                                return@onPreviewKeyEvent true
                                                            }
                                                    }
                                                    if (event.key == Key.Enter && !event.isShiftPressed) {
                                                        val currentText = textFieldValue.text
                                                        if (currentText.startsWith('/') && !currentText.contains(' ')) {
                                                            val cmdName = currentText.substring(1).trim()
                                                            val command = commandStore.availableCommands.find { it.name == cmdName }
                                                            if (command != null) {
                                                                commandStore.sendInteraction(
                                                                    command = command,
                                                                    guildId = navigationStore.selectedGuild?.id,
                                                                    channelId = channel.id
                                                                )
                                                                textFieldValue = TextFieldValue("")
                                                                return@onPreviewKeyEvent true
                                                            }
                                                        }
                                                        if (currentText.isNotBlank() || messageStore.pendingFiles.isNotEmpty()) {
                                                            if (messageStore.editingMessage != null) {
                                                                messageStore.editMessage(messageStore.editingMessage!!, currentText)
                                                                messageStore.editingMessage = null
                                                            } else {
                                                                messageStore.sendMessageDraft(currentText)
                                                            }
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
                                            EmojiPicker { emoji ->
                                                val isExternal = emoji.guild_id != null && emoji.guild_id != navigationStore.selectedGuild?.id
                                                val hasNitro = (currentUser?.premium_type ?: 0) > 0
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
                                visible = textFieldValue.text.isNotBlank() || messageStore.pendingFiles.isNotEmpty() || commandStore.activeCommand != null,
                                enter = fadeIn() + scaleIn(initialScale = 0.8f) + slideInHorizontally { it },
                                exit = fadeOut() + scaleOut(targetScale = 0.8f) + slideOutHorizontally { it },
                                modifier = Modifier.padding(start = 8.dp)
                            ) {
                                IconButton(
                                    onClick = {
                                        if (commandStore.activeCommand != null) {
                                            val interactionOptions = commandStore.commandOptions.map { (name, value) ->
                                                val optionType = commandStore.activeCommand!!.options!!.find { it.name == name }?.type ?: 3
                                                InteractionOption(type = optionType, name = name, value = value)
                                            }
                                            commandStore.sendInteraction(
                                                command = commandStore.activeCommand!!,
                                                guildId = navigationStore.selectedGuild?.id,
                                                channelId = channel.id,
                                                options = interactionOptions
                                            )
                                            commandStore.activeCommand = null
                                            commandStore.commandOptions.clear()
                                        } else {
                                            if (messageStore.editingMessage != null) {
                                                messageStore.editMessage(messageStore.editingMessage!!, textFieldValue.text)
                                                messageStore.editingMessage = null
                                            } else {
                                                messageStore.sendMessageDraft(textFieldValue.text)
                                            }
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
