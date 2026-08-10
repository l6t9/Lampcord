package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.guilds.MuteServerDialog
import androidx.compose.foundation.background
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun GuildIcon(
    guild: Guild,
    isSelected: Boolean,
    chatState: ChatState,
    onClick: () -> Unit
) {
    val isAnimated = guild.icon?.startsWith("a_") == true
    val iconUrl = if (guild.icon != null) {
        val ext = if (isAnimated && isSelected) "gif" else "png"
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.$ext?size=96"
    } else null

    val isMuted by remember(guild.id, chatState.userGuildSettingsStore.userGuildSettings[guild.id]) {
        derivedStateOf { chatState.userGuildSettingsStore.isGuildMuted(guild.id) }
    }
    
    val isUnread by remember(guild.id, chatState.readStates, chatState.guildStore.allGuildChannels[guild.id], chatState.userGuildSettingsStore.userGuildSettings[guild.id]) {
        derivedStateOf { chatState.isGuildUnread(guild.id) }
    }
    val mentionCount by remember(guild.id, chatState.readStates, chatState.guildStore.allGuildChannels[guild.id]) {
        derivedStateOf { chatState.getGuildMentionCount(guild.id) }
    }

    var showMuteDialog by remember { mutableStateOf(false) }
    var showLeaveDialog by remember { mutableStateOf(false) }

    val contextMenuItems = remember(guild, isSelected, chatState.userSettings, isMuted) {
        val items = mutableListOf(
            ContextMenuItem(if (isMuted) "Unmute Server" else "Mute Server", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff) {
                if (isMuted) {
                    chatState.unmuteGuild(guild.id)
                } else {
                    showMuteDialog = true
                }
            },
            ContextMenuItem("Mark as Read", Icons.Filled.Check) {
                chatState.markGuildAsRead(guild.id)
            },
            ContextMenuItem("Server Profile", Icons.Filled.AccountCircle) {
                chatState.currentUser?.let { chatState.showProfile(it.id) }
            }
        )
        if (!isSelected) {
            items.add(ContextMenuItem("Leave Server", Icons.Filled.Logout, color = Color.Red) { 
                showLeaveDialog = true
            })
        }
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(guild.id) })
        }
        items
    }

    ContextMenu(items = contextMenuItems) {
        Box(contentAlignment = Alignment.Center) {
            RegularGuildItem(
                isSelected = isSelected,
                isUnread = isUnread,
                isMuted = isMuted,
                onClick = onClick,
                selectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent,
                unselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.surfaceVariant else Color.Transparent
            ) {
                if (iconUrl != null) {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = guild.name,
                        modifier = Modifier.fillMaxSize(),
                        filterQuality = FilterQuality.Medium
                    )
                } else {
                    val initials = remember(guild.name) {
                        guild.name?.split(" ")?.mapNotNull { it.firstOrNull() }?.joinToString("") ?: "?"
                    }
                    Text(
                        text = initials,
                        style = MaterialTheme.typography.titleMedium,
                        color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary,
                        textAlign = TextAlign.Center,
                        maxLines = 1,
                        fontSize = if (initials.length > 3) 12.sp else 16.sp
                    )
                }
            }

            if (mentionCount > 0) {
                Surface(
                    color = MaterialTheme.colorScheme.error,
                    shape = CircleShape,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(bottom = 2.dp, end = 2.dp)
                        .height(20.dp)
                        .widthIn(min = 20.dp),
                    shadowElevation = 2.dp
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

    if (showMuteDialog) {
        MuteServerDialog(
            guildName = guild.name ?: "Server",
            onDismiss = { showMuteDialog = false },
            onConfirm = { duration ->
                chatState.muteGuild(guild.id, duration)
                showMuteDialog = false
            }
        )
    }

    if (showLeaveDialog) {
        AlertDialog(
            onDismissRequest = { showLeaveDialog = false },
            icon = {
                Icon(
                    Icons.Filled.Logout,
                    contentDescription = null,
                    modifier = Modifier.size(32.dp),
                    tint = MaterialTheme.colorScheme.error
                )
            },
            title = {
                Text(
                    text = "Leave '${guild.name}'",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            text = {
                Text(
                    text = "Are you sure you want to leave '${guild.name}'? You will need an invite to join back.",
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(
                    onClick = {
                        chatState.leaveGuild(guild.id)
                        showLeaveDialog = false
                    },
                    modifier = Modifier.fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.error,
                        contentColor = MaterialTheme.colorScheme.onError
                    ),
                    shape = MaterialTheme.shapes.medium
                ) {
                    Text("Leave Server")
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { showLeaveDialog = false },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Cancel")
                }
            },
            shape = MaterialTheme.shapes.extraLarge,
            tonalElevation = AlertDialogDefaults.TonalElevation
        )
    }
}
