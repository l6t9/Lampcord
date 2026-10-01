package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.*
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.AutocompleteItem
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.InteractionOption
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.model.Role
import me.lampu.lampcord.shared.model.isUnicodeEmoji
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.chat.MediaPicker
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.utils.EmojiIndex
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.getClipboardFiles
import me.lampu.lampcord.shared.utils.showToast
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import me.lampu.lampcord.shared.ui.kit.clickableCursor

class DiscordInputVisualTransformation(val primaryColor: Color) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val build = AnnotatedString.Builder()
        val rawText = text.text
        
        var i = 0
        while (i < rawText.length) {
            val char = rawText[i]
            if (char == '@' || char == '#' || char == '/' || char == ':') {
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
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    relationshipStore: RelationshipStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    var textFieldValue by remember(channel.id) { 
        val draft = messageStore.draftMessages[channel.id] ?: ""
        mutableStateOf(TextFieldValue(draft, TextRange(draft.length))) 
    }
    
    val haptic = LocalHapticFeedback.current

    var mentionRanges by remember(channel.id) { mutableStateOf<Map<IntRange, String>>(emptyMap()) }
    
    var isFocused by remember { mutableStateOf(false) }
    var showFilePicker by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current

    val currentUser by userStore.currentUser.collectAsState()
    val member = remember(navigationStore.selectedGuild, currentUser) {
        val guild = navigationStore.selectedGuild ?: return@remember null
        val user = currentUser ?: return@remember null
        userStore.getMember(guild.id, user.id)
    }

    val canSend = remember(channel, currentUser, member, navigationStore.selectedGuild) {
        val guild = navigationStore.selectedGuild
        val user = currentUser
        if (channel.thread_metadata?.locked == true && (user == null || member == null)) false
        else if (user == null) true
        else if (guild == null) true // DMs
        else if (member == null) channel.thread_metadata?.locked != true // Keep normal channels usable while the member loads, but never expose a locked thread input.
        else me.lampu.lampcord.shared.utils.PermissionHelper.canSendMessages(
            member,
            guild,
            channel,
            user.id
        )
    }

    LaunchedEffect(canSend) {
        if (!canSend) navigationStore.isEmojiPickerVisible = false
    }

    val focusOnChannelOpen = getPlatformName() != "android" && getPlatformName() != "ios"
    LaunchedEffect(channel.id, canSend, focusOnChannelOpen) {
        if (!focusOnChannelOpen || !canSend) return@LaunchedEffect
        if (navigationStore.isEmojiPickerVisible) return@LaunchedEffect
        if (messageStore.editingMessage != null || messageStore.replyingTo != null) return@LaunchedEffect
        focusRequester.requestFocus()
    }

    val replyTarget = messageStore.replyingTo
    val isSelfOrWebhookReply = replyTarget != null &&
        (replyTarget.author?.id == currentUser?.id || replyTarget.webhook_id != null)
    val showReplyMentionToggle = replyTarget != null && navigationStore.selectedGuild != null && !isSelfOrWebhookReply
    val replyShouldMention = replyTarget != null &&
        navigationStore.shouldMentionReply &&
        !isSelfOrWebhookReply

    // Discord only emits a reply ping when allowed_mentions is omitted entirely, or when
    // replied_user is explicitly true. Sending replied_user=true is not enough on its own, so
    // a ping-on reply omits the key and a ping-off reply suppresses it.
    val allowedMentions = if (replyTarget == null) {
        null
    } else if (replyShouldMention) {
        null
    } else {
        me.lampu.lampcord.shared.model.AllowedMentions(
            parse = listOf("users", "roles", "everyone"),
            replied_user = false
        )
    }

    fun applyAutocomplete(item: AutocompleteItem) {
        haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
        val text = textFieldValue.text
        val selection = textFieldValue.selection
        if (selection.collapsed) {
            val cursor = selection.start
            val textBefore = text.take(cursor)
            val lastWordStart = textBefore.lastIndexOfAny(charArrayOf(' ', '\n')) + 1

            val colonStart = if (item.inputText?.startsWith(":") == true) {
                val idx = textBefore.lastIndexOf(':')
                if (idx >= 0) idx else lastWordStart
            } else lastWordStart

            val actualStart = if (item.isCommand) 0 else colonStart

            val inputText = item.inputText ?: item.replacement
            val newText = text.replaceRange(actualStart, cursor, inputText)
            val newCursor = actualStart + inputText.length
            textFieldValue = TextFieldValue(newText, TextRange(newCursor))

            val removedLen = cursor - actualStart
            val delta = inputText.length - removedLen
            var shifted = shiftMentionRanges(mentionRanges, actualStart, cursor, delta)
            if (item.inputText != null && item.inputText != item.replacement) {
                shifted = shifted + (actualStart until newCursor to item.replacement)
            }
            mentionRanges = shifted

            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild, channel)
        }
    }

    fun resolveContent(content: String): String {
        val guildId = navigationStore.selectedGuild?.id
        val channels = guildId?.let { gid ->
            guildStore.allGuildChannels.value.values.filter { it.guild_id == gid }
        } ?: emptyList()
        val members = memberListStore.memberListItems.mapNotNull { it?.member } +
            relationshipStore.relationships.value.mapNotNull { rel -> rel.user?.let { Member(user = it) } }
        val roles = navigationStore.selectedGuild?.roles ?: emptyList()
        
        val guildEmojis = guildStore.guilds.value.flatMap { guild ->
            guild.emojis.map { it.copy(guild_id = guild.id) }
        }
        val allAvailableEmojis = guildEmojis.distinctBy { it.id }
        
        return resolveServerContent(content, mentionRanges, channels, members, roles, allAvailableEmojis, currentUser, guildId)
    }

    fun clearMentions() {
        mentionRanges = emptyMap()
    }

    LaunchedEffect(textFieldValue.text) {
        if (textFieldValue.text.isEmpty()) {
            messageStore.draftMessages.remove(channel.id)
        } else {
            messageStore.draftMessages[channel.id] = textFieldValue.text
        }
    }

    LaunchedEffect(channel.id) {
        val draft = messageStore.draftMessages[channel.id] ?: ""
        textFieldValue = TextFieldValue(draft, TextRange(draft.length))
        autocompleteStore.clear()
    }

    LaunchedEffect(messageStore.editingMessage) {
        messageStore.editingMessage?.let {
            textFieldValue = TextFieldValue(it.content, TextRange(it.content.length))
            clearMentions()
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

    val primaryColor = MaterialTheme.colorScheme.primary
    val settings = me.lampu.lampcord.shared.settings.Settings.shared
    val reduceMotion = settings.reduceMotion
    val chatboxFontSize = settings.chatboxFontSize
    val chatboxMinHeight = settings.chatboxHeight.dp * chatboxFontSize
    val buttonSize = chatboxMinHeight + (6.dp * chatboxFontSize)
    val iconSize = buttonSize * 0.55f

    fun insertEmoji(emoji: me.lampu.lampcord.shared.model.Emoji) {
        if (!canSend) return
        haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
        val hasNitro = (currentUser?.premium_type ?: 0) > 0
        val serverReplacement = if (emoji.id != null) {
            val forceF = settings.freeNitroEmojis && settings.realmojis && !hasNitro
            val namePart = if (forceF) "F_${emoji.name}" else emoji.name ?: "emoji"
            "<${if (emoji.animated == true) "a" else ""}:${namePart}:${emoji.id}>"
        } else null

        if (emoji.id != null && serverReplacement != null) {
            applyAutocomplete(AutocompleteItem(
                id = emoji.id,
                title = ":${emoji.name}:",
                replacement = serverReplacement,
                inputText = ":${emoji.name}:"
            ))
        } else {
            // Standard emoji reach this branch with `name` set to the Unicode character, not a
            // shortcode: EmojiIndex builds the picker grid with `name = entry.surrogates`.
            // So ":$name:" put ":😀:" in the box, and because getCharForName cannot resolve a
            // character back to a name, no mention range was registered and that literal text
            // went to the server. Resolve the character to its shortcode for the box, and keep
            // mapping the range back to the character so the payload still goes out as a plain
            // emoji the way it did before.
            val char = emoji.name ?: return
            val shortcode = if (isUnicodeEmoji(char)) {
                EmojiIndex.getNamesForChar(char)?.firstOrNull()
            } else {
                char.removeSurrounding(":").takeIf { it.isNotBlank() }
            }
            val inputText = if (shortcode != null) ":$shortcode:" else char
            val insertStart = textFieldValue.selection.start
            val newText = textFieldValue.text.replaceRange(
                insertStart,
                textFieldValue.selection.end,
                inputText
            )
            textFieldValue = TextFieldValue(newText, TextRange(insertStart + inputText.length))
            val unicode = EmojiIndex.getCharForName(shortcode ?: char)
            if (unicode != null && unicode != inputText) {
                mentionRanges = mentionRanges + (insertStart until insertStart + inputText.length to unicode)
            }
        }
    }

    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        val platform = getPlatformName()
        val isMobileDevice = platform == "android" || platform == "ios"
        val isMobileView = isMobileDevice || maxWidth < 600.dp
        val isDesktopTarget = platform == "desktop" || platform == "macos" || platform == "windows" || platform == "linux"
        
        Column(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.fillMaxWidth()) {
                AnimatedVisibility(
                    visible = autocompleteStore.autocompleteType != null,
                    enter = if (reduceMotion) EnterTransition.None else slideInVertically(initialOffsetY = { it }) + fadeIn(),
                    exit = if (reduceMotion) ExitTransition.None else slideOutVertically(targetOffsetY = { it }) + fadeOut()
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
                                    commandStore.resetSubCommand()
                                    textFieldValue = TextFieldValue("")
                                    clearMentions()
                                    autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild, channel)
                                } else {
                                    applyAutocomplete(item)
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
                            val isEditing = messageStore.editingMessage != null
                            val activeMsg = messageStore.editingMessage ?: messageStore.replyingTo
                            val guildId = activeMsg?.guild_id ?: navigationStore.selectedGuild?.id
                            
                            val roleColor = remember(activeMsg, guildId) {
                                val guild = guildStore.guilds.value.find { it.id == guildId } ?: return@remember Color.Unspecified
                                val authorId = activeMsg?.author?.id ?: return@remember Color.Unspecified
                                val member = userStore.getMember(guild.id, authorId) ?: return@remember Color.Unspecified
                                val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                                val colorRole = memberRoles.filter { it.color != 0 }.maxByOrNull { it.position }
                                if (colorRole != null) Color(colorRole.color or 0xFF000000.toInt()) else Color.Unspecified
                            }

                            Row(
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Box(
                                    modifier = Modifier.size(buttonSize),
                                    contentAlignment = Alignment.Center
                                ) {
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
                                
                                Spacer(modifier = Modifier.width(4.dp))
                                
                                val displayName = activeMsg?.author?.global_name ?: activeMsg?.author?.username ?: "Unknown"
                                val text = if (isEditing) {
                                    AnnotatedString("Editing message")
                                } else {
                                    AnnotatedString.Builder().apply {
                                        append("Replying to ")
                                        withStyle(SpanStyle(
                                            fontWeight = FontWeight.Bold,
                                            color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.onSurface
                                        )) {
                                            append(displayName)
                                        }
                                    }.toAnnotatedString()
                                }

                                Text(
                                    text = text,
                                    style = MaterialTheme.typography.labelMedium,
                                    modifier = Modifier.weight(1f),
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )

                                if (showReplyMentionToggle) {
                                    val mentionOn = navigationStore.shouldMentionReply
                                    TextButton(
                                        onClick = { 
                                            haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                            navigationStore.shouldMentionReply = !navigationStore.shouldMentionReply 
                                        },
                                        contentPadding = PaddingValues(horizontal = 8.dp),
                                        modifier = Modifier.height(32.dp)
                                    ) {
                                        Text(
                                            text = if (mentionOn) "@ ON" else "@ OFF",
                                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Black),
                                            color = if (mentionOn) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                        )
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
                                enter = if (reduceMotion) EnterTransition.None else expandVertically() + fadeIn(),
                                exit = if (reduceMotion) ExitTransition.None else shrinkVertically() + fadeOut()
                            ) {
                                MediaPicker(onDismiss = {
                                    navigationStore.isMediaPickerVisible = false
                                })
                            }
                        }

                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(start = 8.dp, end = 8.dp, top = 2.dp, bottom = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            val density = LocalDensity.current
                            val isKeyboardVisible = WindowInsets.ime.getBottom(density) > 0
                            val showCollapsibleItems = textFieldValue.text.isEmpty() && (!isFocused || !isKeyboardVisible)

                            val uploadVisible = !settings.chatboxHideUploadButton && messageStore.editingMessage == null && canSend
                            val voiceVisible = !settings.chatboxHideVoiceButton &&
                                messageStore.editingMessage == null &&
                                showCollapsibleItems &&
                                canSend &&
                                getPlatformName() == "android"
                            
                            if (uploadVisible) {
                                if (getPlatformName() != "android") {
                                    FilePicker(
                                        show = showFilePicker,
                                        onFileSelected = { it -> messageStore.pendingFiles.addAll(it.map { PendingFile(it.first, it.second) }) },
                                        onDismiss = { showFilePicker = false }
                                    )
                                }

                                FilledIconButton(
                                    onClick = {
                                        if (getPlatformName() == "android") {
                                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                            navigationStore.isMediaPickerVisible = !navigationStore.isMediaPickerVisible
                                        } else {
                                            showFilePicker = true
                                        }
                                    },
                                    modifier = Modifier.size(buttonSize),
                                    colors = IconButtonDefaults.filledIconButtonColors(
                                        containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                ) {
                                    Icon(
                                        imageVector = Icons.Filled.Add,
                                        contentDescription = "Add",
                                        modifier = Modifier.size(iconSize)
                                    )
                                }
                                
                                Spacer(modifier = Modifier.width(4.dp))
                            }

                            AnimatedVisibility(
                                visible = voiceVisible,
                                enter = if (reduceMotion) EnterTransition.None else expandHorizontally(expandFrom = Alignment.Start) + fadeIn(),
                                exit = if (reduceMotion) ExitTransition.None else shrinkHorizontally(shrinkTowards = Alignment.Start) + fadeOut()
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    VoiceMessageRecorder(
                                        enabled = true,
                                        buttonSize = buttonSize,
                                        iconSize = iconSize,
                                        onRecordingReady = { messageStore.pendingFiles.add(it) }
                                    )
                                    Spacer(modifier = Modifier.width(4.dp))
                                }
                            }

                            AnimatedVisibility(
                                visible = settings.chatboxShowAvatar && currentUser != null && showCollapsibleItems,
                                enter = if (reduceMotion) EnterTransition.None else expandHorizontally(expandFrom = Alignment.Start) + fadeIn(),
                                exit = if (reduceMotion) ExitTransition.None else shrinkHorizontally(shrinkTowards = Alignment.Start) + fadeOut()
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    AvatarWithDecoration(
                                        avatarUrl = CdnUrls.getUserAvatarUrl(currentUser!!.id, currentUser!!.avatar, 128),
                                        decorationData = currentUser!!.avatar_decoration_data,
                                        size = buttonSize,
                                        status = null,
                                        modifier = Modifier
                                            .clickableCursor {
                                                profileStore.showProfile(currentUser!!.id, navigationStore.selectedGuild?.id)
                                            }
                                    )
                                    Spacer(Modifier.width(4.dp))
                                }
                            }

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
                                    .heightIn(min = buttonSize)
                                    .animateContentSize(
                                        animationSpec = if (reduceMotion) snap() else spring(stiffness = Spring.StiffnessMediumLow)
                                    ),
                                shape = RoundedCornerShape(settings.chatboxBorderRadius.dp),
                                color = MaterialTheme.colorScheme.surfaceContainerHigh
                            ) {
                                ContextMenu(
                                    items = listOf(
                                        ContextMenuItem(
                                            label = "Paste image",
                                            onClick = {
                                                val pasted = getClipboardFiles()
                                                if (pasted.isNotEmpty()) {
                                                    messageStore.pendingFiles.addAll(pasted.map { PendingFile(it.first, it.second) })
                                                } else {
                                                    showToast("No image found in clipboard")
                                                }
                                            }
                                        )
                                    ),
                                    enabled = isMobileView && canSend,
                                    respectChildGestures = true
                                ) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp),
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        BasicTextField(
                                            value = textFieldValue,
                                            enabled = canSend,
                                            readOnly = !canSend,
                                            onValueChange = { 
                                                if (canSend) {
                                                    val oldText = textFieldValue.text
                                                    textFieldValue = it
                                                    mentionRanges = shiftMentionRanges(mentionRanges, oldText, it.text)
                                                
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
                                                        autocompleteStore.updateAutocomplete(type, query, navigationStore.selectedGuild, channel)
                                                    } else {
                                                        autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild, channel)
                                                    }

                                                    if (it.text.isNotEmpty() && !it.text.startsWith('/')) messageStore.sendTyping(channel.id)
                                                }
                                            },
                                            visualTransformation = DiscordInputVisualTransformation(primaryColor),
                                            modifier = Modifier
                                                .weight(1f)
                                                .padding(start = 12.dp, top = 8.dp, bottom = 8.dp)
                                                .focusRequester(focusRequester)
                                                .onFocusChanged { isFocused = it.isFocused }
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
                                                                            commandStore.resetSubCommand()
                                                                            textFieldValue = TextFieldValue("")
                                                                            clearMentions()
                                                                            autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild, channel)
                                                                        } else {
                                                                            applyAutocomplete(item)
                                                                        }
                                                                        return@onPreviewKeyEvent true
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if (event.key == Key.Escape) {
                                                            if (autocompleteStore.autocompleteType != null) {
                                                                autocompleteStore.updateAutocomplete(null, "", navigationStore.selectedGuild, channel)
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
                                                                    commandStore.activeCommand = command
                                                                    commandStore.resetSubCommand()
                                                                    textFieldValue = TextFieldValue("")
                                                                    clearMentions()
                                                                    return@onPreviewKeyEvent true
                                                                }
                                                            }
                                                            if (currentText.isNotBlank() || messageStore.pendingFiles.isNotEmpty()) {
                                                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                                                if (messageStore.editingMessage != null) {
                                                                    messageStore.editMessage(messageStore.editingMessage!!, resolveContent(currentText))
                                                                    messageStore.editingMessage = null
                                                                } else {
                                                                    messageStore.sendMessageDraft(resolveContent(currentText), allowedMentions = allowedMentions)
                                                                }
                                                                textFieldValue = TextFieldValue("")
                                                                clearMentions()
                                                                messageStore.resetTypingEmission(channel.id)
                                                                return@onPreviewKeyEvent true
                                                            }
                                                        }
                                                    }
                                                    false
                                                },
                                            textStyle = MaterialTheme.typography.bodyLarge.copy(
                                                color = MaterialTheme.colorScheme.onSurface,
                                                fontSize = MaterialTheme.typography.bodyLarge.fontSize * chatboxFontSize
                                            ),
                                            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                                            decorationBox = { innerTextField: @Composable () -> Unit ->
                                                Box(modifier = Modifier.fillMaxWidth()) {
                                                    if (textFieldValue.text.isEmpty()) {
                                                        Text(
                                                            text = placeholderText,
                                                            style = MaterialTheme.typography.bodyLarge.copy(
                                                                fontSize = MaterialTheme.typography.bodyLarge.fontSize * chatboxFontSize
                                                            ),
                                                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                                                            maxLines = 1,
                                                            overflow = TextOverflow.Ellipsis
                                                        )
                                                    }
                                                    innerTextField()
                                                }
                                            }
                                        )

                                        val silentTypingEnabled = settingsStore.silentTyping
                                        if (settingsStore.silentTypingButtonEnabled && canSend) {
                                            IconButton(
                                                onClick = {
                                                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                                    settingsStore.silentTyping = !settingsStore.silentTyping
                                                },
                                                modifier = Modifier.size(36.dp)
                                            ) {
                                                Icon(
                                                    imageVector = if (silentTypingEnabled) Icons.Filled.KeyboardOff else Icons.Filled.Keyboard,
                                                    contentDescription = if (silentTypingEnabled) "Silent typing enabled" else "Silent typing disabled",
                                                    modifier = Modifier.size(24.dp),
                                                    tint = if (silentTypingEnabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                        }

                                        if (!settings.chatboxHideEmojiButton && canSend) {
                                            IconButton(
                                                onClick = { 
                                                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
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
                                                    modifier = Modifier.size(24.dp),
                                                    tint = if (navigationStore.isEmojiPickerVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                                )
                                            }
                                        }

                                        if (navigationStore.isEmojiPickerVisible && canSend && !isMobileView) {
                                            androidx.compose.ui.window.Popup(
                                                alignment = Alignment.BottomEnd,
                                                offset = IntOffset(0, -48),
                                                onDismissRequest = { navigationStore.isEmojiPickerVisible = false },
                                                properties = androidx.compose.ui.window.PopupProperties(focusable = true)
                                            ) {
                                                EmojiPicker { emoji ->
                                                    insertEmoji(emoji)
                                                    navigationStore.isEmojiPickerVisible = false
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            val showSend = textFieldValue.text.isNotBlank() || messageStore.pendingFiles.isNotEmpty() || commandStore.activeCommand != null
                            
                            AnimatedVisibility(
                                visible = showSend,
                                enter = if (reduceMotion) EnterTransition.None else expandHorizontally(expandFrom = Alignment.Start) + fadeIn() + scaleIn(initialScale = 0.8f),
                                exit = if (reduceMotion) ExitTransition.None else shrinkHorizontally(shrinkTowards = Alignment.Start) + fadeOut() + scaleOut(targetScale = 0.8f),
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Spacer(modifier = Modifier.width(4.dp))
                                    FilledIconButton(
                                        onClick = {
                                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                            if (commandStore.activeCommand != null) {
                                                val options = commandStore.buildInteractionOptions()
                                                commandStore.sendInteraction(
                                                    command = commandStore.activeCommand!!,
                                                    guildId = navigationStore.selectedGuild?.id,
                                                    channelId = channel.id,
                                                    options = options
                                                )
                                                commandStore.activeCommand = null
                                                commandStore.commandOptions.clear()
                                                commandStore.resetSubCommand()
                                            } else {
                                                if (messageStore.editingMessage != null) {
                                                    messageStore.editMessage(messageStore.editingMessage!!, resolveContent(textFieldValue.text))
                                                    messageStore.editingMessage = null
                                                } else {
                                                    messageStore.sendMessageDraft(resolveContent(textFieldValue.text), allowedMentions = allowedMentions)
                                                }
                                            }
                                            textFieldValue = TextFieldValue("")
                                            clearMentions()
                                            messageStore.resetTypingEmission(channel.id)
                                        },
                                        enabled = canSend && (commandStore.activeCommand == null || commandStore.isCommandValid()),
                                        colors = IconButtonDefaults.filledIconButtonColors(
                                            containerColor = MaterialTheme.colorScheme.primary,
                                            contentColor = MaterialTheme.colorScheme.onPrimary
                                        ),
                                        modifier = Modifier.size(buttonSize)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Rounded.Send,
                                            contentDescription = "Send",
                                            modifier = Modifier.size(iconSize)
                                        )
                                    }
                                }
                            }
                        }

                        AnimatedVisibility(
                            visible = navigationStore.isEmojiPickerVisible && canSend && !isDesktopTarget,
                            enter = if (reduceMotion) EnterTransition.None else expandVertically() + fadeIn(),
                            exit = if (reduceMotion) ExitTransition.None else shrinkVertically() + fadeOut()
                        ) {
                            PlatformBackHandler(enabled = navigationStore.isEmojiPickerVisible) {
                                navigationStore.isEmojiPickerVisible = false
                            }
                            EmojiPicker(
                                modifier = Modifier.fillMaxWidth(),
                                onKeyboardClick = {
                                    navigationStore.isEmojiPickerVisible = false
                                    focusRequester.requestFocus()
                                },
                                onEmojiSelected = { emoji ->
                                    insertEmoji(emoji)
                                }
                            )
                        }
                    }
                }
            }
        }
    }
}

