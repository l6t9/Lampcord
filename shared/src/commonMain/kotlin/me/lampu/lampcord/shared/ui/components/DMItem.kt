package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import kotlinx.coroutines.launch
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.handCursor

@Composable
fun DMItem(
    channel: Channel,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    userGuildSettingsStore: UserGuildSettingsStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    readStateStore: ReadStateStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    val isSelected = navigationStore.selectedChannel?.id == channel.id
    val allUsers by userStore.users.collectAsState()
    val readStates by readStateStore.readStates.collectAsState()
    
    val isGroupDm = channel.type == 3
    val currentUser by userStore.currentUser.collectAsState()

    val recipients = remember(channel.recipients, channel.recipient_ids, allUsers, currentUser) {
        val list = mutableListOf<me.lampu.lampcord.shared.model.User>()
        channel.recipients?.forEach { user ->
            list.add(allUsers[user.id] ?: user)
        }
        if (list.isEmpty() && !channel.recipient_ids.isNullOrEmpty()) {
            channel.recipient_ids.forEach { id ->
                allUsers[id]?.let { list.add(it) }
            }
        }
        val otherUsers = if (currentUser != null) list.filter { it.id != currentUser?.id } else list
        if (otherUsers.isNotEmpty()) otherUsers else list
    }

    val recipient = recipients.firstOrNull()

    val avatarUrl = remember(channel, isGroupDm, recipient) {
        if (isGroupDm) {
            channel.icon?.let { CdnUrls.getChannelIconUrl(channel.id, it, 64) }
        } else {
            recipient?.let { CdnUrls.getUserAvatarUrl(it.id, it.avatar, 64) }
        }
    }

    val name = remember(channel, isGroupDm, recipients, recipient) {
        if (isGroupDm) {
            if (!channel.name.isNullOrBlank()) {
                channel.name
            } else {
                recipients.mapNotNull { it.global_name ?: it.username }
                    .joinToString(", ")
                    .ifEmpty { "Unnamed Group DM" }
            }
        } else {
            recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"
        }
    }

    val userSettings = settingsStore.userSettings
    val status = if (!isGroupDm) {
        recipient?.let { presenceStore.getUserStatus(it.id, currentUser?.id, userSettings?.status) } ?: "offline"
    } else null

    var isHovered by remember { mutableStateOf(false) }

    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val isMuted = remember(channel, userGuildSettings) { userGuildSettingsStore.isChannelMuted(null, channel.id) }

    val scope = rememberCoroutineScope()

    val mentionCount = remember(channel.id, readStates) { readStateStore.getMentionCount(channel.id) }

    val errorColor = MaterialTheme.colorScheme.error
    val contextMenuItems = remember(channel, isGroupDm, userSettings, isMuted, recipient, errorColor) {
        val items = mutableListOf<ContextMenuItem>()
        items.add(ContextMenuItem(if (isMuted) "Unmute" else "Mute", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff, onClick = {
            guildStore.toggleMuteChannel("@me", channel.id)
        }, group = "Primary"))
        items.add(ContextMenuItem("Mark as Read", Icons.Filled.Check, onClick = {
            scope.launch {
                readStateStore.mostRecentMessageId(channel)?.let { readStateStore.ackMessage(channel.id, it) }
            }
        }, group = "Primary"))
        items.add(ContextMenuItem("Pinned Messages", Icons.Filled.PushPin, onClick = {
            navigationStore.isPinsVisible = true
        }, group = "Primary"))
        if (!isGroupDm && recipient != null) {
            items.add(ContextMenuItem("Profile", Icons.Filled.AccountCircle, onClick = { profileStore.showProfile(recipient.id) }, group = "Primary"))
        }
        items.add(ContextMenuItem(if (isGroupDm) "Leave Group" else "Close DM", Icons.Filled.Close, onClick = {
            navigationStore.closeDm(channel.id)
        }, color = errorColor, group = "Destructive"))

        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns, onClick = { setClipboardText(channel.id) }, group = "Developer"))
        }

        items
    }
    
    val nameplate = recipient?.collectibles?.nameplate

    val itemHeight = when (settingsStore.messageSpacingMode) {
        me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> 38.dp
        me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> 48.dp
        me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> 56.dp
    }

    val haptic = LocalHapticFeedback.current

    ContextMenu(items = contextMenuItems) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .height(itemHeight)
                .padding(horizontal = 8.dp)
                .handCursor()

                .alpha(if (isMuted && !isSelected) 0.5f else 1f)
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
                },
            onClick = {
                navigationStore.selectChannel(channel, explicitlySelected = true)
                haptic.performHapticFeedback(HapticFeedbackType.VirtualKey)
            },
            color = if (isSelected) 
                MaterialTheme.colorScheme.surfaceVariant 
            else Color.Transparent,
            shape = MaterialTheme.shapes.small
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                if (nameplate != null && (isHovered || isSelected)) {
                    val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                    AsyncImage(
                        model = decoUrl,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        alpha = 0.4f,
                        allowAnimation = isHovered
                    )
                }

                Row(
                    modifier = Modifier.padding(horizontal = 8.dp).fillMaxSize(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(modifier = Modifier.size(32.dp)) {
                        if (isGroupDm) {
                            if (avatarUrl != null) {
                                AvatarWithDecoration(
                                    avatarUrl = avatarUrl,
                                    decorationData = null,
                                    size = 32.dp,
                                    status = null,
                                    isHovered = isHovered
                                )
                            } else {
                                Surface(
                                    modifier = Modifier.fillMaxSize(),
                                    shape = CircleShape,
                                    color = MaterialTheme.colorScheme.primaryContainer
                                ) {
                                    Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                                        Icon(
                                            imageVector = Icons.Rounded.Groups,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                            modifier = Modifier.size(20.dp)
                                        )
                                    }
                                }
                            }
                        } else {
                            AvatarWithDecoration(
                                avatarUrl = avatarUrl,
                                decorationData = recipient?.avatar_decoration_data ?: recipient?.collectibles?.avatar_decoration,
                                size = 32.dp,
                                status = status,
                                isHovered = isHovered
                            )
                        }
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    UsernameView(
                        name = name,
                        style = if (!isGroupDm) recipient?.display_name_styles else null,
                        baseStyle = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                        ignoreEffects = !isHovered,
                        ignoreColors = !isHovered,
                        modifier = Modifier
                    )
                    if (!isGroupDm) {
                        recipient?.primary_guild?.let {
                            Spacer(Modifier.width(4.dp))
                            ClanTagView(it, modifier = Modifier.weight(1f, fill = false))
                        }
                        recipient?.let { 
                            UserTagView(it, modifier = Modifier.padding(start = 4.dp)) 
                        }
                    }

                    if (mentionCount > 0) {
                        Spacer(Modifier.weight(1f))
                        Surface(
                            color = MaterialTheme.colorScheme.error,
                            shape = CircleShape,
                            modifier = Modifier.height(16.dp).widthIn(min = 16.dp)
                        ) {
                            Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(horizontal = 4.dp)) {
                                Text(
                                    text = mentionCount.toString(),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onError,
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 10.sp
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
