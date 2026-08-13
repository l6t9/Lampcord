package me.lampu.lampcord.shared.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import kotlinx.coroutines.launch
import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.DateSeparator
import me.lampu.lampcord.shared.ui.components.DiscordInputVisualTransformation
import me.lampu.lampcord.shared.ui.components.EmojiPicker
import me.lampu.lampcord.shared.ui.components.chat.MessageItem
import me.lampu.lampcord.shared.ui.components.chat.MediaPicker
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.UserAvatar
import me.lampu.lampcord.shared.ui.theme.LampcordTheme
import org.koin.compose.koinInject
import kotlin.time.Duration.Companion.minutes

@Composable
fun BubbleScreen(channelId: String, guildId: String?) {
    val settingsStore: SettingsStore = koinInject()

    val useDarkTheme = when (settingsStore.themeMode) {
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
        ThemeMode.AUTO -> isSystemInDarkTheme()
    }
    val pureBlack = settingsStore.pureBlack && useDarkTheme
    val seedColor = remember(settingsStore.accentColor) {
        try {
            Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
        } catch (e: Exception) {
            Color(0xFF6750A4)
        }
    }

    LampcordTheme(
        useDarkTheme = useDarkTheme,
        pureBlack = pureBlack,
        seedColor = seedColor,
        paletteStyle = settingsStore.themePaletteStyle,
        useMaterialYou = settingsStore.materialYou,
        appFont = settingsStore.appFont,
        fontScale = settingsStore.fontScale,
        customFontPath = settingsStore.customFontPath,
    ) {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.surface) {
            BubbleConversation(channelId = channelId, guildId = guildId)
        }
    }
}

