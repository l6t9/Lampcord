package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.chat.ChannelNotificationsSheet
import me.lampu.lampcord.shared.ui.components.chat.InviteDialog
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.voice.rememberVoiceJoin
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.combinedClickableCursor

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun ChannelItem(
    channel: Channel,
    navigationStore: NavigationStore = koinInject(),
    readStateStore: ReadStateStore = koinInject(),
    userGuildSettingsStore: UserGuildSettingsStore = koinInject(),
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    typingStore: TypingStore = koinInject(),
    profileStore: ProfileStore = koinInject()
) {
    val isSelected = navigationStore.selectedChannel?.id == channel.id
    val readStates by readStateStore.readStates.collectAsState()
    
    val typingUsers by typingStore.typingUsers.collectAsState()
    val isSomeoneTyping = remember(channel.id, typingUsers) {
        typingUsers[channel.id]?.isNotEmpty() == true
    }

    val recentIds by readStateStore.recentIds.collectAsState()
    val isUnread = remember(channel, readStates[channel.id], recentIds[channel.id]) { readStateStore.isUnread(channel) }
    val mentionCount = remember(channel, readStates[channel.id]) { readStateStore.getMentionCount(channel.id) }
    
    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val guildSettings = userGuildSettings[channel.guild_id]
    val isMuted = remember(channel, guildSettings) { userGuildSettingsStore.isChannelMuted(channel.guild_id, channel.id) }

    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val allMembers by userStore.members.collectAsState()
    
    val member = remember(guild, currentUser, allMembers) {
        val u = currentUser
        if (guild == null || u == null) null
        else allMembers[guild.id]?.get(u.id)
    }
    
    val canView = remember(channel, guild, member) {
        val u = currentUser
        if (guild == null || member == null) true
        else PermissionHelper.canViewChannel(member, guild, channel, u?.id)
    }

    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
    var showVoiceJoinSheet by remember { mutableStateOf(false) }

    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val indication = ripple()

    var showNotificationsSheet by remember { mutableStateOf(false) }
    var showInviteDialog by remember { mutableStateOf(false) }
    val isDmChannel = channel.guild_id == null || channel.type == 1 || channel.type == 3

    val canManageChannel = remember(channel, guild, member) {
        if (guild == null || member == null) false
        else PermissionHelper.hasPermission(member, guild, channel, Permission.MANAGE_CHANNELS, currentUser?.id)
    }

    val isPrivate = remember(channel, guild) {
        if (guild == null) false
        else PermissionHelper.isChannelPrivate(guild, channel)
    }

    val contextMenuItems = remember(channel, userSettings, isMuted, canView, canManageChannel) {
        val items = mutableListOf<ContextMenuItem>()
        if (canView) {
            if (isDmChannel) {
                val recipientId = channel.recipients?.firstOrNull()?.id ?: channel.recipient_ids?.firstOrNull()
                if (recipientId != null) {
                    items.add(ContextMenuItem("Profile", Icons.Filled.AccountCircle, onClick = {
                        profileStore.showProfile(recipientId, null)
                    }, group = "Primary"))
                }
            }

            items.add(ContextMenuItem("Mark as Read", Icons.Filled.Check, onClick = {
                scope.launch {
                    readStateStore.mostRecentMessageId(channel)?.let { readStateStore.ackMessage(channel.id, it) }
                }
            }, group = "Primary"))
            
            items.add(ContextMenuItem(if (isMuted) "Unmute Channel" else "Mute Channel", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff, onClick = {
                guildStore.toggleMuteChannel(channel.guild_id ?: "@me", channel.id)
            }, group = "Primary"))

            val canManageThreads = if (guild == null || member == null) false 
                else PermissionHelper.hasPermission(member, guild, channel, Permission.MANAGE_THREADS, currentUser?.id)
            
            if (canManageThreads) {
                items.add(ContextMenuItem("Threads", Icons.Rounded.Forum, onClick = {
                    navigationStore.isThreadPanelVisible = true
                }, group = "Primary"))
            }

            items.add(ContextMenuItem("Notification Settings", Icons.Filled.Notifications, onClick = {
                showNotificationsSheet = true
            }, group = "Primary"))
            
            val canCreateInvite = if (guild == null || member == null) true
                else PermissionHelper.hasPermission(member, guild, channel, Permission.CREATE_INSTANT_INVITE, currentUser?.id)

            if (canCreateInvite && channel.type != 4 && channel.type != 2 && channel.type != 13) {
                items.add(ContextMenuItem("Invite People", Icons.Filled.PersonAdd, onClick = {
                    showInviteDialog = true
                }, group = "Primary"))
            }
            if (!isDmChannel && canManageChannel) {
                items.add(ContextMenuItem("Channel Settings", Icons.Filled.Settings, onClick = {
                    navigationStore.openChannelSettings(channel)
                }, group = "Primary"))
            }
        }
        items.add(ContextMenuItem("Copy Link", Icons.Filled.Link, onClick = {
            val guildId = channel.guild_id ?: "@me"
            setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
        }, group = "Utilities"))
        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns, onClick = { setClipboardText(channel.id) }, group = "Developer"))
        }
        items
    }

    ContextMenu(
        items = contextMenuItems,
        header = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    imageVector = when(channel.type) {
                        15 -> Icons.Rounded.Forum
                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                        5 -> Icons.Filled.Campaign
                        else -> Icons.Filled.Tag
                    },
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.width(12.dp))
                Text(
                    text = channel.name ?: "unnamed",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface
                )
            }
        }
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = if (settingsStore.messageSpacingMode == me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT) 1.dp else 0.dp)
                .alpha(if (canView) 1f else 0.4f),
            verticalArrangement = Arrangement.Center
        ) {
            val (itemHeight, iconSize) = when (settingsStore.messageSpacingMode) {
                me.lampu.lampcord.shared.settings.MessageSpacingMode.COMPACT -> 28.dp to 24.dp
                me.lampu.lampcord.shared.settings.MessageSpacingMode.DEFAULT -> 36.dp to 24.dp
                me.lampu.lampcord.shared.settings.MessageSpacingMode.SPACIOUS -> 44.dp to 24.dp
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(itemHeight),
                contentAlignment = Alignment.CenterStart
            ) {
                if (isUnread && !isMuted && !isSelected && canView) {
                    Box(
                        modifier = Modifier
                            .size(width = 4.dp, height = 12.dp)
                            .clip(MaterialTheme.shapes.extraSmall)
                            .background(MaterialTheme.colorScheme.onSurface)
                    )
                }

                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(itemHeight)
                        .padding(horizontal = 8.dp)
                        .clip(MaterialTheme.shapes.small)
                        .combinedClickableCursor(
                            interactionSource = interactionSource,
                            indication = indication,
                            enabled = canView,
                            onClick = {
                                if (isMobile && (channel.type == 2 || channel.type == 13)) {
                                    showVoiceJoinSheet = true
                                } else {
                                    navigationStore.selectChannel(channel, explicitlySelected = true)
                                }
                            },
                            onDoubleClick = {
                                if (!isMobile && (channel.type == 2 || channel.type == 13)) {
                                    voiceStore.connectToVoice(channel)
                                }
                            }
                        ),
                    color = if (isSelected) 
                        MaterialTheme.colorScheme.surfaceContainerHigh 
                    else Color.Transparent,
                    shape = MaterialTheme.shapes.small,
                ) {
                    Row(
                        modifier = Modifier
                            .padding(horizontal = 10.dp)
                            .alpha(if (isMuted && !isSelected) 0.5f else 1f),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val contentColor = when {
                            isSelected -> MaterialTheme.colorScheme.onSurface
                            isUnread && !isMuted && canView -> MaterialTheme.colorScheme.onSurface
                            isHovered -> MaterialTheme.colorScheme.onSurface
                            else -> MaterialTheme.colorScheme.onSurfaceVariant
                        }
                        
                        val iconAlpha = if (isSelected || (isUnread && !isMuted) || isHovered) 1f else 0.6f

                        Box(modifier = Modifier.size(iconSize)) {
                            Icon(
                                imageVector = if (!canView) {
                                    if (isSelected) Icons.Filled.Lock else Icons.Rounded.Lock
                                } else when(channel.type) {
                                    15 -> if (isSelected) Icons.Filled.Forum else Icons.Rounded.Forum
                                    2, 13 -> if (isSelected) Icons.AutoMirrored.Filled.VolumeUp else Icons.AutoMirrored.Rounded.VolumeUp
                                    5 -> if (isSelected) Icons.Filled.Campaign else Icons.Rounded.Campaign
                                    10, 11, 12 -> if (isSelected) Icons.Filled.Tag else Icons.Rounded.Tag
                                    else -> if (isSelected) Icons.Filled.Tag else Icons.Rounded.Tag
                                },
                                contentDescription = null,
                                modifier = Modifier.matchParentSize(),
                                tint = contentColor.copy(alpha = iconAlpha)
                            )
                            
                            if (canView && isPrivate && channel.type != 4) {
                                Surface(
                                    modifier = Modifier
                                        .align(Alignment.TopEnd)
                                        .offset(x = 2.dp, y = (-2).dp)
                                        .size(12.dp),
                                    shape = CircleShape,
                                    color = if (isSelected)
                                        MaterialTheme.colorScheme.surfaceContainerHigh
                                    else MaterialTheme.colorScheme.surface,
                                ) {
                                    Icon(
                                        imageVector = Icons.Filled.Lock,
                                        contentDescription = null,
                                        modifier = Modifier.padding(1.dp).fillMaxSize(),
                                        tint = contentColor.copy(alpha = iconAlpha)
                                    )
                                }
                            }
                        }
                        
                        Spacer(modifier = Modifier.width(12.dp))

                        val displayName = remember(channel.name) {
                            me.lampu.lampcord.shared.utils.CleanUtils.cleanChannelName(channel.name ?: "unnamed")
                        }

                        Text(
                            text = displayName,
                            style = MaterialTheme.typography.bodyLarge.copy(
                                fontWeight = if (isUnread && !isMuted) FontWeight.Bold else FontWeight.Medium
                            ),
                            color = contentColor,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f).then(
                                if (Settings.shared.reduceMotion) Modifier else Modifier.basicMarquee(
                                    iterations = 1,
                                    initialDelayMillis = 3000,
                                    velocity = 30.dp
                                )
                            )
                        )

                        if (isSomeoneTyping && !isSelected) {
                            TypingDots(
                                modifier = Modifier
                                    .padding(start = 4.dp)
                                    .size(12.dp)
                            )
                        }
                        
                        if (mentionCount > 0) {
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
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                            }
                        }

                        if ((channel.type == 2 || channel.type == 13) && isSelected) {
                            IconButton(
                                onClick = { voiceStore.isVoiceChatTextVisible = !voiceStore.isVoiceChatTextVisible },
                                modifier = Modifier.size(24.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Chat,
                                    contentDescription = "Open Chat",
                                    modifier = Modifier.size(16.dp),
                                    tint = if (voiceStore.isVoiceChatTextVisible) MaterialTheme.colorScheme.primary else contentColor
                                )
                            }
                        }
                    }
                }
            }

            if (channel.type == 2 || channel.type == 13) {
                val participants = voiceStore.voiceStates[channel.guild_id ?: "@me"]?.values?.filter { it.channel_id == channel.id } ?: emptyList()
                if (participants.isNotEmpty()) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(start = 38.dp, bottom = 4.dp, end = 8.dp),
                        verticalArrangement = Arrangement.spacedBy(2.dp)
                    ) {
                        participants.forEach { state ->
                            VoiceParticipantSidebarItem(state, userStore = userStore, presenceStore = presenceStore, settingsStore = settingsStore)
                        }
                    }
                }
            }
            
        }
    }

    if (showNotificationsSheet) {
        ChannelNotificationsSheet(
            channel = channel,
            onDismiss = { showNotificationsSheet = false }
        )
    }

    if (showInviteDialog) {
        InviteDialog(
            channel = channel,
            onDismiss = { showInviteDialog = false }
        )
    }

    if (showVoiceJoinSheet) {
        VoiceJoinSheet(
            channel = channel,
            onDismiss = { showVoiceJoinSheet = false }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VoiceJoinSheet(
    channel: Channel,
    onDismiss: () -> Unit,
    voiceStore: VoiceStore = koinInject()
) {
    val join = rememberVoiceJoin(voiceStore)
    
    DiscordBottomSheet(
        onDismissRequest = onDismiss
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = channel.name ?: "Voice Channel",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(Modifier.height(24.dp))
            
            Button(
                onClick = { 
                    join(channel, false)
                    onDismiss()
                },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(
                    containerColor = Color(0xFF23A559)
                )
            ) {
                Icon(Icons.Filled.Call, null)
                Spacer(Modifier.width(8.dp))
                Text("Join Voice")
            }
            
            Spacer(Modifier.height(8.dp))
            
            TextButton(
                onClick = onDismiss,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Cancel")
            }
            
            Spacer(Modifier.navigationBarsPadding())
        }
    }
}

@Composable
fun VoiceParticipantSidebarItem(
    state: me.lampu.lampcord.shared.model.VoiceState,
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    val user = userStore.getUser(state.user_id)
    val member = state.guild_id?.let { userStore.getMember(it, state.user_id) }
    val currentUser by userStore.currentUser.collectAsState()
    val name = member?.nick ?: user?.global_name ?: user?.username ?: "Unknown"
    val avatarUrl = if (state.guild_id != null && member?.avatar != null) {
        me.lampu.lampcord.shared.api.CdnUrls.getMemberAvatarUrl(state.guild_id, state.user_id, member.avatar, user?.avatar, 64)
    } else {
        me.lampu.lampcord.shared.api.CdnUrls.getUserAvatarUrl(state.user_id, user?.avatar, 64)
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(24.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        val u = currentUser
        AvatarWithDecoration(
            avatarUrl = avatarUrl,
            decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
            size = 18.dp,
            status = presenceStore.getUserStatus(state.user_id, u?.id, settingsStore.userSettings?.status)
        )
        Spacer(Modifier.width(8.dp))
        Text(
            text = name,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f).then(
                if (Settings.shared.reduceMotion) Modifier else Modifier.basicMarquee(
                    iterations = 1,
                    initialDelayMillis = 3000,
                    velocity = 30.dp
                )
            )
        )
        if (state.self_mute || state.mute) {
            Icon(
                imageVector = Icons.Filled.MicOff,
                contentDescription = null,
                modifier = Modifier.size(10.dp),
                tint = MaterialTheme.colorScheme.error
            )
        }
        if (state.self_deaf || state.deaf) {
            Spacer(Modifier.width(2.dp))
            Icon(
                imageVector = Icons.Filled.HeadsetOff,
                contentDescription = null,
                modifier = Modifier.size(10.dp),
                tint = MaterialTheme.colorScheme.error
            )
        }
    }
}

@Composable
fun ChannelSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(34.dp)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(16.dp),
            shape = RoundedCornerShape(4.dp)
        )
        Spacer(modifier = Modifier.width(8.dp))
        ShimmerBox(
            modifier = Modifier
                .width(120.dp)
                .height(12.dp),
            shape = RoundedCornerShape(6.dp)
        )
    }
}
