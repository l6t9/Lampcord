package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.TextButton
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.ui.input.pointer.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import kotlin.math.roundToInt
import me.lampu.lampcord.shared.model.toTwemojiUrl
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.utils.EmojiIndex
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject

@Composable
fun DiscordMarkdownText(
    content: String,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    color: Color = Color.Unspecified,
    maxLines: Int = Int.MAX_VALUE,
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    remember { EmojiIndex.initialize() }
    var revealedSpoilers by remember { mutableStateOf(setOf<Int>()) }
    val primaryColor = MaterialTheme.colorScheme.primary
    
    val fontSize = style.fontSize.takeIf { it.isSp } ?: 16.sp
    
    val isJumbo = remember(content) {
        if (maxLines != Int.MAX_VALUE) return@remember false
        val trimmed = content.trim()
        if (trimmed.isEmpty()) return@remember false
        val customEmojiRegex = me.lampu.lampcord.shared.utils.FreeNitroEmojis.emojiRegex
        var temp = trimmed
        var count = 0
        customEmojiRegex.findAll(trimmed).forEach { 
            count++
            temp = temp.replace(it.value, "")
        }
        var scanIdx = 0
        while (scanIdx < temp.length) {
            val found = EmojiIndex.findEmojiInString(temp, scanIdx)
            if (found != null) {
                count++
                scanIdx += found.second
            } else {
                if (!temp[scanIdx].isWhitespace()) return@remember false
                scanIdx++
            }
        }
        count in 1..30
    }

    val emojiSize = if (isJumbo) (fontSize.value * 2.8f).sp else (fontSize.value * 1.4f).sp
    
    val processedContent = remember(content) {
        content.split('\n').joinToString("\n") { line ->
            val leadingSpaces = line.takeWhile { it == ' ' }.length
            if (leadingSpaces > 0 && line.trim().isNotEmpty()) {
                "\u00A0".repeat(leadingSpaces) + line.substring(leadingSpaces)
            } else {
                line
            }
        }
    }

    val allGuildChannels by guildStore.allGuildChannels.collectAsState()

    val annotatedString = remember(processedContent, revealedSpoilers, primaryColor, allGuildChannels.size) {
        buildAnnotatedString {
            appendDiscordMarkdown(
                content = processedContent,
                revealedSpoilers = revealedSpoilers,
                primaryColor = primaryColor,
                navigationStore = navigationStore,
                guildStore = guildStore,
                userStore = userStore,
                profileStore = profileStore,
                onSpoilerClick = { index ->
                    revealedSpoilers = revealedSpoilers + index
                }
            )
        }
    }

    val inlineContent = remember(annotatedString, emojiSize) {
        val map = mutableMapOf<String, InlineTextContent>()
        annotatedString.getStringAnnotations("EMOJI", 0, annotatedString.length).forEach { annotation ->
            val parts = annotation.item.split(":")
            val id = parts[0]
            val animated = parts[1] == "a"
            val name = parts[2]
            val url = "https://cdn.discordapp.com/emojis/$id.webp?size=44&animated=$animated"
            
            map[annotation.item] = InlineTextContent(
                Placeholder(emojiSize, emojiSize, PlaceholderVerticalAlign.Center)
            ) {
                AsyncImage(
                    model = url,
                    contentDescription = name,
                    modifier = Modifier.fillMaxSize(),
                    filterQuality = FilterQuality.Medium,
                    showPlaceholder = false
                )
            }
        }
        annotatedString.getStringAnnotations("UNICODE_EMOJI", 0, annotatedString.length).forEach { annotation ->
            val emoji = annotation.item
            val url = emoji.toTwemojiUrl()
            val key = "UNICODE_EMOJI:$emoji:${annotation.start}"
            map[key] = InlineTextContent(
                Placeholder(emojiSize, emojiSize, PlaceholderVerticalAlign.Center)
            ) {
                var loadFailed by remember { mutableStateOf(false) }
                
                Box(contentAlignment = Alignment.Center) {
                    if (!loadFailed && url.isNotEmpty()) {
                        AsyncImage(
                            model = url,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize(),
                            filterQuality = FilterQuality.Medium,
                            showPlaceholder = false,
                            onState = { state ->
                                if (state is coil3.compose.AsyncImagePainter.State.Error) {
                                    loadFailed = true
                                }
                            }
                        )
                    }
                    if (loadFailed || url.isEmpty()) {
                        Text(text = emoji, fontSize = if (isJumbo) 34.sp else 17.sp)
                    }
                }
            }
        }
        map
    }

    val uriHandler = LocalUriHandler.current
    var textLayoutResult by remember { mutableStateOf<TextLayoutResult?>(null) }
    var contextMenuUrl by remember { mutableStateOf<String?>(null) }
    var contextMenuOffset by remember { mutableStateOf(IntOffset.Zero) }
    val textContent: @Composable () -> Unit = {
        Box {
            Text(
                text = annotatedString,
                modifier = modifier
                    .pointerInput(annotatedString) {
                        if (getPlatformName() == "android") {
                            awaitEachGesture {
                                var downEvent: PointerEvent
                                var down: PointerInputChange?
                                do {
                                    downEvent = awaitPointerEvent(PointerEventPass.Main)
                                    down = downEvent.changes.firstOrNull { it.changedToDown() }
                                } while (down == null)

                                // Leave mouse secondary-clicks to the pointer
                                // handler below, which shows the desktop link menu.
                                if (downEvent.buttons.isSecondaryPressed) return@awaitEachGesture

                                val pressed = down
                                val offset = textLayoutResult?.getOffsetForPosition(pressed.position)
                                val mentionsAtOffset = offset?.let {
                                    annotatedString.getStringAnnotations("MENTION", it, it)
                                        .firstOrNull()
                                        ?.item
                                }
                                val url = offset?.let {
                                    annotatedString.getStringAnnotations("URL", it, it)
                                        .firstOrNull()
                                        ?.item
                                }

                                // Leave ordinary text untouched so the
                                // enclosing message context menu can handle it.
                                if (url == null && mentionsAtOffset == null) return@awaitEachGesture

                                // Link presses are owned by this text gesture
                                // and must not open the parent message menu.
                                var isSlopExceeded = false
                                val completedTap = withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) {
                                    while (true) {
                                        val event = awaitPointerEvent(PointerEventPass.Main)
                                        val change = event.changes.firstOrNull { it.id == pressed.id }
                                            ?: continue
                                        if (change.isConsumed) return@withTimeoutOrNull false
                                        if ((change.position - pressed.position).getDistance() > viewConfiguration.touchSlop) {
                                            isSlopExceeded = true
                                            return@withTimeoutOrNull false
                                        }
                                        if (!change.pressed) {
                                            change.consume()
                                            return@withTimeoutOrNull true
                                        }
                                    }
                                }

                                when {
                                        isSlopExceeded -> {
                                            // Ignore tap if pointer moved beyond touch slop.
                                        }
                                        completedTap == null -> {
                                            if (url != null) {
                                                contextMenuUrl = url
                                                contextMenuOffset = IntOffset(
                                                    pressed.position.x.roundToInt(),
                                                    pressed.position.y.roundToInt()
                                                )
                                                pressed.consume()
                                            }
                                        }
                                        completedTap == true -> {
                                            val mentionId = mentionsAtOffset
                                            if (url != null) uriHandler.openUri(url)
                                            else if (mentionId != null) {
                                                profileStore.showProfile(mentionId, navigationStore.selectedGuild?.id)
                                                pressed.consume()
                                            }
                                        }
                                    }
                            }
                        } else {
                            detectTapGestures(
                                onTap = { position ->
                                    val offset = textLayoutResult?.getOffsetForPosition(position)
                                        ?: return@detectTapGestures
                                    val url = annotatedString.getStringAnnotations("URL", offset, offset)
                                        .firstOrNull()
                                        ?.item
                                    if (url != null) {
                                        uriHandler.openUri(url)
                                    } else {
                                        annotatedString.getStringAnnotations("MENTION", offset, offset)
                                            .firstOrNull()
                                            ?.item
                                            ?.let { profileStore.showProfile(it, navigationStore.selectedGuild?.id) }
                                    }
                                }
                            )
                        }
                    }
                    .pointerInput(annotatedString, textLayoutResult) {
                        awaitPointerEventScope {
                            while (true) {
                                val event = awaitPointerEvent(PointerEventPass.Main)
                                val down = event.changes.firstOrNull { it.changedToDown() }
                                if (down != null && event.buttons.isSecondaryPressed) {
                                    val offset = textLayoutResult?.getOffsetForPosition(down.position)
                                        ?: continue
                                    val url = annotatedString.getStringAnnotations("URL", offset, offset)
                                        .firstOrNull()
                                        ?.item
                                        ?: continue
                                    contextMenuUrl = url
                                    contextMenuOffset = IntOffset(
                                        down.position.x.roundToInt(),
                                        down.position.y.roundToInt()
                                    )
                                    down.consume()
                                }
                            }
                        }
                    },
                style = style.copy(color = if (color != Color.Unspecified) color else LocalContentColor.current),
                inlineContent = inlineContent,
                onTextLayout = { textLayoutResult = it },
                maxLines = maxLines,
                overflow = if (maxLines != Int.MAX_VALUE) TextOverflow.Ellipsis else TextOverflow.Clip
            )

            if (contextMenuUrl != null && getPlatformName() == "android") {
                val url = contextMenuUrl ?: return@Box
                AlertDialog(
                    onDismissRequest = { contextMenuUrl = null },
                    title = { Text("Link") },
                    text = { Text(url) },
                    confirmButton = {
                        TextButton(onClick = {
                            setClipboardText(url)
                            showToast("Copied link")
                            contextMenuUrl = null
                        }) { Text("Copy Link") }
                    },
                    dismissButton = {
                        TextButton(onClick = {
                            uriHandler.openUri(url)
                            contextMenuUrl = null
                        }) { Text("Open Link") }
                    }
                )
            } else if (contextMenuUrl != null) Popup(
                alignment = Alignment.TopStart,
                offset = contextMenuOffset,
                onDismissRequest = { contextMenuUrl = null },
                properties = PopupProperties(focusable = true)
            ) {
                Surface(
                    modifier = Modifier.width(180.dp),
                    shape = androidx.compose.foundation.shape.RoundedCornerShape(8.dp),
                    tonalElevation = 4.dp
                ) {
                    androidx.compose.foundation.layout.Column {
                        contextMenuUrl?.let { url ->
                            DropdownMenuItem(
                                text = { Text("Open Link") },
                                onClick = {
                                    uriHandler.openUri(url)
                                    contextMenuUrl = null
                                }
                            )
                            DropdownMenuItem(
                                text = { Text("Copy Link") },
                                onClick = {
                                    setClipboardText(url)
                                    showToast("Copied link")
                                    contextMenuUrl = null
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    textContent()
}

private fun AnnotatedString.Builder.appendDiscordMarkdown(
    content: String,
    revealedSpoilers: Set<Int>,
    primaryColor: Color,
    navigationStore: NavigationStore,
    guildStore: GuildStore,
    userStore: UserStore,
    profileStore: ProfileStore,
    onSpoilerClick: (Int) -> Unit
) {
    val patterns = listOf(
        // Code blocks
        Regex("""```(\w*)\n?([\s\S]*?)\n?```""") to "CODE_BLOCK",
        // Block quotes (Multi-line)
        Regex("""^>>> ([\s\S]*)$""", RegexOption.MULTILINE) to "BLOCKQUOTE_MULTI",
        // Block quotes (Single-line)
        Regex("""^> (.*)$""", RegexOption.MULTILINE) to "BLOCKQUOTE",
        // Headers
        Regex("""^### (.*)$""", RegexOption.MULTILINE) to "H3",
        Regex("""^## (.*)$""", RegexOption.MULTILINE) to "H2",
        Regex("""^# (.*)$""", RegexOption.MULTILINE) to "H1",
        // Bullets
        Regex("""^[*\-]\s+(.*)$""", RegexOption.MULTILINE) to "BULLET",
        // Subtext
        Regex("""^-# (.*)$""", RegexOption.MULTILINE) to "SUBTEXT",
        // Spoilers
        Regex("""\|\|([\s\S]+?)\|\|""") to "SPOILER",
        // Suppressed links
        Regex("""<(https?://[^>]+)>""") to "URL_SUPPRESSED",
        // Masked links. Discord also permits the destination to be wrapped
        // in angle brackets: [label](<https://example.com>).
        maskedLinkPattern to "MASKED_LINK",
        // Auto links
        Regex("""(https?://[^\s)>]+)""") to "URL",
        // Bold
        Regex("""\*\*([^*]+)\*\*""") to "BOLD",
        // Underline
        Regex("""__([^_]+)__""") to "UNDERLINE",
        // Italic
        Regex("""\*([^*]+)\*""") to "ITALIC",
        Regex("""_([^_]+)_""") to "ITALIC",
        // Strikethrough
        Regex("""~~([^~]+)~~""") to "STRIKE",
        // Inline code
        Regex("""`([^`]+)`""") to "CODE",
        // Custom Emojis
        me.lampu.lampcord.shared.utils.FreeNitroEmojis.emojiRegex to "EMOJI",
        // Timestamps
        Regex("""<t:(-?\d+)(?::([tTdDfFR]))?>""") to "TIMESTAMP",
        // Mentions
        Regex("""<@!?(\d+)>""") to "MENTION",
        Regex("""<#(\d+)>""") to "CHANNEL",
        Regex("""<@&(\d+)>""") to "ROLE",
        // Slash Commands
        Regex("""</([\w\- ]+):(\d+)>""") to "SLASH_COMMAND",
        Regex("""@(everyone)""") to "EVERYONE",
        Regex("""@(here)""") to "HERE"
    )

    val allMatches = mutableListOf<Triple<IntRange, MatchResult?, String>>()
    patterns.forEach { (regex, tag) ->
        regex.findAll(content).forEach { match ->
            allMatches.add(Triple(match.range, match, tag))
        }
    }
    
    // Add Unicode Emojis using EmojiIndex manual scan
    var scanIdx = 0
    while (scanIdx < content.length) {
        val found = EmojiIndex.findEmojiInString(content, scanIdx)
        if (found != null) {
            val (emoji, len) = found
            allMatches.add(Triple(scanIdx until (scanIdx + len), null, "UNICODE_EMOJI"))
            scanIdx += len
        } else {
            scanIdx++
        }
    }
    
    val sortedMatches = allMatches.sortedWith(compareBy({ it.first.first }, { -it.first.last }))
    
    var lastIndex = 0
    var i = 0
    while (i < sortedMatches.size) {
        val (range, match, tag) = sortedMatches[i]
        
        if (range.first < lastIndex) {
            i++
            continue
        }
        
        if (range.first > lastIndex) {
            append(content.substring(lastIndex, range.first))
        }
        
        when (tag) {
            "UNICODE_EMOJI" -> {
                val emoji = content.substring(range)
                val key = "UNICODE_EMOJI:$emoji:${range.first}"
                pushStringAnnotation("UNICODE_EMOJI", emoji)
                appendInlineContent(key, emoji)
                pop()
            }
            "H1" -> withStyle(SpanStyle(fontWeight = FontWeight.Bold, fontSize = 24.sp)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "H2" -> withStyle(SpanStyle(fontWeight = FontWeight.Bold, fontSize = 20.sp)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "H3" -> withStyle(SpanStyle(fontWeight = FontWeight.Bold, fontSize = 18.sp)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "BLOCKQUOTE", "BLOCKQUOTE_MULTI" -> {
                withStyle(style = SpanStyle(color = Color.Gray, background = Color.Gray.copy(alpha = 0.1f))) {
                    append("▎")
                    appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick)
                }
            }
            "SUBTEXT" -> withStyle(SpanStyle(fontSize = 12.sp, color = Color.Gray)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "BULLET" -> {
                append("  • ")
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick)
            }
            "SPOILER" -> {
                val spoilerText = match!!.groupValues[1]
                val index = range.first
                val isRevealed = revealedSpoilers.contains(index)
                if (isRevealed) {
                    withStyle(style = SpanStyle(background = Color.Gray.copy(alpha = 0.2f))) {
                        appendDiscordMarkdown(spoilerText, revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick)
                    }
                } else {
                    val link = LinkAnnotation.Clickable(
                        tag = "SPOILER",
                        linkInteractionListener = { onSpoilerClick(index) }
                    )
                    withStyle(style = SpanStyle(color = Color.Transparent, background = Color.DarkGray)) {
                        pushLink(link)
                        append(spoilerText)
                        pop()
                    }
                }
            }
            "MASKED_LINK" -> {
                val text = match!!.groupValues[1]
                val url = match.groupValues[2]
                withStyle(style = SpanStyle(color = primaryColor, textDecoration = TextDecoration.Underline)) {
                    pushStringAnnotation("URL", url)
                    appendDiscordMarkdown(text, revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick)
                    pop()
                }
            }
            "URL", "URL_SUPPRESSED" -> {
                val url = if (tag == "URL_SUPPRESSED") match!!.groupValues[1] else match!!.groupValues[0]
                withStyle(style = SpanStyle(color = primaryColor, textDecoration = TextDecoration.Underline)) {
                    pushStringAnnotation("URL", url)
                    append(url)
                    pop()
                }
            }
            "BOLD" -> withStyle(style = SpanStyle(fontWeight = FontWeight.Bold)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "UNDERLINE" -> withStyle(style = SpanStyle(textDecoration = TextDecoration.Underline)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "ITALIC" -> withStyle(style = SpanStyle(fontStyle = FontStyle.Italic)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick) 
            }
            "STRIKE" -> withStyle(style = SpanStyle(textDecoration = TextDecoration.LineThrough)) { 
                appendDiscordMarkdown(match!!.groupValues[1], revealedSpoilers, primaryColor, navigationStore, guildStore, userStore, profileStore, onSpoilerClick)
            }
            "CODE_BLOCK" -> {
                val language = match!!.groupValues[1].lowercase().let { languageAliases[it] ?: it }
                val code = match.groupValues[2]
                withStyle(style = SpanStyle(
                    fontFamily = FontFamily.Monospace,
                    fontSize = 13.sp,
                    background = Color.Black.copy(alpha = 0.3f)
                )) {
                    append("\n")
                    appendCodeWithHighlighting(code, language)
                    append("\n")
                }
            }
            "CODE" -> withStyle(style = SpanStyle(fontFamily = FontFamily.Monospace, background = Color.LightGray.copy(alpha = 0.2f))) { 
                append(match!!.groupValues[1]) 
            }
            "EMOJI" -> {
                val animated = match!!.groupValues[1] == "a"
                val name = match.groupValues[3]
                val id = match.groupValues[4]
                val key = "$id:${if (animated) "a" else "p"}:$name"
                pushStringAnnotation("EMOJI", key)
                appendInlineContent(key, ":$name:")
                pop()
            }
            "TIMESTAMP" -> {
                val epoch = match!!.groupValues[1].toLongOrNull() ?: 0L
                val format = match.groupValues[2].ifEmpty { "f" }
                val formatted = DateTimeUtils.formatDiscordTimestamp(epoch, format)
                withStyle(SpanStyle(background = Color.Gray.copy(alpha = 0.1f))) {
                    append(formatted)
                }
            }
            "SLASH_COMMAND" -> {
                val name = match!!.groupValues[1]
                val mentionBg = primaryColor.copy(alpha = 0.1f)
                withStyle(style = SpanStyle(color = primaryColor, fontWeight = FontWeight.Medium, background = mentionBg)) {
                    append("/$name")
                }
            }
            "MENTION", "CHANNEL", "ROLE", "EVERYONE", "HERE" -> {
                val id = if (tag == "EVERYONE" || tag == "HERE") "" else match!!.groupValues[1]
                var name = id
                var prefix = "@"
                var mentionBg = primaryColor.copy(alpha = 0.1f)
                var mentionColor = primaryColor
                
                when(tag) {
                    "MENTION" -> {
                        val member = userStore.getMember(navigationStore.selectedGuild?.id ?: "", id)
                        val user = userStore.getUser(id)
                        name = member?.nick ?: user?.let { it.global_name ?: it.username } ?: id
                        
                        if (member != null && navigationStore.selectedGuild != null) {
                            val colorRole = member.getRoleColorRole(navigationStore.selectedGuild)
                            if (colorRole != null && colorRole.color != 0) {
                                mentionColor = Color(colorRole.color or 0xFF000000.toInt())
                                mentionBg = mentionColor.copy(alpha = 0.15f)
                            }
                        }
                    }
                    "CHANNEL" -> {
                        prefix = "#"
                        name = guildStore.allGuildChannels.value[id]?.name ?: id
                    }
                    "ROLE" -> {
                        val role = navigationStore.selectedGuild?.roles?.find { it.id == id }
                        name = role?.name ?: id
                        if (role != null && role.color != 0) {
                            mentionColor = Color(role.color or 0xFF000000.toInt())
                            mentionBg = mentionColor.copy(alpha = 0.15f)
                        }
                    }
                    "EVERYONE" -> name = "everyone"
                    "HERE" -> name = "here"
                }
                
                val isMentionLink = tag == "MENTION"
                withStyle(style = SpanStyle(color = mentionColor, fontWeight = FontWeight.Bold, background = mentionBg)) {
                    if (isMentionLink) {
                        pushStringAnnotation("MENTION", id)
                    }
                    append("$prefix$name")
                    if (isMentionLink) {
                        pop()
                    }
                }
            }
        }
        
        lastIndex = range.last + 1
        i++
    }
    
    if (lastIndex < content.length) {
        append(content.substring(lastIndex))
    }
}

// Older cached messages can contain an escaped opening bracket. Treat that
// legacy representation as a masked link too, so historical messages render
// the same way as newly received ones.
private val maskedLinkPattern = Regex("""(?:\\)?\[([^\]\r\n]+)]\(<?(https?://[^\s<>]+)>?\)""")

private val languageAliases = mapOf(
    "cs" to "csharp",
    "ps" to "powershell",
    "py" to "python",
    "ml" to "ocaml",
    "md" to "markdown",
    "xl" to "excel-formula",
    "kt" to "kotlin",
    "js" to "javascript",
    "ts" to "typescript",
)

private fun AnnotatedString.Builder.appendCodeWithHighlighting(code: String, language: String) {
    val keywordColor = Color(0xFFF47067)
    val stringColor = Color(0xFF96D0FF)
    val commentColor = Color(0xFF8B949E)
    val numberColor = Color(0xFFD2A8FF)
    val functionColor = Color(0xFFD2A8FF)
    
    val keywords = setOf(
        "val", "var", "fun", "class", "object", "if", "else", "when", "return", "import", "package", 
        "private", "public", "protected", "override", "suspend", "interface", "typealias", "it", "this", 
        "true", "false", "null", "let", "struct", "enum", "extension", "guard", "guard", "func", "throws", 
        "try", "catch", "async", "await", "for", "while", "do", "break", "continue", "in", "as", "is"
    )
    
    val regex = Regex("""(//.*|/\*[\s\S]*?\*/)|(".*?"|'.*?')|(\b\d+\b)|(\b\w+\b\()|(\b\w+\b)""")
    
    val matches = regex.findAll(code)
    var lastMatchEnd = 0
    
    for (match in matches) {
        if (match.range.first > lastMatchEnd) {
            append(code.substring(lastMatchEnd, match.range.first))
        }
        
        val group1 = match.groups[1] // comment
        val group2 = match.groups[2] // string
        val group3 = match.groups[3] // number
        val group4 = match.groups[4] // function
        val group5 = match.groups[5] // identifier/keyword
        
        when {
            group1 != null -> withStyle(SpanStyle(color = commentColor)) { append(match.value) }
            group2 != null -> withStyle(SpanStyle(color = stringColor)) { append(match.value) }
            group3 != null -> withStyle(SpanStyle(color = numberColor)) { append(match.value) }
            group4 != null -> {
                val funcName = match.value.dropLast(1)
                withStyle(SpanStyle(color = functionColor)) { append(funcName) }
                append("(")
            }
            group5 != null -> {
                if (keywords.contains(group5.value)) {
                    withStyle(SpanStyle(color = keywordColor)) { append(match.value) }
                } else {
                    append(match.value)
                }
            }
            else -> append(match.value)
        }
        
        lastMatchEnd = match.range.last + 1
    }
    
    if (lastMatchEnd < code.length) {
        append(code.substring(lastMatchEnd))
    }
}
