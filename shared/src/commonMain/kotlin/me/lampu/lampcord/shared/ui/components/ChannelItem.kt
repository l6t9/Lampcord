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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun ChannelItem(channel: Channel, chatState: ChatState) {
    val isSelected = chatState.selectedChannel?.id == channel.id
    val isUnread by remember(channel, chatState.readStates[channel.id]) {
        derivedStateOf { chatState.isUnread(channel) }
    }
    val mentionCount by remember(channel, chatState.readStates[channel.id]) {
        derivedStateOf { chatState.getMentionCount(channel.id) }
    }
    
    val guildSettings = chatState.userGuildSettingsStore.userGuildSettings[channel.guild_id]
    val isMuted by remember(channel, guildSettings) {
        derivedStateOf { chatState.userGuildSettingsStore.isChannelMuted(channel.guild_id, channel.id) }
    }

    val guild = chatState.selectedGuild
    val member = chatState.currentMember
    val canView = remember(channel, guild, member) {
        if (guild == null || member == null) true
        else PermissionHelper.canViewChannel(member, guild, channel, chatState.currentUser?.id)
    }

    val contextMenuItems = remember(channel, chatState.userSettings, isMuted) {
        val items = mutableListOf<ContextMenuItem>()
        if (canView) {
            items.add(ContextMenuItem(if (isMuted) "Unmute Channel" else "Mute Channel", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff) {
                chatState.toggleMuteChannel(channel.guild_id ?: "@me", channel.id)
            })
            items.add(ContextMenuItem("Mark as Read", Icons.Filled.Check) { /* TODO */ })
        }
        items.add(ContextMenuItem("Copy Link", Icons.Filled.Link) {
            val guildId = channel.guild_id ?: "@me"
            setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
        })
        if (chatState.userSettings?.developer_mode == true) {
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
                            .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                            .background(MaterialTheme.colorScheme.onSurface)
                    )
                }

                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(34.dp)
                        .padding(horizontal = 8.dp),
                    onClick = { 
                        if (!canView) return@Surface
                        chatState.selectChannel(channel)
                    },
                    color = if (isSelected) 
                        MaterialTheme.colorScheme.surfaceVariant 
                    else Color.Transparent,
                    shape = RoundedCornerShape(12.dp),
                    enabled = canView
                ) {
                    Row(
                        modifier = Modifier
                            .padding(horizontal = 8.dp)
                            .alpha(if (isMuted && !isSelected) 0.5f else 1f),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val contentColor = if (isSelected || (isUnread && canView)) 
                            MaterialTheme.colorScheme.onSurface 
                        else MaterialTheme.colorScheme.onSurfaceVariant
                        
                        Icon(
                            imageVector = if (!canView) Icons.Rounded.Lock else when(channel.type) {
                                15 -> Icons.Outlined.Forum
                                2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                                5 -> Icons.Filled.Campaign
                                10, 11, 12 -> Icons.Filled.Tag // Thread icons
                                else -> Icons.Filled.Tag
                            },
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                            tint = contentColor
                        )
                        
                        Spacer(modifier = Modifier.width(8.dp))
                        
                        Text(
                            text = channel.name ?: "unnamed",
                            style = MaterialTheme.typography.bodyMedium,
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
                                onClick = { chatState.isVoiceChatTextVisible = !chatState.isVoiceChatTextVisible },
                                modifier = Modifier.size(24.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Chat,
                                    contentDescription = "Open Chat",
                                    modifier = Modifier.size(16.dp),
                                    tint = if (chatState.isVoiceChatTextVisible) MaterialTheme.colorScheme.primary else contentColor
                                )
                            }
                        }
                    }
                }
            }

            // Voice Participants
            if (channel.type == 2 || channel.type == 13) {
                val participants = chatState.voiceStates[channel.guild_id ?: "@me"]?.values?.filter { it.channel_id == channel.id } ?: emptyList()
                if (participants.isNotEmpty()) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(start = 38.dp, bottom = 4.dp, end = 8.dp),
                        verticalArrangement = Arrangement.spacedBy(2.dp)
                    ) {
                        participants.forEach { state ->
                            VoiceParticipantSidebarItem(state, chatState)
                        }
                    }
                }
            }
            
            // Threads (if any are associated with this channel)
            val threads = chatState.guildStore.allGuildChannels[channel.guild_id]?.filter { it.parent_id == channel.id && it.isThread() } ?: emptyList()
            if (threads.isNotEmpty() && isSelected) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(start = 32.dp)
                ) {
                    threads.forEach { thread ->
                        ChannelItem(thread, chatState)
                    }
                }
            }
        }
    }
}

fun Channel.isThread() = type in listOf(10, 11, 12)

@Composable
fun VoiceParticipantSidebarItem(state: me.lampu.lampcord.shared.model.VoiceState, chatState: ChatState) {
    val user = chatState.userStore.getUser(state.user_id)
    val member = state.guild_id?.let { chatState.userStore.getMember(it, state.user_id) }
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
        AvatarWithDecoration(
            avatarUrl = avatarUrl,
            decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
            size = 18.dp
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
