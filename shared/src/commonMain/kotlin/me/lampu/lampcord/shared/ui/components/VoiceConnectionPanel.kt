package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun VoiceConnectionPanel(
    voiceStore: VoiceStore = koinInject(),
    guildStore: GuildStore = koinInject()
) {
    val voiceState = voiceStore.currentVoiceState ?: return
    val guilds by guildStore.guilds.collectAsState()
    val allChannels by guildStore.allGuildChannels.collectAsState()
    val guild = guilds.find { it.id == voiceState.guild_id }
    val channel = allChannels.values.find { it.id == voiceState.channel_id }
    
    val duration = voiceStore.voiceConnectionDuration
    val hours = duration / 3600
    val minutes = (duration % 3600) / 60
    val seconds = duration % 60
    val durationText = if (hours > 0) {
        "${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
    } else {
        "${minutes}:${seconds.toString().padStart(2, '0')}"
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .height(52.dp),
        color = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 2.dp,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = "Voice Connected",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary,
                        fontWeight = FontWeight.Bold
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = durationText,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                    )
                }
                Text(
                    text = "${channel?.name ?: "Unknown"} / ${guild?.name ?: "Direct Message"}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            IconButton(onClick = { voiceStore.toggleVoiceVideo() }) {
                Icon(
                    imageVector = Icons.Filled.VideoCall,
                    contentDescription = null,
                    modifier = Modifier.size(20.dp),
                    tint = if (voiceState.self_video) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            IconButton(onClick = { voiceStore.toggleVoiceStream() }) {
                Icon(
                    imageVector = Icons.Filled.ScreenShare,
                    contentDescription = null,
                    modifier = Modifier.size(20.dp),
                    tint = if (voiceState.self_stream == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            IconButton(
                onClick = { voiceStore.disconnectFromVoice(voiceState.guild_id) },
                colors = IconButtonDefaults.iconButtonColors(contentColor = MaterialTheme.colorScheme.error)
            ) {
                Icon(Icons.AutoMirrored.Filled.Logout, null, modifier = Modifier.size(20.dp))
            }
        }
    }
}
