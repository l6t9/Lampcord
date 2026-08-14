package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.*
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.InteractionOption
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.chat.MediaPicker
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.getClipboardFiles
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

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
    channel: Channel,
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
    val keyboardController = LocalSoftwareKeyboardController.current

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

    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        val platform = getPlatformName()
        val isMobileDevice = platform == "android" || platform == "ios"
        val isMobileView = isMobileDevice || maxWidth < 600.dp
        val isDesktopTarget = platform == "desktop" || platform == "macos" || platform == "windows" || platform == "linux"
        
        Column(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.fillMaxWidth()) {
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

                        if (isMobileView) {
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
                                    val recipientId = channel.recipients?.firstOrNull()?.id ?: channel.recipient_ids?.firstOrNull()
                                    val recipient = recipientId?.let { userStore.getUser(it) } ?: channel.recipients?.firstOrNull()
                                    val name = recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                                    "Message @$name"
                                }
                                else -> "Message #${channel.name ?: "unnamed"}"
                            }

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
                                                    if (event.key == Key.Enter && !event.isShiftPressed && !isMobileView) {
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
                                        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
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
                                        onClick = { 
                                            navigationStore.isEmojiPickerVisible = !navigationStore.isEmojiPickerVisible
                                            if (isMobileView) {
                                                keyboardController?.hide()
                                            }
                                        },
                                        modifier = Modifier.size(36.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Filled.SentimentSatisfied,
                                            contentDescription = "Emojis",
                                            modifier = Modifier.size(22.dp),
                                            tint = if (navigationStore.isEmojiPickerVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }

                                    if (navigationStore.isEmojiPickerVisible && !isMobileView) {
                                        androidx.compose.ui.window.Popup(
                                            alignment = Alignment.BottomEnd,
                                            offset = IntOffset(0, -48),
                                            onDismissRequest = { navigationStore.isEmojiPickerVisible = false },
                                            properties = androidx.compose.ui.window.PopupProperties(focusable = true)
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
                                                navigationStore.isEmojiPickerVisible = false
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Send Button
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

                        // Mobile Emoji Picker - Moved below input row
                        AnimatedVisibility(
                            visible = navigationStore.isEmojiPickerVisible && !isDesktopTarget,
                            enter = expandVertically() + fadeIn(),
                            exit = shrinkVertically() + fadeOut()
                        ) {
                            EmojiPicker(
                                modifier = Modifier.fillMaxWidth(),
                                onEmojiSelected = { emoji ->
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
                                }
                            )
                        }
                    }
                }
            }
        }
    }
}