@Composable
private fun BubbleConversation(
    channelId: String,
    guildId: String?,
    discordClient: DiscordClient = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    entityStore: EntityStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    json: kotlinx.serialization.json.Json = koinInject()
) {
    val scope = rememberCoroutineScope()
    val messages = remember { mutableStateListOf<Message>() }
    var input by remember { mutableStateOf("") }
    val listState = rememberLazyListState()
    var isSending by remember { mutableStateOf(false) }
    var showMediaPicker by remember { mutableStateOf(false) }

    val channels by entityStore.channels.collectAsState()
    val channel = channels[channelId]
    val recipients = channel?.recipients
    val title = when {
        guildId != null && channel?.name != null -> "#${channel.name}"
        channel?.name != null -> channel.name
        !recipients.isNullOrEmpty() ->
            recipients.joinToString(", ") { it.global_name ?: it.username ?: "Unknown" }
        else -> "Conversation"
    }

    LaunchedEffect(channelId) {
        try {
            val history = discordClient.getChannelMessages(channelId, 50)
            messages.clear()
            messages.addAll(history.reversed())
        } catch (e: Exception) {
        }
    }

    LaunchedEffect(channelId) {
        gatewayManager.events.collect { payload ->
            val d = payload.d ?: return@collect
            try {
                when (payload.t) {
                    "MESSAGE_CREATE" -> {
                        val message = json.decodeFromJsonElement<Message>(d)
                        if (message.channel_id == channelId) {
                            messages.indexOfFirst { it.id == message.id }
                                .takeIf { it == -1 }
                                ?.let { messages.add(message) }
                        }
                    }
                    "MESSAGE_UPDATE" -> {
                        val dataObj = d as? JsonObject ?: return@collect
                        val messageId = dataObj["id"]?.jsonPrimitive?.content ?: return@collect
                        val index = messages.indexOfFirst { it.id == messageId }
                        if (index != -1) {
                            messages[index] = messages[index].merge(dataObj)
                        }
                    }
                    "MESSAGE_DELETE" -> {
                        val dataObj = d as? JsonObject ?: return@collect
                        if (dataObj["channel_id"]?.jsonPrimitive?.content == channelId) {
                            val id = dataObj["id"]?.jsonPrimitive?.content ?: return@collect
                            messages.removeAll { it.id == id }
                        }
                    }
                }
            } catch (e: Exception) {
            }
        }
    }

    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            listState.animateScrollToItem(messages.size - 1)
        }
    }

    val density = LocalDensity.current
    val imeBottom = WindowInsets.ime.getBottom(density) + WindowInsets.navigationBars.getBottom(density)
    LaunchedEffect(imeBottom) {
        if (imeBottom > 0 && messages.isNotEmpty()) {
            listState.scrollToItem(messages.size - 1)
        }
    }

    LaunchedEffect(messageStore.editingMessage?.id) {
        messageStore.editingMessage?.let {
            input = it.content
            messageStore.replyingTo = null
        }
    }

    fun sendToChannel(text: String) {
        if (isSending) return
        val content = text.trim()
        val files = messageStore.pendingFiles.toList()
        val replyTo = messageStore.replyingTo
        val editing = messageStore.editingMessage
        if (content.isBlank() && files.isEmpty()) return
        if (editing != null && files.isNotEmpty()) return
        isSending = true
        input = ""
        messageStore.pendingFiles.clear()
        scope.launch {
            try {
                if (editing != null) {
                    discordClient.editMessage(
                        channelId = editing.channel_id,
                        messageId = editing.id,
                        content = content
                    )
                } else {
                    discordClient.sendMessage(
                        channelId = channelId,
                        content = content,
                        replyTo = replyTo?.id,
                        files = files.map { it.name to it.data }
                    )
                }
            } catch (e: Exception) {
            } finally {
                isSending = false
                if (editing != null) messageStore.editingMessage = null
                messageStore.replyingTo = null
            }
        }
    }

    Column(modifier = Modifier.fillMaxSize().imePadding()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surfaceContainer)
                .padding(horizontal = 16.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            BubbleChannelAvatar(channel = channel)
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier
                    .weight(1f)
                    .padding(start = 8.dp)
            )
        }

        LazyColumn(
            state = listState,
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
        ) {
            itemsIndexed(
                items = messages,
                key = { _, message -> message.id }
            ) { index, message ->
                val priorMessage = messages.getOrNull(index - 1)

                val showDateSeparator = remember(message, priorMessage) {
                    if (priorMessage == null) return@remember true
                    try {
                        val currentTs = Instant.parse(message.timestamp)
                            .toLocalDateTime(TimeZone.currentSystemDefault()).date
                        val priorTs = Instant.parse(priorMessage.timestamp)
                            .toLocalDateTime(TimeZone.currentSystemDefault()).date
                        currentTs != priorTs
                    } catch (e: Exception) {
                        false
                    }
                }

                val isInline = remember(message, priorMessage, showDateSeparator) {
                    if (priorMessage == null) return@remember false
                    if (showDateSeparator) return@remember false
                    if (priorMessage.author?.id != message.author?.id) return@remember false
                    if (message.referenced_message != null) return@remember false
                    if (message.type != 0) return@remember false
                    try {
                        val currentTs = Instant.parse(message.timestamp)
                        val priorTs = Instant.parse(priorMessage.timestamp)
                        (currentTs - priorTs) < 7.minutes
                    } catch (e: Exception) {
                        false
                    }
                }

                Column {
                    if (showDateSeparator) {
                        DateSeparator(message.timestamp)
                    }
                    MessageItem(
                        message = message,
                        priorMessage = if (isInline) priorMessage else null
                    )
                }
            }
        }

        BubbleComposer(
            channelId = channelId,
            channel = channel,
            onSend = ::sendToChannel,
            messageStore = messageStore,
            isSending = isSending,
            input = input,
            onInputChange = { input = it },
            onAddMedia = { showMediaPicker = true }
        )

        if (showMediaPicker) {
            MediaPicker(onDismiss = { showMediaPicker = false })
        }
    }
}

@Composable
private fun BubbleChannelAvatar(
    channel: Channel?
) {
    if (channel == null) return
    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
    val user = if (isDm) channel.recipients?.firstOrNull() else null
    if (user != null) {
        UserAvatar(user = user, size = 28.dp)
    }
}

