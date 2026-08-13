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
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import kotlinx.coroutines.launch
import org.koin.compose.koinInject

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
    
    // Improved recipient resolution to avoid "Unnamed DM"
    val recipient = remember(channel.recipients, channel.recipient_ids, allUsers) {
        val recipientId = channel.recipients?.firstOrNull()?.id 
            ?: channel.recipient_ids?.firstOrNull()
            ?: return@remember null
            
        allUsers[recipientId] ?: channel.recipients?.firstOrNull()
    }
    
    val avatarUrl = recipient?.avatar?.let { 
        "https://cdn.discordapp.com/avatars/${recipient.id}/$it.png"
    }
    val name = recipient?.let { it.global_name ?: it.username } ?: "Unnamed DM"

    val currentUser by userStore.currentUser.collectAsState()
    val userSettings = settingsStore.userSettings
    val status = recipient?.let { presenceStore.getUserStatus(it.id, currentUser?.id, userSettings?.status) } ?: "offline"

    var isHovered by remember { mutableStateOf(false) }

    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val isMuted by remember(channel, userGuildSettings) {
        derivedStateOf { userGuildSettingsStore.isChannelMuted(null, channel.id) }
    }

    val scope = rememberCoroutineScope()

    val contextMenuItems = remember(channel, userSettings, isMuted, recipient) {
        val items = mutableListOf(
            ContextMenuItem(if (isMuted) "Unmute" else "Mute", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff) {
                guildStore.toggleMuteChannel("@me", channel.id)
            },
            ContextMenuItem("Mark as Read", Icons.Filled.Check) {
                scope.launch {
                    readStateStore.ackMessage(channel.id, channel.lastMessageId() ?: "0")
                }
            },
            ContextMenuItem("Profile", Icons.Filled.AccountCircle) { recipient?.let { profileStore.showProfile(it.id) } },
            ContextMenuItem("Close DM", Icons.Filled.Close, color = Color.Red) { /* TODO */ }
        )
        if (userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(channel.id) })
        }
        items
    }
    
    val nameplate = recipient?.collectibles?.nameplate

    ContextMenu(items = contextMenuItems) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .padding(horizontal = 8.dp)
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
            onClick = { navigationStore.selectChannel(channel, explicitlySelected = true) },
            color = if (isSelected) 
                MaterialTheme.colorScheme.surfaceVariant 
            else Color.Transparent,
            shape = MaterialTheme.shapes.small
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                // Nameplate background (only on hover/selected)
                if (nameplate != null && (isHovered || isSelected)) {
                    val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                    AsyncImage(
                        model = decoUrl,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        alpha = 0.4f
                    )
                }

                Row(
                    modifier = Modifier.padding(horizontal = 8.dp).fillMaxSize(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(modifier = Modifier.size(32.dp)) {
                        AvatarWithDecoration(
                            avatarUrl = avatarUrl,
                            decorationData = recipient?.avatar_decoration_data ?: recipient?.collectibles?.avatar_decoration,
                            size = 32.dp,
                            status = status
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    UsernameView(
                        name = name,
                        style = recipient?.display_name_styles,
                        baseStyle = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        ignoreEffects = !isHovered,
                        ignoreColors = !isHovered
                    )
                    recipient?.primary_guild?.let {
                        Spacer(Modifier.width(4.dp))
                        ClanTagView(it)
                    }
                    recipient?.let { 
                        UserTagView(it, modifier = Modifier.padding(start = 4.dp)) 
                    }
                }
            }
        }
    }
}
