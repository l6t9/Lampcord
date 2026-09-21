package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.gateway.VoicePhase
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.voice.rememberVoiceJoin
import org.koin.compose.koinInject

@Composable
fun VoiceConnectionPanel(voiceStore: VoiceStore = koinInject(), userStore: UserStore = koinInject()) {
    val channel = voiceStore.activeChannel ?: return
    var showVerification by remember { mutableStateOf(false) }
    var showScreenshareDialog by remember { mutableStateOf(false) }
    val secure = voiceStore.connection.phase == VoicePhase.SECURE
    val duration = voiceStore.voiceConnectionDuration
    val status = when (voiceStore.connection.phase) {
        VoicePhase.SECURE -> "DAVE encrypted · ${duration / 60}:${(duration % 60).toString().padStart(2, '0')}"
        VoicePhase.CONNECTED -> "Waiting for encrypted audio"
        else -> "Connecting"
    }
    Surface(color = MaterialTheme.colorScheme.surfaceContainerHigh) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f).padding(8.dp)) {
                Text(voiceChannelName(channel, userStore), maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.labelLarge)
                Text(status, style = MaterialTheme.typography.labelSmall, maxLines = 2)
            }
            IconButton(onClick = { showVerification = true }, enabled = secure) {
                Icon(Icons.Filled.Lock, "Verify DAVE encryption", tint = if (secure) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
            }
            IconButton(onClick = voiceStore::toggleVoiceMute) {
                Icon(if (voiceStore.selfMuted) Icons.Filled.MicOff else Icons.Filled.Mic, if (voiceStore.selfMuted) "Unmute microphone" else "Mute microphone")
            }
            IconButton(onClick = voiceStore::toggleVoiceDeaf) {
                Icon(if (voiceStore.selfDeafened) Icons.Filled.HeadsetOff else Icons.Filled.Headphones, if (voiceStore.selfDeafened) "Undeafen" else "Deafen")
            }
            IconButton(onClick = { showScreenshareDialog = true }) {
                Icon(Icons.Filled.ScreenShare, "Share your screen")
            }
            if (getPlatformName() == "android") {
                IconButton(onClick = voiceStore::toggleSpeaker) {
                    Icon(Icons.AutoMirrored.Filled.VolumeUp, if (voiceStore.speakerEnabled) "Use headset or earpiece" else "Use speaker",
                        tint = if (voiceStore.speakerEnabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            IconButton(onClick = { voiceStore.disconnectFromVoice() }) {
                Icon(Icons.AutoMirrored.Filled.Logout, "End call", tint = MaterialTheme.colorScheme.error)
            }
        }
    }
    if (showVerification) AlertDialog(
        onDismissRequest = { showVerification = false },
        title = { Text("DAVE encryption") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Compare this MLS epoch authenticator with the other participants over a trusted channel. It changes when the encryption group changes.")
                SelectionContainer { Text(voiceStore.connection.verificationCode.ifEmpty { "Waiting for a new encrypted epoch" }, style = MaterialTheme.typography.bodySmall) }
                Text("Keys are ephemeral for this call. Matching codes verify the group; they do not establish someone's real-world identity.", style = MaterialTheme.typography.bodySmall)
            }
        },
        confirmButton = { TextButton(onClick = { showVerification = false }) { Text("Close") } }
    )
    if (showScreenshareDialog) AlertDialog(
        onDismissRequest = { showScreenshareDialog = false },
        title = { Text("Screensharing & Viewing") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Screensharing and stream viewing capabilities are currently being configured for Lampcord.")
                Text("Following the Discord connection standards and modern desktop setups (like Serein), outbound video streaming uses platform-native capture pipelines and H.264 video encoders with DAVE end-to-end media encryption filters.", style = MaterialTheme.typography.bodySmall)
            }
        },
        confirmButton = { TextButton(onClick = { showScreenshareDialog = false }) { Text("OK") } }
    )
}

@Composable
fun VoiceCallButton(channel: Channel, voiceStore: VoiceStore = koinInject()) {
    val join = rememberVoiceJoin(voiceStore)
    LaunchedEffect(channel.id) { voiceStore.requestCall(channel) }
    if (channel.type == 1 || channel.type == 3) {
        IconButton(onClick = { join(channel, channel.id !in voiceStore.calls) }, enabled = voiceStore.activeChannel?.id != channel.id) {
            Icon(Icons.Filled.Call, if (channel.id in voiceStore.calls) "Join call" else "Start voice call")
        }
    }
}

@Composable
fun VoiceCallDialogs(voiceStore: VoiceStore = koinInject(), guildStore: GuildStore = koinInject(), userStore: UserStore = koinInject()) {
    val join = rememberVoiceJoin(voiceStore)
    val privateChannels by guildStore.privateChannels.collectAsState()
    voiceStore.incomingChannelId?.let { id ->
        val channel = privateChannels.find { it.id == id } ?: Channel(id, 1)
        AlertDialog(
            onDismissRequest = { voiceStore.declineCall(id) },
            title = { Text("Incoming voice call") },
            text = { Text(voiceChannelName(channel, userStore) + if (voiceStore.activeChannel != null) "\nAnswering will end your current call." else "") },
            confirmButton = { TextButton(onClick = { join(channel, false) }) { Text("Answer") } },
            dismissButton = { TextButton(onClick = { voiceStore.declineCall(id) }) { Text("Decline") } }
        )
    }
    voiceStore.error?.let { message ->
        AlertDialog(
            onDismissRequest = { voiceStore.error = null },
            title = { Text("Voice call") }, text = { Text(message) },
            confirmButton = { TextButton(onClick = { voiceStore.error = null }) { Text("Close") } }
        )
    }
}

private fun voiceChannelName(channel: Channel, users: UserStore): String = channel.name?.takeIf { it.isNotBlank() }
    ?: channel.recipients?.firstOrNull()?.let { it.global_name ?: it.username }
    ?: channel.recipient_ids?.firstOrNull()?.let { users.getUser(it)?.let { user -> user.global_name ?: user.username } }
    ?: "Voice call"