@Composable
private fun BubbleComposer(
    channelId: String,
    channel: Channel?,
    messageStore: MessageStore,
    isSending: Boolean,
    input: String,
    onInputChange: (String) -> Unit,
    onSend: (String) -> Unit,
    onAddMedia: () -> Unit
) {
    var showEmojiPicker by remember { mutableStateOf(false) }
    val primaryColor = MaterialTheme.colorScheme.primary
    val isDm = channel?.type == 1 || channel?.type == 3 || channel?.guild_id == null
    val placeholderText = if (isDm) {
        val recipient = channel?.recipients?.firstOrNull()
        val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
        "Message @$name"
    } else {
        "Message #${channel.name ?: "unnamed"}"
    }

    fun send() {
        val text = input.trim()
        if ((text.isNotBlank() || messageStore.pendingFiles.isNotEmpty()) && !isSending) {
            onSend(text)
        }
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        val editing = messageStore.editingMessage
        val replying = messageStore.replyingTo
        if (editing != null || replying != null) {
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = MaterialTheme.colorScheme.surfaceContainerLow
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 12.dp, vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = if (editing != null) Icons.Filled.Edit else Icons.Rounded.Reply,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = if (editing != null) "Editing message" else "Replying to ${replying?.author?.global_name ?: replying?.author?.username}",
                        style = MaterialTheme.typography.labelSmall,
                        modifier = Modifier.weight(1f)
                    )
                    IconButton(
                        onClick = {
                            if (editing != null) {
                                messageStore.editingMessage = null
                                onInputChange("")
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
                    .padding(horizontal = 8.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                messageStore.pendingFiles.forEachIndexed { index, pendingFile ->
                    Surface(
                        shape = RoundedCornerShape(8.dp),
                        color = MaterialTheme.colorScheme.surfaceContainerHigh
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(start = 8.dp, top = 2.dp, bottom = 2.dp, end = 4.dp)
                        ) {
                            Text(
                                text = pendingFile.name,
                                style = MaterialTheme.typography.labelSmall,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            IconButton(
                                onClick = { messageStore.pendingFiles.removeAt(index) },
                                modifier = Modifier.size(20.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Close,
                                    contentDescription = null,
                                    modifier = Modifier.size(14.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                }
            }
        }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        IconButton(
            onClick = onAddMedia,
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

        Surface(
            modifier = Modifier
                .weight(1f)
                .heightIn(min = 40.dp),
            shape = RoundedCornerShape(16.dp),
            color = MaterialTheme.colorScheme.surfaceContainerHigh
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                BasicTextField(
                    value = input,
                    onValueChange = {
                        onInputChange(it)
                        if (it.isNotEmpty()) {
                            messageStore.sendTyping(channelId)
                        }
                    },
                    visualTransformation = DiscordInputVisualTransformation(primaryColor),
                    textStyle = MaterialTheme.typography.bodyMedium.copy(
                        color = MaterialTheme.colorScheme.onSurface
                    ),
                    cursorBrush = SolidColor(primaryColor),
                    modifier = Modifier
                        .weight(1f)
                        .padding(start = 12.dp, top = 8.dp, bottom = 8.dp)
                        .onPreviewKeyEvent { event ->
                            if (event.key == Key.Enter && !event.isShiftPressed) {
                                if (event.type == KeyEventType.KeyDown) {
                                    send()
                                }
                                true
                            } else {
                                false
                            }
                        }
                ) {
                    if (input.isEmpty()) {
                        Text(
                            text = placeholderText,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                    }
                    it()
                }

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
                    Popup(
                        alignment = Alignment.BottomEnd,
                        offset = IntOffset(0, -48),
                        onDismissRequest = { showEmojiPicker = false }
                    ) {
                        EmojiPicker { emoji ->
                            val emojiText = if (emoji.id != null) {
                                "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>"
                            } else emoji.name ?: ""
                            onInputChange(input + emojiText)
                            showEmojiPicker = false
                        }
                    }
                }
            }
        }

        AnimatedVisibility(
            visible = input.isNotBlank() || messageStore.pendingFiles.isNotEmpty(),
            enter = fadeIn() + scaleIn(initialScale = 0.8f) + slideInHorizontally { it },
            exit = fadeOut() + scaleOut(targetScale = 0.8f) + slideOutHorizontally { it },
            modifier = Modifier.padding(start = 8.dp)
        ) {
            IconButton(
                onClick = { send() },
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