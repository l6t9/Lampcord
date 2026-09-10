package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.voice.rememberVoiceJoin
import org.koin.compose.koinInject

@Composable
fun VoiceArea(channel: Channel, modifier: Modifier = Modifier, voiceStore: VoiceStore = koinInject(), userStore: UserStore = koinInject()) {
    val participants = voiceStore.voiceStates[channel.guild_id ?: "@me"]?.values?.filter { it.channel_id == channel.id }.orEmpty()
    val join = rememberVoiceJoin(voiceStore)
    Column(modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainerLowest).padding(24.dp)) {
        Text(channel.name ?: "Voice call", style = MaterialTheme.typography.headlineSmall)
        Text("${participants.size} in voice", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(24.dp))
        if (participants.isEmpty()) {
            Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Text("No one is here yet", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items(participants, key = { it.user_id }) { state ->
                    val user = userStore.getUser(state.user_id)
                    val member = state.guild_id?.let { userStore.getMember(it, state.user_id) }
                    val speaking = state.user_id in voiceStore.connection.speaking
                    ListItem(
                        headlineContent = { Text(member?.nick ?: user?.global_name ?: user?.username ?: state.user_id) },
                        supportingContent = { Text(if (speaking) "Speaking" else if (state.self_deaf || state.deaf) "Deafened" else if (state.self_mute || state.mute) "Muted" else "Listening") },
                        leadingContent = {
                            AvatarWithDecoration(
                                avatarUrl = user?.avatar?.let { "https://cdn.discordapp.com/avatars/${state.user_id}/$it.png?size=80" },
                                decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
                                size = 40.dp
                            )
                        },
                        trailingContent = {
                            Icon(if (state.self_mute || state.mute) Icons.Filled.MicOff else Icons.Filled.Mic,
                                if (speaking) "Speaking" else "Microphone", tint = if (speaking) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
                        },
                        colors = ListItemDefaults.colors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow)
                    )
                }
            }
        }
        if (channel.type == 13) {
            Text("Stage channels do not support DAVE calls here.", color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else if (voiceStore.activeChannel?.id != channel.id) {
            Button(onClick = { join(channel, false) }, modifier = Modifier.fillMaxWidth()) {
                Text(if (voiceStore.activeChannel == null) "Join voice" else "Switch to this channel")
            }
            Text("Audio is sent only after DAVE encryption is ready.", style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 8.dp))
        }
    }
}
