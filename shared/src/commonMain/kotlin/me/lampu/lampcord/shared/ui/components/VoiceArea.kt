package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.VoiceState
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.voice.rememberVoiceJoin
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun VoiceArea(
    channel: Channel,
    modifier: Modifier = Modifier,
    voiceStore: VoiceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val participants = voiceStore.voiceStates[channel.guild_id ?: "@me"]?.values?.filter { it.channel_id == channel.id }.orEmpty()
    val isJoined = voiceStore.activeChannel?.id == channel.id

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            if (participants.isEmpty()) {
                VoiceEmptyState(channel, isJoined, voiceStore)
            } else {
                VoiceStageGrid(participants, voiceStore, userStore)
            }

            if (isJoined) {
                VoiceControlsToolbar(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 24.dp)
                        .zIndex(1f),
                    voiceStore = voiceStore
                )
            }
        }

        if (!isJoined) {
            VoiceJoinBar(channel, voiceStore)
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun VoiceControlsToolbar(
    modifier: Modifier = Modifier,
    voiceStore: VoiceStore
) {
    HorizontalFloatingToolbar(
        expanded = true,
        modifier = modifier,
        floatingActionButton = {
            FloatingToolbarDefaults.StandardFloatingActionButton(
                onClick = { voiceStore.disconnectFromVoice() },
                containerColor = MaterialTheme.colorScheme.error,
                contentColor = MaterialTheme.colorScheme.onError
            ) {
                Icon(Icons.AutoMirrored.Filled.Logout, "Disconnect")
            }
        },
        content = {
            IconButton(onClick = voiceStore::toggleVoiceMute) {
                Icon(
                    if (voiceStore.selfMuted) Icons.Filled.MicOff else Icons.Filled.Mic,
                    if (voiceStore.selfMuted) "Unmute" else "Mute",
                    tint = if (voiceStore.selfMuted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            IconButton(onClick = voiceStore::toggleVoiceDeaf) {
                Icon(
                    if (voiceStore.selfDeafened) Icons.Filled.HeadsetOff else Icons.Filled.Headphones,
                    if (voiceStore.selfDeafened) "Undeafen" else "Deafen",
                    tint = if (voiceStore.selfDeafened) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            IconButton(onClick = { /* TODO: Screen Share */ }) {
                Icon(Icons.Filled.ScreenShare, "Share Screen")
            }
            IconButton(onClick = { /* TODO: Camera */ }) {
                Icon(Icons.Filled.Videocam, "Camera")
            }
        }
    )
}

@Composable
private fun VoiceEmptyState(channel: Channel, isJoined: Boolean, voiceStore: VoiceStore) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Icons.Filled.Headphones,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )
        Spacer(Modifier.height(16.dp))
        Text(
            text = if (isJoined) "Connected · Waiting for others" else "No one is here yet",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        if (channel.type == 13) {
            Text(
                text = "Stage channels do not support DAVE calls here.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
    }
}

@Composable
private fun VoiceStageGrid(
    participants: List<VoiceState>,
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 240.dp),
        contentPadding = PaddingValues(16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxSize()
    ) {
        items(participants, key = { it.user_id }) { state ->
            VoiceParticipantTile(state, voiceStore, userStore)
        }
    }
}

@Composable
private fun VoiceParticipantTile(
    state: VoiceState,
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    val user = userStore.getUser(state.user_id)
    val member = state.guild_id?.let { userStore.getMember(it, state.user_id) }
    val isSpeaking = state.user_id in voiceStore.connection.speaking
    val isSilenced = state.self_mute || state.mute || state.self_deaf || state.deaf
    
    val speakingBorderColor by animateColorAsState(
        if (isSpeaking) Color(0xFF23A559) else Color.Transparent
    )
    val silencedBorderColor by animateColorAsState(
        if (isSilenced) Color(0xFFF23F42) else Color.Transparent
    )

    Surface(
        color = Color(0xFF2B2D31),
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .aspectRatio(16f / 9f)
            .fillMaxWidth()
            .then(
                when {
                    isSpeaking -> Modifier.border(2.dp, speakingBorderColor, RoundedCornerShape(12.dp))
                    isSilenced -> Modifier.border(2.dp, silencedBorderColor, RoundedCornerShape(12.dp))
                    else -> Modifier
                }
            )
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            Box(
                modifier = Modifier
                    .align(Alignment.Center)
                    .size(80.dp)
            ) {
                AvatarWithDecoration(
                    avatarUrl = user?.avatar?.let { "https://cdn.discordapp.com/avatars/${state.user_id}/$it.png?size=160" },
                    decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
                    size = 80.dp
                )
            }

            Surface(
                color = Color.Black.copy(alpha = 0.5f),
                shape = RoundedCornerShape(4.dp),
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(8.dp)
            ) {
                Text(
                    text = member?.nick ?: user?.global_name ?: user?.username ?: "Unknown User",
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    color = Color(0xFFDBDEE1),
                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                )
            }

            Row(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(8.dp)
                    .background(Color.Black.copy(alpha = 0.5f), RoundedCornerShape(4.dp))
                    .padding(4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                if (state.self_deaf || state.deaf) {
                    Icon(
                        Icons.Filled.HeadsetOff,
                        contentDescription = "Deafened",
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.error
                    )
                } else if (state.self_mute || state.mute) {
                    Icon(
                        Icons.Filled.MicOff,
                        contentDescription = "Muted",
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.error
                    )
                }
            }
        }
    }
}

@Composable
private fun VoiceJoinBar(
    channel: Channel,
    voiceStore: VoiceStore
) {
    val join = rememberVoiceJoin(voiceStore)
    Surface(
        color = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 4.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    text = "You are not in this channel",
                    style = MaterialTheme.typography.bodyMedium
                )
                Text(
                    text = "Audio is sent only after DAVE encryption is ready.",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Button(
                onClick = { join(channel, false) },
                colors = ButtonDefaults.buttonColors(
                    containerColor = Color(0xFF23A559)
                )
            ) {
                Icon(Icons.Filled.Call, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text(if (voiceStore.activeChannel == null) "Join Voice" else "Switch")
            }
        }
    }
}
