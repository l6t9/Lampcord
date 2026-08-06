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

@Composable
fun VoiceArea(channel: Channel, chatState: ChatState, modifier: Modifier = Modifier) {
    val guildId = channel.guild_id ?: "@me"
    val participants = chatState.voiceStates[guildId]?.values?.filter { it.channel_id == channel.id } ?: emptyList()

    Box(modifier = modifier.fillMaxSize().background(Color.Black)) {
        if (participants.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("No one is here", color = Color.White.copy(alpha = 0.6f))
            }
        } else {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 240.dp),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(bottom = 80.dp, start = 16.dp, end = 16.dp, top = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                items(participants) { state ->
                    VoiceParticipantCard(state, chatState)
                }
            }
        }

        // Bottom Controls Overlay
        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(bottom = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                modifier = Modifier.padding(bottom = 16.dp)
            ) {
                Button(
                    onClick = { /* TODO: Invite */ },
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White.copy(alpha = 0.1f)),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp)
                ) {
                    Icon(Icons.Filled.PersonAdd, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Invite to Voice")
                }
                Button(
                    onClick = { /* TODO: Activity */ },
                    colors = ButtonDefaults.buttonColors(containerColor = Color.White.copy(alpha = 0.1f)),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp)
                ) {
                    Icon(Icons.Filled.SportsEsports, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Choose Activity")
                }
            }

            Surface(
                color = Color(0xFF1E1F22),
                shape = RoundedCornerShape(24.dp),
                tonalElevation = 8.dp
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    VoiceControlButton(
                        icon = if (chatState.currentVoiceState?.self_mute == true) Icons.Filled.MicOff else Icons.Filled.Mic,
                        checked = chatState.currentVoiceState?.self_mute == true,
                        onClick = { chatState.toggleVoiceMute() },
                        tint = if (chatState.currentVoiceState?.self_mute == true) Color.White else Color.Unspecified
                    )
                    VoiceControlButton(
                        icon = if (chatState.currentVoiceState?.self_video == true) Icons.Filled.VisibilityOff else Icons.Filled.VideoCall,
                        checked = chatState.currentVoiceState?.self_video == true,
                        onClick = { chatState.toggleVoiceVideo() }
                    )
                    VoiceControlButton(
                        icon = Icons.Filled.ScreenShare,
                        checked = chatState.currentVoiceState?.self_stream == true,
                        onClick = { chatState.toggleVoiceStream() }
                    )
                    VoiceControlButton(
                        icon = Icons.Filled.Star,
                        onClick = { /* Effects/Stickers */ }
                    )
                    VoiceControlButton(
                        icon = Icons.Filled.MoreHoriz,
                        onClick = { /* More */ }
                    )
                    
                    Spacer(Modifier.width(8.dp))
                    
                    IconButton(
                        onClick = { chatState.disconnectFromVoice() },
                        modifier = Modifier
                            .size(40.dp)
                            .background(MaterialTheme.colorScheme.error, CircleShape)
                    ) {
                        Icon(Icons.AutoMirrored.Filled.Logout, null, tint = Color.White)
                    }
                }
            }
        }
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
