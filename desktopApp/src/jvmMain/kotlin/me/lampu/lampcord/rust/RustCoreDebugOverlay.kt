package me.lampu.lampcord.rust

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun RustCoreDebugOverlay(bridge: RustCoreBridge, modifier: Modifier = Modifier) {
    val snap by bridge.snapshots.collectAsState()
    Column(
        modifier = modifier
            .widthIn(max = 360.dp)
            .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.92f), RoundedCornerShape(8.dp))
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text("Rust Core PoC", style = MaterialTheme.typography.labelLarge)
        Text("seq: ${snap.seq}", style = MaterialTheme.typography.bodySmall)
        Text("connection: ${snap.connection}", style = MaterialTheme.typography.bodySmall)
        Text("user: ${snap.currentUser ?: "-"} (${snap.currentUserId ?: "-"})", style = MaterialTheme.typography.bodySmall)
        Text("guilds: ${snap.guilds.size} channels: ${snap.channels.size} messages: ${snap.messages.size}", style = MaterialTheme.typography.bodySmall)
        snap.error?.let { Text("error: $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
    }
}
