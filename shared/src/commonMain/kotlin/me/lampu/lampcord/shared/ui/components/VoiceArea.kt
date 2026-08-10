package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.VoiceState
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun VoiceArea(channel: Channel, chatState: ChatState, modifier: Modifier = Modifier) {
    val guildId = channel.guild_id ?: "@me"
    val participants = chatState.voiceStates[guildId]?.values?.filter { it.channel_id == channel.id } ?: emptyList()

    Box(modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainerLowest)) {
        if (participants.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("No one is here", color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
            }
        } else {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 240.dp),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(bottom = 100.dp, start = 16.dp, end = 16.dp, top = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                items(participants) { state ->
                    VoiceParticipantCard(state, chatState)
                }
            }
        }

        // Floating Toolbar (M3 Expressive)
        HorizontalFloatingToolbar(
            expanded = true,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 32.dp),
            floatingActionButton = {
                FloatingToolbarDefaults.StandardFloatingActionButton(
                    onClick = { chatState.disconnectFromVoice() },
                    containerColor = MaterialTheme.colorScheme.error,
                    contentColor = MaterialTheme.colorScheme.onError,
                ) {
                    Icon(Icons.AutoMirrored.Filled.Logout, "Hang up")
                }
            },
            colors = FloatingToolbarDefaults.standardFloatingToolbarColors(),
            content = {
                VoiceControlButton(
                    icon = if (chatState.currentVoiceState?.self_mute == true) Icons.Filled.MicOff else Icons.Filled.Mic,
                    checked = chatState.currentVoiceState?.self_mute == true,
                    onClick = { chatState.toggleVoiceMute() },
                    tint = if (chatState.currentVoiceState?.self_mute == true) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface
                )
                VoiceControlButton(
                    icon = if (chatState.currentVoiceState?.self_video == true) Icons.Filled.VisibilityOff else Icons.Filled.VideoCall,
                    checked = chatState.currentVoiceState?.self_video == true,
                    onClick = { chatState.toggleVoiceVideo() },
                    tint = MaterialTheme.colorScheme.onSurface
                )
                VoiceControlButton(
                    icon = Icons.Filled.ScreenShare,
                    checked = chatState.currentVoiceState?.self_stream == true,
                    onClick = { chatState.toggleVoiceStream() },
                    tint = MaterialTheme.colorScheme.onSurface
                )
                VoiceControlButton(
                    icon = Icons.Filled.PersonAdd,
                    onClick = { /* Invite */ },
                    tint = MaterialTheme.colorScheme.onSurface
                )
                VoiceControlButton(
                    icon = Icons.Filled.SportsEsports,
                    onClick = { /* Activities */ },
                    tint = MaterialTheme.colorScheme.onSurface
                )
                VoiceControlButton(
                    icon = Icons.Filled.MoreHoriz,
                    onClick = { /* More */ },
                    tint = MaterialTheme.colorScheme.onSurface
                )
            }
        )
    }
}

@Composable
fun VoiceControlButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    checked: Boolean = false,
    onClick: () -> Unit,
    tint: Color = Color.White
) {
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .size(40.dp)
            .background(if (checked) Color.White.copy(alpha = 0.2f) else Color.Transparent, CircleShape)
    ) {
        Icon(icon, null, tint = tint, modifier = Modifier.size(20.dp))
    }
}

@Composable
fun VoiceParticipantCard(state: VoiceState, chatState: ChatState) {
    val user = chatState.userStore.getUser(state.user_id)
    val member = state.guild_id?.let { chatState.userStore.getMember(it, state.user_id) }
    val name = member?.nick ?: user?.global_name ?: user?.username ?: "Unknown"
    val avatarUrl = member?.avatar?.let { 
        "https://cdn.discordapp.com/guilds/${state.guild_id}/users/${state.user_id}/avatars/$it.png?size=160"
    } ?: user?.avatar?.let {
        "https://cdn.discordapp.com/avatars/${state.user_id}/$it.png?size=160"
    }

    Surface(
        modifier = Modifier.fillMaxWidth().aspectRatio(1.6f),
        shape = RoundedCornerShape(12.dp),
        color = Color(0xFF2B2D31)
    ) {
        Box(contentAlignment = Alignment.Center) {
            // Placeholder for Video / Stream
            if (state.self_video || state.self_stream == true) {
                Box(Modifier.fillMaxSize().background(Color.Black)) {
                    Text(
                        text = if (state.self_stream == true) "Streaming" else "Video Active",
                        modifier = Modifier.align(Alignment.Center),
                        color = Color.White,
                        style = MaterialTheme.typography.labelMedium
                    )
                }
            } else {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Box(modifier = Modifier.size(80.dp)) {
                        AvatarWithDecoration(
                            avatarUrl = avatarUrl,
                            decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
                            size = 80.dp
                        )
                    }
                }
            }

            // Name Label
            Surface(
                color = Color.Black.copy(alpha = 0.5f),
                shape = RoundedCornerShape(4.dp),
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(8.dp)
            ) {
                Text(
                    text = name,
                    style = MaterialTheme.typography.labelSmall,
                    color = Color.White,
                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                )
            }

            // Status Icons
            Row(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(8.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                if (state.self_mute || state.mute) {
                    Icon(Icons.Filled.MicOff, null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error)
                }
                if (state.self_deaf || state.deaf) {
                    Icon(Icons.Filled.HeadsetOff, null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error)
                }
            }
        }
    }
}
