package me.lampu.lampcord.shared.ui.components

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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
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
    settingsStore: SettingsStore = koinInject()
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
                        style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold)
                    )
                } else if (channel != null) {
                    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                    val name = remember(channel, allUsers, isDm) {
                        if (isDm) {
                            val recipientId = channel.recipients?.firstOrNull()?.id
                                ?: channel.recipient_ids?.firstOrNull()
                            val recipient = recipientId?.let { allUsers[it] }
                                ?: channel.recipients?.firstOrNull()
                            if (channel.name?.isNotBlank() == true) {
                                channel.name
                            } else {
                                recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                            }
                        } else {
                            CleanUtils.cleanChannelName(channel.name ?: "unnamed")
                        }
                    }
                    Text(
                        text = name,
                        style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    
                    if (!isDm && channel.topic?.isNotBlank() == true) {
                        Spacer(Modifier.width(8.dp))
                        Box(
                            modifier = Modifier
                                .width(1.dp)
                                .height(24.dp)
                                .background(MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(
                            text = channel.topic,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false)
                        )
                    }
                }
            }
        },
        navigationIcon = {
            Box(Modifier.padding(start = 12.dp)) {
                if (isChannelsAndRoles) {
                    Icon(
                        imageVector = Icons.Filled.Flag,
                        contentDescription = null,
                        modifier = Modifier.size(22.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                } else if (channel != null) {
                    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                    val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12

                    if (isDm) {
                        val recipientId = channel.recipients?.firstOrNull()?.id
                            ?: channel.recipient_ids?.firstOrNull()
                        val recipient = recipientId?.let { userStore.getUser(it) }
                            ?: channel.recipients?.firstOrNull()
                        if (recipient != null) {
                            Box(modifier = Modifier.size(24.dp)) {
                                AvatarWithDecoration(
                                    avatarUrl = recipient.avatar?.let { "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png?size=64" },
                                    decorationData = recipient.avatar_decoration_data,
                                    size = 24.dp,
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
                                modifier = Modifier.size(22.dp),
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
                            Text(
                                ">",
                                style = MaterialTheme.typography.titleMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        } else {
                            Icon(
                                imageVector = icon,
                                contentDescription = null,
                                modifier = Modifier.size(22.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        },
        actions = {
            if (!isChannelsAndRoles && channel != null) {
                if (settingsStore.showChatSearch) {
                    IconButton(onClick = { navigationStore.isSearchVisible = !navigationStore.isSearchVisible }) {
                        Icon(
                            Icons.Filled.Search,
                            "Search",
                            modifier = Modifier.size(20.dp),
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
                            Icons.Filled.Tag,
                            "Threads",
                            modifier = Modifier.size(20.dp),
                            tint = if (navigationStore.isThreadPanelVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                if (channel.type != 2 && channel.type != 13 && settingsStore.showChatPins) {
                    IconButton(onClick = { navigationStore.isPinsVisible = !navigationStore.isPinsVisible }) {
                        Icon(
                            Icons.Filled.PushPin,
                            "Pins",
                            modifier = Modifier.size(20.dp),
                            tint = if (navigationStore.isPinsVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                
                if (isDesktop && channel.type == 1) {
                    IconButton(onClick = { navigationStore.isProfilePanelVisible = !navigationStore.isProfilePanelVisible }) {
                        Icon(
                            Icons.Filled.AccountCircle,
                            "User Profile",
                            modifier = Modifier.size(20.dp),
                            tint = if (navigationStore.isProfilePanelVisible) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        },
        expandedHeight = 48.dp,
        colors = TopAppBarDefaults.topAppBarColors(
            containerColor = Color.Transparent,
            scrolledContainerColor = Color.Transparent
        )
    )
}