private fun shiftMentionRanges(
    ranges: Map<IntRange, String>,
    editStart: Int,
    editOldEnd: Int,
    delta: Int
): Map<IntRange, String> {
    return ranges.mapNotNull { (range, value) ->
        when {
            range.last < editStart -> range to value
            range.first >= editOldEnd -> (range.first + delta)..(range.last + delta) to value
            else -> null
        }
    }.toMap()
}

private fun shiftMentionRanges(ranges: Map<IntRange, String>, oldText: String, newText: String): Map<IntRange, String> {
    val editStart = oldText.commonPrefixWith(newText).length
    val commonSuffixLen = oldText.commonSuffixWith(newText).length
    val editOldEnd = oldText.length - commonSuffixLen
    val delta = newText.length - oldText.length
    return shiftMentionRanges(ranges, editStart, editOldEnd, delta)
}

private fun resolveServerContent(
    content: String,
    ranges: Map<IntRange, String>,
    channels: List<Channel>,
    members: List<Member>,
    roles: List<Role>,
    emojis: List<me.lampu.lampcord.shared.model.Emoji>,
    currentUser: me.lampu.lampcord.shared.model.User?,
    selectedGuildId: String?
): String {
    var result = content

    for ((range, value) in ranges.entries.sortedByDescending { it.key.first }) {
        if (range.first >= 0 && range.last < result.length && range.first <= range.last) {
            result = result.replaceRange(range.first, range.last + 1, value)
        }
    }

    val pattern = Regex("""(?<![<\p{L}\p{N}_])(?:([#@])([^\s]+)|(:)([a-zA-Z0-9_-]+)(:))""")
    val subs = pattern.findAll(result).mapNotNull { m ->
        val trigger = m.groupValues[1].ifEmpty { m.groupValues[3] }
        val token = if (trigger == ":") m.groupValues[4] else m.groupValues[2].trimEnd(',', '.', '!', '?', ';', ':', '"', '\'', ')', ']', '}').removeSuffix(":")

        val replacement = when (trigger) {
            "#" -> channels.firstOrNull { it.name?.equals(token, ignoreCase = true) == true }
                ?.let { "<#${it.id}>" }
            "@" -> when {
                token == "everyone" || token == "here" -> null
                else -> {
                    val role = roles.firstOrNull { it.name.equals(token, ignoreCase = true) }
                    if (role != null) {
                        "<@&${role.id}>"
                    } else {
                        val member = members.firstOrNull { mem ->
                            mem.nick?.equals(token, ignoreCase = true) == true ||
                            mem.user?.global_name?.equals(token, ignoreCase = true) == true ||
                            mem.user?.username?.equals(token, ignoreCase = true) == true
                        }
                        member?.user?.id?.let { "<@$it>" }
                    }
                }
            }
            ":" -> {
                val parts = token.split('-')
                val baseName = if (parts.size > 1 && parts.last().toIntOrNull() != null) {
                    token.substringBeforeLast('-')
                } else token
                val index = if (parts.size > 1) parts.last().toIntOrNull() ?: 0 else 0

                val matches = emojis.filter { it.name?.equals(baseName, ignoreCase = true) == true }
                    .sortedWith(compareBy({ it.guild_id != selectedGuildId }, { it.id }))
                val emoji = matches.getOrNull(index)
                if (emoji != null) {
                    me.lampu.lampcord.shared.utils.FreeNitroEmojis.getReplacement(emoji, currentUser, selectedGuildId)
                } else null
            }
            else -> null
        }
        if (replacement != null) m.range to replacement else null
    }.toList()
    subs.sortedByDescending { it.first.first }.forEach { (range, value) ->
        result = result.replaceRange(range.first, range.last + 1, value)
    }

    val unicodeRegex = Regex(""":([a-zA-Z0-9_+-]+):""")
    result = unicodeRegex.replace(result) { m ->
        val unmatchedBackticks = result.substring(0, m.range.first).count { it == '`' } % 2
        if (unmatchedBackticks == 1) m.value
        else EmojiIndex.getCharForName(m.groupValues[1]) ?: m.value
    }

    return result
}
