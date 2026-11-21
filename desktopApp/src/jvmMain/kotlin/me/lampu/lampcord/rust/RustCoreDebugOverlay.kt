package me.lampu.lampcord.rust

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore

@Composable
fun RustCoreDebugOverlay(
    bridge: RustCoreBridge,
    modifier: Modifier = Modifier,
    entityStore: EntityStore? = null,
    messageStore: MessageStore? = null,
    navigationStore: NavigationStore? = null,
) {
    val snap by bridge.snapshots.collectAsState()
    val mappedGuilds = remember(snap.seq) { RustSnapshotMapper.mapGuilds(snap) }
    val mappedChannels = remember(snap.seq) { RustSnapshotMapper.mapChannels(snap) }
    val mappedMessages = remember(snap.seq) { RustSnapshotMapper.mapMessages(snap) }

    val kotlinGuilds = entityStore?.guilds?.collectAsState()?.value
    val kotlinChannels = entityStore?.channels?.collectAsState()?.value
    val kotlinMessages = messageStore?.messages?.collectAsState()?.value

    Column(
        modifier = modifier
            .widthIn(max = 460.dp)
            .heightIn(max = 520.dp)
            .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.94f), RoundedCornerShape(8.dp))
            .padding(10.dp)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text("Rust Core PoC (read-only parity slice)", style = MaterialTheme.typography.labelLarge)
        Text("seq: ${snap.seq}", style = MaterialTheme.typography.bodySmall)
        Text("connection: ${snap.connection}", style = MaterialTheme.typography.bodySmall)
        Text("user: ${snap.currentUser ?: "-"} (${snap.currentUserId ?: "-"})", style = MaterialTheme.typography.bodySmall)
        Text(
            "snapshot: guilds=${snap.guilds.size} channels=${snap.channels.size} messages=${snap.messages.size}",
            style = MaterialTheme.typography.bodySmall
        )
        Text(
            "mapped UI models: guilds=${mappedGuilds.size} channels=${mappedChannels.size} messages=${mappedMessages.size}",
            style = MaterialTheme.typography.bodySmall
        )
        if (kotlinGuilds != null || kotlinChannels != null || kotlinMessages != null) {
            HorizontalDivider()
            Text("Kotlin stores (parity)", style = MaterialTheme.typography.labelMedium)
            Text(
                "stores: guilds=${kotlinGuilds?.size ?: "-"} channels=${kotlinChannels?.size ?: "-"} messages=${kotlinMessages?.size ?: "-"}",
                style = MaterialTheme.typography.bodySmall
            )
            Text(
                "nav: guild=${navigationStore?.selectedGuild?.id ?: "-"} channel=${navigationStore?.selectedChannel?.id ?: "-"} connected=${navigationStore?.isConnected ?: "-"}",
                style = MaterialTheme.typography.bodySmall
            )
        }
        snap.selectedGuildId?.let { Text("rust selected guild: $it", style = MaterialTheme.typography.bodySmall) }
        snap.selectedChannelId?.let { Text("rust selected channel: $it", style = MaterialTheme.typography.bodySmall) }
        snap.error?.let {
            Text("error: $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
        }

        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Button(onClick = {
                snap.guilds.firstOrNull()?.let { bridge.selectGuild(it.id) }
            }, enabled = snap.guilds.isNotEmpty()) {
                Text("Guild", style = MaterialTheme.typography.labelSmall)
            }
            Button(onClick = {
                snap.channels.firstOrNull()?.let { bridge.selectChannel(it.id) }
            }, enabled = snap.channels.isNotEmpty()) {
                Text("Channel", style = MaterialTheme.typography.labelSmall)
            }
            Button(onClick = {
                snap.selectedChannelId?.let { bridge.fetchMessages(it, null, 20.toUByte()) }
            }, enabled = snap.selectedChannelId != null) {
                Text("Fetch", style = MaterialTheme.typography.labelSmall)
            }
        }

        if (mappedGuilds.isNotEmpty()) {
            HorizontalDivider()
            Text("Guilds (mapped to shared.model.Guild)", style = MaterialTheme.typography.labelMedium)
            mappedGuilds.take(8).forEach { g ->
                Text("• ${g.name ?: g.id} (${g.id})", style = MaterialTheme.typography.bodySmall)
            }
        }
        if (mappedChannels.isNotEmpty()) {
            HorizontalDivider()
            Text("Channels (mapped to shared.model.Channel)", style = MaterialTheme.typography.labelMedium)
            mappedChannels.take(12).forEach { c ->
                Text("• #${c.name ?: c.id} [${c.type}] (${c.id})", style = MaterialTheme.typography.bodySmall)
            }
        }
        if (mappedMessages.isNotEmpty()) {
            HorizontalDivider()
            Text("Messages (mapped to shared.model.Message)", style = MaterialTheme.typography.labelMedium)
            mappedMessages.takeLast(8).forEach { m ->
                val author = m.author?.global_name ?: m.author?.username ?: "unknown"
                Text("• $author: ${m.content.take(80)}", style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}
