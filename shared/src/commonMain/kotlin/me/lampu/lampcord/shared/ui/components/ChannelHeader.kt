package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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

@Composable
fun ChannelHeader(
    channel: Channel?,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    val currentUser by userStore.currentUser.collectAsState()

    Surface(
        modifier = Modifier.fillMaxWidth().height(48.dp),
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = 0.dp,
        tonalElevation = 0.dp
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
                if (navigationStore.isChannelsAndRolesVisible) {
                    Icon(
                        imageVector = Icons.Filled.Flag,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(text = "Channels & Roles", style = MaterialTheme.typography.titleSmall)
                    Spacer(modifier = Modifier.weight(1f))
                } else if (channel != null) {
                    val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                    val isThread = channel.type == 10 || channel.type == 11 || channel.type == 12

                    if (isDm) {
                        val recipientId = channel.recipients?.firstOrNull()?.id
                            ?: channel.recipient_ids?.firstOrNull()
                        val recipient = recipientId?.let { userStore.getUser(it) }
                            ?: channel.recipients?.firstOrNull()
                        val name = if (channel.name?.isNotBlank() == true) {
                            channel.name
                        } else {
                            recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                        }

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
                                modifier = Modifier.size(18.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(text = name, style = MaterialTheme.typography.titleSmall)
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
                            ) // Thread
                        } else {
                            Icon(
                                imageVector = icon,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                    Text(text = CleanUtils.cleanChannelName(channel.name ?: "unnamed"), style = MaterialTheme.typography.titleSmall)
                    }

                    if (channel.topic?.isNotBlank() == true) {
                        Spacer(modifier = Modifier.width(12.dp))
                        Text("•", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Spacer(modifier = Modifier.width(12.dp))
                        Text(
                            text = channel.topic,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f)
                        )
                    } else {
                        Spacer(modifier = Modifier.weight(1f))
                    }

                    // Actions (Far Right)
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        if (channel.type == 0 || channel.type == 5 || channel.type == 15) {
                            IconButton(onClick = {
                                navigationStore.isThreadPanelVisible =
                                    !navigationStore.isThreadPanelVisible
                            }) {
                                Icon(
                                    Icons.Filled.Tag,
                                    "Threads",
                                    modifier = Modifier.size(20.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                        if (channel.type != 2 && channel.type != 13) {
                            IconButton(onClick = { navigationStore.isPinsVisible = true }) {
                                Icon(
                                    Icons.Filled.PushPin,
                                    "Pins",
                                    modifier = Modifier.size(20.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                }
            }
        }
    }

