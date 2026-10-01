package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.api.CdnUrls
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.CleanUtils
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ChannelHeader(
    channel: Channel?,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    isCompactMemberList: Boolean = false
) {
    val currentUser by userStore.currentUser.collectAsState()
    val allUsers by userStore.users.collectAsState()
    val isChannelsAndRoles = navigationStore.isChannelsAndRolesVisible
    val isDesktop = me.lampu.lampcord.shared.utils.getPlatformName() == "desktop" || 
                    me.lampu.lampcord.shared.utils.getPlatformName() == "macos" || 
                    me.lampu.lampcord.shared.utils.getPlatformName() == "windows" || 
                    me.lampu.lampcord.shared.utils.getPlatformName() == "linux"

    TopAppBar(
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (isChannelsAndRoles) {
                    Text(
                        text = "Channels & Roles", 
                        style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold, fontSize = 18.sp)
                    )
                } else if (channel != null) {
                    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                    val name = remember(channel, allUsers, isDm, currentUser) {
                        if (isDm) {
                            if (channel.type == 3) {
                                if (!channel.name.isNullOrBlank()) {
                                    channel.name
                                } else {
                                    val recipients = channel.recipients?.mapNotNull { allUsers[it.id] ?: it }
                                        ?: channel.recipient_ids?.mapNotNull { allUsers[it] }
                                        ?: emptyList()
                                    val otherRecipients = if (currentUser != null) recipients.filter { it.id != currentUser?.id } else recipients
                                    val displayList = if (otherRecipients.isNotEmpty()) otherRecipients else recipients
                                    displayList.mapNotNull { it.global_name ?: it.username }
                                        .joinToString(", ")
                                        .ifEmpty { "Unnamed Group DM" }
                                }
                            } else {
                                val recipientId = channel.recipients?.firstOrNull()?.id
                                    ?: channel.recipient_ids?.firstOrNull()
                                val recipient = recipientId?.let { allUsers[it] }
                                    ?: channel.recipients?.firstOrNull()
                                recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                            }
                        } else {
                            CleanUtils.cleanChannelName(channel.name ?: "unnamed")
                        }
                    }
                    Text(
                        text = name,
                        style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold, fontSize = 18.sp),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )

                    if (isDesktop && channel.topic?.isNotBlank() == true) {
                        Text(
                            text = " • ",
                            style = MaterialTheme.typography.titleLarge.copy(
                                fontWeight = FontWeight.Bold,
                                fontSize = 18.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
                            ),
                            modifier = Modifier.padding(horizontal = 4.dp)
                        )
                        Text(
                            text = channel.topic,
                            style = MaterialTheme.typography.bodyMedium.copy(
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            ),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false)
                        )
                    }
                }
            }
        },
        navigationIcon = {
            Box(Modifier.padding(start = 16.dp)) {
                if (isChannelsAndRoles) {
                    Icon(
                        imageVector = Icons.Filled.Flag,
                        contentDescription = null,
                        modifier = Modifier.size(24.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                } else if (channel != null) {
                    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                    val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12

                    if (channel.type == 3) {
                        val groupIconUrl = channel.icon?.let { CdnUrls.getChannelIconUrl(channel.id, it, 96) }
                        if (groupIconUrl != null) {
                            Box(modifier = Modifier.size(28.dp)) {
                                AvatarWithDecoration(
                                    avatarUrl = groupIconUrl,
                                    decorationData = null,
                                    size = 28.dp
                                )
                            }
                        } else {
                            Icon(
                                imageVector = Icons.Rounded.Groups,
                                contentDescription = null,
                                modifier = Modifier.size(24.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else if (isDm) {
                        val recipientId = channel.recipients?.firstOrNull()?.id
                            ?: channel.recipient_ids?.firstOrNull()
                        val recipient = recipientId?.let { userStore.getUser(it) }
                            ?: channel.recipients?.firstOrNull()
                        if (recipient != null) {
                            Box(modifier = Modifier.size(28.dp)) {
                                AvatarWithDecoration(
                                    avatarUrl = recipient.avatar?.let { "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png?size=96" },
                                    decorationData = recipient.avatar_decoration_data,
                                    size = 28.dp,
                                    status = presenceStore.getUserStatus(
                                        recipient.id,
                                        currentUser?.id,
                                        settingsStore.userSettings?.status
                                    )
                                )
                            }
                        } else {
                            Icon(
                                imageVector = Icons.Rounded.AlternateEmail,
                                contentDescription = null,
                                modifier = Modifier.size(24.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    } else {
                        val icon = when (channel.type) {
                            15 -> Icons.Rounded.Forum
                            2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                            5 -> Icons.Filled.Campaign
                            else -> Icons.Filled.Tag
                        }
                        if (isThread) {
                            Icon(
                                imageVector = Icons.Rounded.Topic,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        } else {
                            Icon(
                                imageVector = icon,
                                contentDescription = null,
                                modifier = Modifier.size(24.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        },
        actions = {
            if (!isChannelsAndRoles && channel != null) {
                if (settingsStore.showCallButton && (channel.type == 1 || channel.type == 3)) VoiceCallButton(channel)
                if (settingsStore.showChatSearch) {
                    IconButton(onClick = { navigationStore.isSearchVisible = !navigationStore.isSearchVisible }) {
                        Icon(
                            Icons.Filled.Search,
                            "Search",
                            modifier = Modifier.size(22.dp),
                            tint = if (navigationStore.isSearchVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                if (channel.type == 0 || channel.type == 5 || channel.type == 15) {
                    IconButton(onClick = {
                        navigationStore.isThreadPanelVisible =
                            !navigationStore.isThreadPanelVisible
                    }) {
                        Icon(
                            Icons.Rounded.Topic,
                            "Threads",
                            modifier = Modifier.size(22.dp),
                            tint = if (navigationStore.isThreadPanelVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                if (settingsStore.showChatPins || isDesktop) {
                    IconButton(onClick = { navigationStore.isPinsVisible = !navigationStore.isPinsVisible }) {
                        Icon(
                            Icons.Filled.PushPin,
                            "Pins",
                            modifier = Modifier.size(22.dp),
                            tint = if (navigationStore.isPinsVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                if (channel.type == 2 || channel.type == 13) {
                    IconButton(onClick = { voiceStore.isVoiceChatTextVisible = !voiceStore.isVoiceChatTextVisible }) {
                        Icon(
                            Icons.Rounded.Chat,
                            if (voiceStore.isVoiceChatTextVisible) "Hide chat" else "Show chat",
                            modifier = Modifier.size(22.dp),
                            tint = if (voiceStore.isVoiceChatTextVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                if (isDesktop && channel.type != 15 && (channel.guild_id != null || channel.type == 1 || channel.type == 3)) {
                    val memberListVisible =
                        if (isCompactMemberList) navigationStore.isMemberListModalVisible
                        else navigationStore.isProfilePanelVisible
                    IconButton(onClick = {
                        if (isCompactMemberList) {
                            navigationStore.isMemberListModalVisible = !navigationStore.isMemberListModalVisible
                        } else {
                            navigationStore.isProfilePanelVisible = !navigationStore.isProfilePanelVisible
                        }
                    }) {
                        Icon(
                            imageVector = if (channel.type == 1) Icons.Filled.AccountCircle else Icons.Filled.Group,
                            contentDescription = "Toggle Member List",
                            modifier = Modifier.size(24.dp),
                            tint = if (memberListVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        },
        expandedHeight = 56.dp,
        colors = TopAppBarDefaults.topAppBarColors(
            containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
            scrolledContainerColor = MaterialTheme.colorScheme.surfaceContainerLow
        )
    )
}
