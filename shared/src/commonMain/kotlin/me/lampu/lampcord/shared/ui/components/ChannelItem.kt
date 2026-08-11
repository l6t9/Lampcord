package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
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
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject

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
    presenceStore: PresenceStore = koinInject()
) {
    val isSelected = navigationStore.selectedChannel?.id == channel.id
    val readStates by readStateStore.readStates.collectAsState()
    
    val isUnread by remember(channel, readStates[channel.id]) {
        derivedStateOf { readStateStore.isUnread(channel) }
    }
    val mentionCount by remember(channel, readStates[channel.id]) {
        derivedStateOf { readStateStore.getMentionCount(channel.id) }
    }
    
    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val guildSettings = userGuildSettings[channel.guild_id]
    val isMuted by remember(channel, guildSettings) {
        derivedStateOf { userGuildSettingsStore.isChannelMuted(channel.guild_id, channel.id) }
    }

    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val allMembers by userStore.members.collectAsState()
    
    val member = remember(guild, currentUser, allMembers) {
        val g = guild
        val u = currentUser
        if (g == null || u == null) null
        else allMembers[g.id]?.get(u.id)
    }
    
    val canView = remember(channel, guild, member) {
        val g = guild
        val u = currentUser
        if (g == null || member == null) true
        else PermissionHelper.canViewChannel(member, g, channel, u?.id)
    }

    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()
    
    val contextMenuItems = remember(channel, userSettings, isMuted) {
        val items = mutableListOf<ContextMenuItem>()
        if (canView) {
            items.add(ContextMenuItem(if (isMuted) "Unmute Channel" else "Mute Channel", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff) {
                guildStore.toggleMuteChannel(channel.guild_id ?: "@me", channel.id)
            })
            items.add(ContextMenuItem("Mark as Read", Icons.Filled.Check) { 
                scope.launch {
                    readStateStore.ackMessage(channel.id, channel.lastMessageId() ?: "0")
                }
            })
        }
        items.add(ContextMenuItem("Copy Link", Icons.Filled.Link) {
            val guildId = channel.guild_id ?: "@me"
            setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
        })
        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(channel.id) })
        }
        items
    }

    ContextMenu(items = contextMenuItems) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 1.dp)
                .alpha(if (canView) 1f else 0.4f),
            verticalArrangement = Arrangement.Center
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(40.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                if (isUnread && !isSelected && canView) {
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
                        .height(40.dp)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                    onClick = { 
                        if (!canView) return@Surface
                        navigationStore.selectChannel(channel)
                    },
                    color = if (isSelected) 
                        MaterialTheme.colorScheme.surfaceContainerHigh 
                    else Color.Transparent,
                    shape = MaterialTheme.shapes.small,
                    enabled = canView
                ) {
                    Row(
                        modifier = Modifier
                            .padding(horizontal = 12.dp)
                            .alpha(if (isMuted && !isSelected) 0.5f else 1f),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val contentColor = if (isSelected || (isUnread && canView)) 
                            MaterialTheme.colorScheme.onSurface 
                        else MaterialTheme.colorScheme.onSurfaceVariant
                        
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
                            modifier = Modifier.size(20.dp),
                            tint = contentColor
                        )
                        
                        Spacer(modifier = Modifier.width(12.dp))
                        
                        Text(
                            text = me.lampu.lampcord.shared.utils.CleanUtils.cleanChannelName(channel.name ?: "unnamed"),
                            style = MaterialTheme.typography.bodyLarge,
                            color = contentColor,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                        
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

                        // Voice Chat Text Icon
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

            // Voice Participants
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
            
            // Threads (if any are associated with this channel)
            val allChannels by guildStore.allGuildChannels.collectAsState()
            val threads = allChannels.values.filter { it.guild_id == channel.guild_id && it.parent_id == channel.id && it.isThread() }
            if (threads.isNotEmpty() && isSelected) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(start = 32.dp)
                ) {
                    threads.forEach { thread ->
                        ChannelItem(thread)
                    }
                }
            }
        }
    }
}

fun Channel.isThread() = type in listOf(10, 11, 12)

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
    val avatarUrl = member?.avatar?.let {
        "https://cdn.discordapp.com/guilds/${state.guild_id}/users/${state.user_id}/avatars/$it.png?size=40"
    } ?: user?.avatar?.let {
        "https://cdn.discordapp.com/avatars/${state.user_id}/$it.png?size=40"
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
            modifier = Modifier.weight(1f)
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
